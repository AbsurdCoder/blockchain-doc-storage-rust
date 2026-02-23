use anyhow::{anyhow, Context, Result};
use bcrypt::{hash, verify, DEFAULT_COST};
use chrono::{Duration, Utc};
use jsonwebtoken::{encode, EncodingKey, Header};
use sqlx::mysql::MySqlQueryResult;

use crate::models::{AuthResponse, Claims, LoginRequest, RegisterRequest, User, UserResponse};

pub struct AuthService;

impl AuthService {
    pub async fn register(
        db: &sqlx::MySqlPool,
        jwt_secret: &str,
        req: RegisterRequest,
    ) -> Result<AuthResponse> {
        let password_hash = hash(req.password, DEFAULT_COST).context("failed to hash password")?;

        // Basic uniqueness check
        let existing = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM users WHERE email = ?")
            .bind(&req.email)
            .fetch_one(db)
            .await
            .context("failed to query existing user")?;
        if existing > 0 {
            return Err(anyhow!("email already registered"));
        }

        let res: MySqlQueryResult = sqlx::query(
            r#"
            INSERT INTO users (open_id, name, email, password_hash, login_method, role, last_signed_in)
            VALUES ('', ?, ?, ?, 'password', 'user', NOW())
            "#,
        )
        .bind(req.name)
        .bind(&req.email)
        .bind(password_hash)
        .execute(db)
        .await
        .context("failed to insert user")?;

        let user_id = res.last_insert_id() as i32;

        let user: User = sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = ?")
            .bind(user_id)
            .fetch_one(db)
            .await
            .context("failed to load created user")?;

        let token = Self::issue_jwt(jwt_secret, &user)?;

        Ok(AuthResponse {
            user: UserResponse::from(user),
            token,
        })
    }

    pub async fn login(db: &sqlx::MySqlPool, jwt_secret: &str, req: LoginRequest) -> Result<AuthResponse> {
        let user: User = sqlx::query_as::<_, User>("SELECT * FROM users WHERE email = ?")
            .bind(&req.email)
            .fetch_one(db)
            .await
            .context("invalid email or password")?;

        let Some(ph) = user.password_hash.clone() else {
            return Err(anyhow!("invalid email or password"));
        };

        let ok = verify(req.password, &ph).context("failed to verify password")?;
        if !ok {
            return Err(anyhow!("invalid email or password"));
        }

        sqlx::query("UPDATE users SET last_signed_in = NOW() WHERE id = ?")
            .bind(user.id)
            .execute(db)
            .await
            .ok();

        let token = Self::issue_jwt(jwt_secret, &user)?;
        Ok(AuthResponse {
            user: UserResponse::from(user),
            token,
        })
    }

    pub async fn get_user_by_id(db: &sqlx::MySqlPool, user_id: i32) -> Result<User> {
        let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = ?")
            .bind(user_id)
            .fetch_one(db)
            .await
            .context("user not found")?;
        Ok(user)
    }

    fn issue_jwt(jwt_secret: &str, user: &User) -> Result<String> {
        let exp = (Utc::now() + Duration::hours(24)).timestamp() as usize;
        let claims = Claims {
            sub: user.id.to_string(),
            email: user.email.clone().unwrap_or_default(),
            role: user.role.clone(),
            exp,
        };
        let token = encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(jwt_secret.as_bytes()),
        )
        .context("failed to encode jwt")?;
        Ok(token)
    }
}

