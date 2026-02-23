use anyhow::{anyhow, Context, Result};
use aws_sdk_s3::{primitives::ByteStream, Client};
use std::env;
use uuid::Uuid;

#[derive(Clone)]
pub struct StorageService {
    client: Client,
    bucket: String,
    region: String,
}

impl StorageService {
    pub async fn new() -> Self {
        let config = aws_config::load_from_env().await;
        let client = Client::new(&config);

        let bucket = env::var("AWS_S3_BUCKET").unwrap_or_else(|_| "change-me".to_string());
        let region = env::var("AWS_REGION").unwrap_or_else(|_| "us-east-1".to_string());

        StorageService {
            client,
            bucket,
            region,
        }
    }

    pub async fn upload_bytes(
        &self,
        user_id: i32,
        file_name: &str,
        mime_type: &str,
        bytes: Vec<u8>,
    ) -> Result<String> {
        if self.bucket == "change-me" {
            return Err(anyhow!(
                "AWS_S3_BUCKET is not configured (set AWS_S3_BUCKET)"
            ));
        }

        let key = format!(
            "documents/{}/{}-{}",
            user_id,
            Uuid::new_v4(),
            sanitize_file_name(file_name)
        );

        self.client
            .put_object()
            .bucket(&self.bucket)
            .key(&key)
            .content_type(mime_type)
            .body(ByteStream::from(bytes))
            .send()
            .await
            .with_context(|| format!("failed to upload object to s3: {}/{}", self.bucket, key))?;

        Ok(self.object_url(&key))
    }

    pub async fn download_to_bytes(&self, key: &str) -> Result<Vec<u8>> {
        let out = self
            .client
            .get_object()
            .bucket(&self.bucket)
            .key(key)
            .send()
            .await
            .with_context(|| format!("failed to get object from s3: {}/{}", self.bucket, key))?;

        let data = out
            .body
            .collect()
            .await
            .context("failed to read s3 object body")?
            .into_bytes()
            .to_vec();

        Ok(data)
    }

    pub fn bucket(&self) -> &str {
        &self.bucket
    }

    pub fn object_url(&self, key: &str) -> String {
        // Simple virtual-hosted-style URL. For some regions / setups this may differ.
        format!(
            "https://{}.s3.{}.amazonaws.com/{}",
            self.bucket, self.region, key
        )
    }
}

fn sanitize_file_name(name: &str) -> String {
    name.chars()
        .map(|c| if c.is_ascii_alphanumeric() || c == '.' || c == '-' || c == '_' { c } else { '_' })
        .collect()
}

