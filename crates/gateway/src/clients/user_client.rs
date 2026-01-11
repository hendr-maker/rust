//! gRPC client for user-service.

use tonic::transport::Channel;
use tracing::debug;

use common::{AppError, AppResult};
use domain::{User, UserRole};
use proto::user::{
    user_service_client::UserServiceClient as ProtoUserServiceClient, DeleteUserRequest,
    GetUserRequest, ListUsersRequest, RestoreUserRequest, UpdateUserRequest,
};

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

    /// Get user by ID.
    pub async fn get_user(&self, id: uuid::Uuid) -> AppResult<User> {
        let request = tonic::Request::new(GetUserRequest { id: id.to_string() });

        let mut client = self.client.clone();
        let response = client.get_user(request).await.map_err(AppError::from)?;
        proto_to_user(response.into_inner())
    }

    /// List all users.
    pub async fn list_users(&self) -> AppResult<Vec<User>> {
        let request = tonic::Request::new(ListUsersRequest {
            limit: None,
            offset: None,
        });

        let mut client = self.client.clone();
        let response = client.list_users(request).await.map_err(AppError::from)?;
        let proto = response.into_inner();

        proto
            .users
            .into_iter()
            .map(proto_to_user)
            .collect()
    }

    /// Update user.
    pub async fn update_user(
        &self,
        id: uuid::Uuid,
        name: Option<String>,
        role: Option<String>,
    ) -> AppResult<User> {
        let request = tonic::Request::new(UpdateUserRequest {
            id: id.to_string(),
            name,
            role,
        });

        let mut client = self.client.clone();
        let response = client.update_user(request).await.map_err(AppError::from)?;
        proto_to_user(response.into_inner())
    }

    /// Soft delete user.
    pub async fn delete_user(&self, id: uuid::Uuid) -> AppResult<()> {
        let request = tonic::Request::new(DeleteUserRequest { id: id.to_string() });

        let mut client = self.client.clone();
        client.delete_user(request).await.map_err(AppError::from)?;
        Ok(())
    }

    /// Restore soft-deleted user.
    pub async fn restore_user(&self, id: uuid::Uuid) -> AppResult<User> {
        let request = tonic::Request::new(RestoreUserRequest { id: id.to_string() });

        let mut client = self.client.clone();
        let response = client.restore_user(request).await.map_err(AppError::from)?;
        proto_to_user(response.into_inner())
    }
}

/// Convert proto UserResponse to domain User.
fn proto_to_user(proto: proto::user::UserResponse) -> AppResult<User> {
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
        password_hash: String::new(), // Gateway doesn't receive password hash (security)
        name: proto.name,
        role: UserRole::from(proto.role),
        created_at,
        updated_at,
        deleted_at,
    })
}
