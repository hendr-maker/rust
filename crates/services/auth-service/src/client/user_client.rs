//! gRPC client for user-service.

use async_trait::async_trait;
use tonic::transport::Channel;
use tracing::debug;

use common::{AppError, AppResult};
use domain::User;
use proto::user::{
    user_service_client::UserServiceClient as ProtoUserServiceClient, CreateUserRequest,
    GetUserByEmailRequest, InternalUserResponse,
};

/// Trait for user operations needed by auth-service.
#[async_trait]
pub trait UserServiceClient: Send + Sync {
    /// Find user by email (excludes soft-deleted)
    async fn find_by_email(&self, email: &str) -> AppResult<Option<User>>;

    /// Find user by email including soft-deleted
    async fn find_by_email_with_deleted(&self, email: &str) -> AppResult<Option<User>>;

    /// Create a new user
    async fn create(&self, email: String, password_hash: String, name: String) -> AppResult<User>;
}

/// gRPC client wrapper for user-service.
pub struct UserClient {
    client: ProtoUserServiceClient<Channel>,
}

impl UserClient {
    /// Connect to user-service.
    pub async fn connect(endpoint: &str) -> Result<Self, tonic::transport::Error> {
        debug!("Connecting to user-service at {}", endpoint);
        let client = ProtoUserServiceClient::connect(endpoint.to_string()).await?;
        Ok(Self { client })
    }
}

#[async_trait]
impl UserServiceClient for UserClient {
    async fn find_by_email(&self, email: &str) -> AppResult<Option<User>> {
        let request = tonic::Request::new(GetUserByEmailRequest {
            email: email.to_string(),
        });

        let mut client = self.client.clone();
        // Use internal endpoint to get password hash for authentication
        match client.get_user_by_email_internal(request).await {
            Ok(response) => {
                let proto_user = response.into_inner();
                Ok(Some(internal_proto_to_user(proto_user)?))
            }
            Err(status) if status.code() == tonic::Code::NotFound => Ok(None),
            Err(status) => Err(AppError::from(status)),
        }
    }

    async fn find_by_email_with_deleted(&self, email: &str) -> AppResult<Option<User>> {
        let request = tonic::Request::new(GetUserByEmailRequest {
            email: email.to_string(),
        });

        let mut client = self.client.clone();
        // Use internal endpoint to get password hash for authentication
        match client.get_user_by_email_internal_with_deleted(request).await {
            Ok(response) => {
                let proto_user = response.into_inner();
                Ok(Some(internal_proto_to_user(proto_user)?))
            }
            Err(status) if status.code() == tonic::Code::NotFound => Ok(None),
            Err(status) => Err(AppError::from(status)),
        }
    }

    async fn create(&self, email: String, password_hash: String, name: String) -> AppResult<User> {
        let request = tonic::Request::new(CreateUserRequest {
            email,
            password_hash,
            name,
        });

        let mut client = self.client.clone();
        // Create returns public UserResponse, but we need to fetch internally for password
        let response = client.create_user(request).await.map_err(AppError::from)?;
        let proto = response.into_inner();

        // Create returns the user without password_hash in proto, but we just created it
        // so we can construct the User. For auth purposes, after create we typically login.
        let id = proto.id.parse()
            .map_err(|_| AppError::internal("Invalid UUID from user-service"))?;
        let created_at = chrono::DateTime::parse_from_rfc3339(&proto.created_at)
            .map_err(|_| AppError::internal("Invalid created_at from user-service"))?
            .with_timezone(&chrono::Utc);
        let updated_at = chrono::DateTime::parse_from_rfc3339(&proto.updated_at)
            .map_err(|_| AppError::internal("Invalid updated_at from user-service"))?
            .with_timezone(&chrono::Utc);

        Ok(User {
            id,
            email: proto.email,
            password_hash: String::new(), // Not returned from create for security
            name: proto.name,
            role: domain::UserRole::from(proto.role),
            created_at,
            updated_at,
            deleted_at: None,
        })
    }
}

/// Convert proto InternalUserResponse to domain User (includes password hash).
fn internal_proto_to_user(proto: InternalUserResponse) -> AppResult<User> {
    let id = proto
        .id
        .parse()
        .map_err(|_| AppError::internal("Invalid UUID from user-service"))?;

    let created_at = chrono::DateTime::parse_from_rfc3339(&proto.created_at)
        .map_err(|_| AppError::internal("Invalid created_at from user-service"))?
        .with_timezone(&chrono::Utc);

    let updated_at = chrono::DateTime::parse_from_rfc3339(&proto.updated_at)
        .map_err(|_| AppError::internal("Invalid updated_at from user-service"))?
        .with_timezone(&chrono::Utc);

    let deleted_at = proto
        .deleted_at
        .map(|dt| {
            chrono::DateTime::parse_from_rfc3339(&dt)
                .map(|d| d.with_timezone(&chrono::Utc))
                .map_err(|_| AppError::internal("Invalid deleted_at from user-service"))
        })
        .transpose()?;

    Ok(User {
        id,
        email: proto.email,
        password_hash: proto.password_hash,
        name: proto.name,
        role: domain::UserRole::from(proto.role),
        created_at,
        updated_at,
        deleted_at,
    })
}
