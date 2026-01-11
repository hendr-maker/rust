//! gRPC implementation for AuthService.

use std::sync::Arc;

use tonic::{Request, Response, Status};

use crate::service::AuthService;
use proto::auth::{
    auth_service_server::AuthService as AuthServiceProto, LoginRequest, LoginResponse,
    RefreshTokenRequest, RegisterRequest, RegisterResponse, VerifyTokenRequest,
    VerifyTokenResponse,
};

/// gRPC service wrapper for AuthService.
pub struct AuthGrpcService {
    service: Arc<dyn AuthService>,
}

impl AuthGrpcService {
    /// Create a new gRPC service wrapper.
    pub fn new(service: Arc<dyn AuthService>) -> Self {
        Self { service }
    }
}

#[tonic::async_trait]
impl AuthServiceProto for AuthGrpcService {
    async fn register(
        &self,
        request: Request<RegisterRequest>,
    ) -> Result<Response<RegisterResponse>, Status> {
        let req = request.into_inner();

        let user = self
            .service
            .register(req.email, req.password, req.name)
            .await
            .map_err(Status::from)?;

        Ok(Response::new(RegisterResponse {
            id: user.id.to_string(),
            email: user.email,
            name: user.name,
            role: user.role.to_string(),
            created_at: user.created_at.to_rfc3339(),
        }))
    }

    async fn login(
        &self,
        request: Request<LoginRequest>,
    ) -> Result<Response<LoginResponse>, Status> {
        let req = request.into_inner();

        let token = self
            .service
            .login(req.email, req.password)
            .await
            .map_err(Status::from)?;

        Ok(Response::new(LoginResponse {
            access_token: token.access_token,
            token_type: token.token_type,
            expires_in: token.expires_in,
        }))
    }

    async fn verify_token(
        &self,
        request: Request<VerifyTokenRequest>,
    ) -> Result<Response<VerifyTokenResponse>, Status> {
        let req = request.into_inner();

        match self.service.verify_token(&req.token) {
            Ok(claims) => Ok(Response::new(VerifyTokenResponse {
                user_id: claims.sub.to_string(),
                email: claims.email,
                role: claims.role,
                valid: true,
            })),
            Err(_) => Ok(Response::new(VerifyTokenResponse {
                user_id: String::new(),
                email: String::new(),
                role: String::new(),
                valid: false,
            })),
        }
    }

    async fn refresh_token(
        &self,
        request: Request<RefreshTokenRequest>,
    ) -> Result<Response<LoginResponse>, Status> {
        let req = request.into_inner();

        // Verify the existing token first
        let claims = self
            .service
            .verify_token(&req.token)
            .map_err(Status::from)?;

        // Generate a new token with fresh user data
        let token = self
            .service
            .refresh_token(&claims)
            .await
            .map_err(Status::from)?;

        Ok(Response::new(LoginResponse {
            access_token: token.access_token,
            token_type: token.token_type,
            expires_in: token.expires_in,
        }))
    }
}

