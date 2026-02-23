use anyhow::{Context, Result};
use ethers::prelude::*;
use std::env;
use std::sync::Arc;
use std::time::Duration;

#[derive(Clone, Debug)]
pub struct BlockchainAnchor {
    pub transaction_hash: String,
    pub block_number: i64,
    pub status: String, // e.g. confirmed | pending | skipped | failed
}

#[derive(Clone)]
pub struct BlockchainService {
    client: Option<Arc<SignerMiddleware<Provider<Http>, LocalWallet>>>,
    from: Option<Address>,
}

impl BlockchainService {
    pub fn new() -> Self {
        let rpc_url = env::var("ETHEREUM_RPC_URL").ok();
        let private_key = env::var("PRIVATE_KEY").ok();

        if let (Some(rpc_url), Some(private_key)) = (rpc_url, private_key) {
            let provider = Provider::<Http>::try_from(rpc_url)
                .expect("invalid ETHEREUM_RPC_URL")
                .interval(Duration::from_millis(700));

            let wallet: LocalWallet = private_key
                .parse()
                .expect("invalid PRIVATE_KEY (expected hex private key)");

            // Chain id is optional; if it fails, default is used by wallet.
            let wallet = if let Ok(chain_id) = env::var("CHAIN_ID").ok().and_then(|s| s.parse().ok()) {
                wallet.with_chain_id(chain_id)
            } else {
                wallet
            };

            let from = Some(wallet.address());
            let client = SignerMiddleware::new(provider, wallet);

            BlockchainService {
                client: Some(Arc::new(client)),
                from,
            }
        } else {
            BlockchainService {
                client: None,
                from: None,
            }
        }
    }

    pub fn is_configured(&self) -> bool {
        self.client.is_some()
    }

    /// Anchors a document hash on-chain by sending a zero-value transaction with the hash bytes
    /// embedded in the data field.
    ///
    /// This avoids requiring a contract ABI, but still produces an immutable chain anchor.
    pub async fn anchor_document_hash(&self, document_hash_hex: &str) -> Result<BlockchainAnchor> {
        let Some(client) = &self.client else {
            return Ok(BlockchainAnchor {
                transaction_hash: String::new(),
                block_number: 0,
                status: "skipped".to_string(),
            });
        };
        let from = self.from.context("missing from address")?;

        let data = hex::decode(document_hash_hex.trim_start_matches("0x"))
            .context("document_hash must be hex")?;

        let tx = TransactionRequest::new()
            .from(from)
            .to(from) // self-transfer (no value)
            .value(U256::zero())
            .data(Bytes::from(data));

        let pending = client
            .send_transaction(tx, None)
            .await
            .context("failed to send anchor tx")?;

        let tx_hash = format!("{:#x}", pending.tx_hash());

        let receipt = pending
            .confirmations(1)
            .await
            .context("failed waiting for tx receipt")?;

        let (block_number, status) = if let Some(rcpt) = receipt {
            (
                rcpt.block_number.map(|b| b.as_u64() as i64).unwrap_or(0),
                "confirmed".to_string(),
            )
        } else {
            (0, "pending".to_string())
        };

        Ok(BlockchainAnchor {
            transaction_hash: tx_hash,
            block_number,
            status,
        })
    }
}

