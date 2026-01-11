//! gRPC client for auth-service.

use tonic::transport::Channel;
use tracing::debug;

use common::{AppError, AppResult};
use domain::UserResponse;
use proto::auth::{
    auth_service_client::AuthServiceClient as ProtoAuthServiceClient, LoginRequest,
    RegisterRequest, VerifyTokenRequest,
};

/// Token response from auth-service.
#[derive(Debug, Clone, serde::Serialize, utoipa::ToSchema)]
pub struct TokenResponse {
    pub access_token: String,
    pub token_type: String,
    pub expires_in: i64,
}

/// Verified token claims.
#[derive(Debug, Clone)]
pub struct Claims {
    pub user_id: uuid::Uuid,
    pub email: String,
    pub role: String,
}

/// gRPC client wrapper for auth-service.
pub struct AuthClient {
    client: ProtoAuthServiceClient<Channel>,
}

impl AuthClient {
    /// Connect to auth-service.
    pub async fn connect(endpoint: &str) -> Result<Self, tonic::transport::Error> {
        debug!("Connecting to auth-service at {}", endpoint);
        let client = ProtoAuthServiceClient::connect(endpoint.to_string()).await?;
        Ok(Self { client })
    }

    /// Register a new user.
    pub async fn register(
        &self,
        email: String,
        password: String,
        name: String,
    ) -> AppResult<UserResponse> {
        let request = tonic::Request::new(RegisterRequest {
            email,
            password,
            name,
        });

        let mut client = self.client.clone();
        let response = client.register(request).await.map_err(AppError::from)?;
        let proto = response.into_inner();

        Ok(UserResponse {
            id: proto.id.parse().map_err(|_| AppError::internal("Invalid UUID"))?,
            email: proto.email,
            name: proto.name,
            role: proto.role,
            created_at: chrono::DateTime::parse_from_rfc3339(&proto.created_at)
                .map_err(|_| AppError::internal("Invalid date"))?
                .with_timezone(&chrono::Utc),
            deleted_at: None,
        })
    }

    /// Login and get token.
    pub async fn login(&self, email: String, password: String) -> AppResult<TokenResponse> {
        let request = tonic::Request::new(LoginRequest { email, password });

        let mut client = self.client.clone();
        let response = client.login(request).await.map_err(AppError::from)?;
        let proto = response.into_inner();

        Ok(TokenResponse {
            access_token: proto.access_token,
            token_type: proto.token_type,
            expires_in: proto.expires_in,
        })
    }

    /// Verify a JWT token.
    pub async fn verify_token(&self, token: &str) -> AppResult<Option<Claims>> {
        let request = tonic::Request::new(VerifyTokenRequest {
            token: token.to_string(),
        });

        let mut client = self.client.clone();
        let response = client.verify_token(request).await.map_err(AppError::from)?;
        let proto = response.into_inner();

        if !proto.valid {
            return Ok(None);
        }

        Ok(Some(Claims {
            user_id: proto
                .user_id
                .parse()
                .map_err(|_| AppError::internal("Invalid UUID"))?,
            email: proto.email,
            role: proto.role,
        }))
    }
}
