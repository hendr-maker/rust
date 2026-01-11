//! gRPC implementation for UserService.

use std::sync::Arc;

use tonic::{Request, Response, Status};
use uuid::Uuid;

use crate::service::UserService;
use proto::user::{
    user_service_server::UserService as UserServiceProto, CreateUserRequest, DeleteUserRequest,
    DeleteUserResponse, GetUserByEmailRequest, GetUserRequest, InternalUserResponse,
    ListUsersRequest, ListUsersResponse, RestoreUserRequest, UpdateUserRequest, UserResponse,
};

/// gRPC service wrapper for UserService.
pub struct UserGrpcService {
    service: Arc<dyn UserService>,
}

impl UserGrpcService {
    /// Create a new gRPC service wrapper.
    pub fn new(service: Arc<dyn UserService>) -> Self {
        Self { service }
    }
}

#[tonic::async_trait]
impl UserServiceProto for UserGrpcService {
    async fn get_user(
        &self,
        request: Request<GetUserRequest>,
    ) -> Result<Response<UserResponse>, Status> {
        let req = request.into_inner();
        let id = parse_uuid(&req.id)?;

        let user = self.service.get_user(id).await.map_err(Status::from)?;
        Ok(Response::new(user_to_proto(&user)))
    }

    async fn get_user_with_deleted(
        &self,
        request: Request<GetUserRequest>,
    ) -> Result<Response<UserResponse>, Status> {
        let req = request.into_inner();
        let id = parse_uuid(&req.id)?;

        let user = self
            .service
            .get_user_with_deleted(id)
            .await
            .map_err(Status::from)?;
        Ok(Response::new(user_to_proto(&user)))
    }

    async fn get_user_by_email(
        &self,
        request: Request<GetUserByEmailRequest>,
    ) -> Result<Response<UserResponse>, Status> {
        let req = request.into_inner();

        let user = self
            .service
            .get_user_by_email(&req.email)
            .await
            .map_err(Status::from)?;
        Ok(Response::new(user_to_proto(&user)))
    }

    async fn get_user_by_email_with_deleted(
        &self,
        request: Request<GetUserByEmailRequest>,
    ) -> Result<Response<UserResponse>, Status> {
        let req = request.into_inner();

        let user = self
            .service
            .get_user_by_email_with_deleted(&req.email)
            .await
            .map_err(Status::from)?;
        Ok(Response::new(user_to_proto(&user)))
    }

    async fn list_users(
        &self,
        _request: Request<ListUsersRequest>,
    ) -> Result<Response<ListUsersResponse>, Status> {
        let users = self.service.list_users().await.map_err(Status::from)?;
        let total = users.len() as i32;
        let users: Vec<UserResponse> = users.iter().map(user_to_proto).collect();

        Ok(Response::new(ListUsersResponse { users, total }))
    }

    async fn list_users_with_deleted(
        &self,
        _request: Request<ListUsersRequest>,
    ) -> Result<Response<ListUsersResponse>, Status> {
        let users = self
            .service
            .list_users_with_deleted()
            .await
            .map_err(Status::from)?;
        let total = users.len() as i32;
        let users: Vec<UserResponse> = users.iter().map(user_to_proto).collect();

        Ok(Response::new(ListUsersResponse { users, total }))
    }

    async fn list_deleted_users(
        &self,
        _request: Request<ListUsersRequest>,
    ) -> Result<Response<ListUsersResponse>, Status> {
        let users = self
            .service
            .list_deleted_users()
            .await
            .map_err(Status::from)?;
        let total = users.len() as i32;
        let users: Vec<UserResponse> = users.iter().map(user_to_proto).collect();

        Ok(Response::new(ListUsersResponse { users, total }))
    }

    async fn create_user(
        &self,
        request: Request<CreateUserRequest>,
    ) -> Result<Response<UserResponse>, Status> {
        let req = request.into_inner();

        let user = self
            .service
            .create_user(req.email, req.password_hash, req.name)
            .await
            .map_err(Status::from)?;
        Ok(Response::new(user_to_proto(&user)))
    }

    async fn update_user(
        &self,
        request: Request<UpdateUserRequest>,
    ) -> Result<Response<UserResponse>, Status> {
        let req = request.into_inner();
        let id = parse_uuid(&req.id)?;

        let user = self
            .service
            .update_user(id, req.name, req.role)
            .await
            .map_err(Status::from)?;
        Ok(Response::new(user_to_proto(&user)))
    }

    async fn delete_user(
        &self,
        request: Request<DeleteUserRequest>,
    ) -> Result<Response<DeleteUserResponse>, Status> {
        let req = request.into_inner();
        let id = parse_uuid(&req.id)?;

        self.service.delete_user(id).await.map_err(Status::from)?;
        Ok(Response::new(DeleteUserResponse { success: true }))
    }

    async fn hard_delete_user(
        &self,
        request: Request<DeleteUserRequest>,
    ) -> Result<Response<DeleteUserResponse>, Status> {
        let req = request.into_inner();
        let id = parse_uuid(&req.id)?;

        self.service.hard_delete_user(id).await.map_err(Status::from)?;
        Ok(Response::new(DeleteUserResponse { success: true }))
    }

    async fn restore_user(
        &self,
        request: Request<RestoreUserRequest>,
    ) -> Result<Response<UserResponse>, Status> {
        let req = request.into_inner();
        let id = parse_uuid(&req.id)?;

        let user = self.service.restore_user(id).await.map_err(Status::from)?;
        Ok(Response::new(user_to_proto(&user)))
    }

    async fn get_user_by_email_internal(
        &self,
        request: Request<GetUserByEmailRequest>,
    ) -> Result<Response<InternalUserResponse>, Status> {
        let req = request.into_inner();

        let user = self
            .service
            .get_user_by_email(&req.email)
            .await
            .map_err(Status::from)?;
        Ok(Response::new(internal_user_to_proto(&user)))
    }

    async fn get_user_by_email_internal_with_deleted(
        &self,
        request: Request<GetUserByEmailRequest>,
    ) -> Result<Response<InternalUserResponse>, Status> {
        let req = request.into_inner();

        let user = self
            .service
            .get_user_by_email_with_deleted(&req.email)
            .await
            .map_err(Status::from)?;
        Ok(Response::new(internal_user_to_proto(&user)))
    }
}

/// Parse UUID from string.
fn parse_uuid(s: &str) -> Result<Uuid, Status> {
    Uuid::parse_str(s).map_err(|_| Status::invalid_argument("Invalid UUID format"))
}

/// Convert domain User to proto UserResponse (public - no password hash).
fn user_to_proto(user: &domain::User) -> UserResponse {
    UserResponse {
        id: user.id.to_string(),
        email: user.email.clone(),
        name: user.name.clone(),
        role: user.role.to_string(),
        created_at: user.created_at.to_rfc3339(),
        updated_at: user.updated_at.to_rfc3339(),
        deleted_at: user.deleted_at.map(|dt| dt.to_rfc3339()),
    }
}

/// Convert domain User to proto InternalUserResponse (includes password hash).
fn internal_user_to_proto(user: &domain::User) -> InternalUserResponse {
    InternalUserResponse {
        id: user.id.to_string(),
        email: user.email.clone(),
        name: user.name.clone(),
        role: user.role.to_string(),
        password_hash: user.password_hash.clone(),
        created_at: user.created_at.to_rfc3339(),
        updated_at: user.updated_at.to_rfc3339(),
        deleted_at: user.deleted_at.map(|dt| dt.to_rfc3339()),
    }
}

