# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

**rust-api-starter** - A production-ready Rust web API built with Axum, following Domain-Driven Design, Clean Architecture, SOLID, and DRY principles.

**Architecture Type:** Microservices (with gRPC inter-service communication)

## Tech Stack

- **Axum 0.7** - Web framework (API Gateway)
- **Tonic 0.12** - gRPC framework for inter-service communication
- **SeaORM 1.0** - Async ORM for PostgreSQL
- **Redis** - Caching, rate limiting, distributed locks, semaphores
- **JWT** - Authentication (jsonwebtoken + Argon2)
- **Tokio** - Async runtime
- **Docker** - Containerization
- **Mockall** - Testing with mocks

## CLI Commands

```bash
# Combined binary (development mode - all services in one process)
cargo run -p combined -- serve
cargo run -p combined -- serve --gateway-port 3000 --auth-port 50051 --user-port 50052

# Database migrations (via combined binary)
cargo run -p combined -- migrate up       # Run pending migrations
cargo run -p combined -- migrate down     # Rollback last migration
cargo run -p combined -- migrate status   # Check migration status
cargo run -p combined -- migrate fresh    # Reset and re-run all

# Individual service binaries (production mode)
cargo run -p gateway -- serve --port 3000
cargo run -p auth-service -- serve --port 50051
cargo run -p user-service -- serve --port 50052

# User service migrations (owns the database)
cargo run -p user-service -- migrate up
cargo run -p user-service -- migrate status
```

## Development Commands

```bash
cargo build                      # Build project
cargo fmt && cargo clippy        # Format and lint
cargo test                       # All tests
cargo test --lib                 # Unit tests only
cargo test --test api_test       # Integration tests
```

## Docker Commands

```bash
# Development mode (combined binary - simpler for local dev)
docker-compose -f docker-compose.dev.yml up -d   # Start combined + PostgreSQL + Redis
docker-compose -f docker-compose.dev.yml down    # Stop containers
docker-compose -f docker-compose.dev.yml logs -f # View logs

# Production mode (separate microservices)
docker-compose up -d             # Start gateway + auth + user + PostgreSQL + Redis
docker-compose down              # Stop containers
docker-compose logs -f           # View logs
docker-compose exec redis redis-cli  # Redis CLI
```

## Microservice Architecture

```
                    ┌─────────────────┐
                    │  Load Balancer  │  (nginx/Traefik/AWS ALB)
                    └────────┬────────┘
                             │
                    ┌────────▼────────┐
                    │    Gateway      │  HTTP REST API (:3000)
                    │   (Axum)        │
                    └────────┬────────┘
                             │ gRPC
              ┌──────────────┼──────────────┐
              │              │              │
     ┌────────▼────────┐     │     ┌────────▼────────┐
     │  Auth Service   │     │     │  User Service   │
     │  (Tonic gRPC)   │     │     │  (Tonic gRPC)   │
     │     :50051      │     │     │     :50052      │
     └────────┬────────┘     │     └────────┬────────┘
              │              │              │
              └──────────────┼──────────────┘
                             │
              ┌──────────────┼──────────────┐
              │              │              │
     ┌────────▼────────┐     │     ┌────────▼────────┐
     │     Redis       │     │     │   PostgreSQL    │
     │ (sessions/cache)│     │     │   (user data)   │
     └─────────────────┘     │     └─────────────────┘
```

**Service Responsibilities:**

| Service | Port | Protocol | Database | Responsibilities |
|---------|------|----------|----------|------------------|
| gateway | 3000 | HTTP | Redis | REST API, auth middleware, rate limiting, OpenAPI |
| auth-service | 50051 | gRPC | Redis | JWT, login, register, token verification |
| user-service | 50052 | gRPC | PostgreSQL | User CRUD, soft delete, migrations |

**Deployment Modes:**

| Mode | Binary | Use Case |
|------|--------|----------|
| Development | `combined` (rust-api) | Local development, single-process debugging |
| Production | Separate binaries | Kubernetes/Docker Swarm, independent scaling |

## Workspace Structure

```
rust-api-starter/
├── Cargo.toml                      # Workspace root
├── docker-compose.yml              # Production (separate services)
├── docker-compose.dev.yml          # Development (combined binary)
├── Dockerfile                      # Combined binary
├── Dockerfile.gateway              # Gateway service
├── Dockerfile.auth                 # Auth service
├── Dockerfile.user                 # User service
│
├── crates/
│   ├── shared/
│   │   ├── domain/                 # User entity, Password VO, DTOs
│   │   ├── proto/                  # gRPC protobuf definitions
│   │   └── common/                 # AppError, AppResult, configs
│   │
│   ├── services/
│   │   ├── auth-service/           # Auth microservice
│   │   │   └── src/{main.rs, lib.rs, grpc/, service/, client/}
│   │   └── user-service/           # User microservice
│   │       └── src/{main.rs, lib.rs, grpc/, service/, repository/, infra/}
│   │
│   └── gateway/                    # API Gateway
│       └── src/{main.rs, lib.rs, handlers/, middleware/, clients/}
│
├── combined/                       # Combined binary for development
│   └── src/main.rs
│
└── src/                            # (Legacy - original monolith code)
```

## Health Check Endpoint

The `/health` endpoint verifies Redis connectivity to ensure the gateway can function properly.

**Request:**
```bash
curl http://localhost:3000/health
```

**Response (200 OK - All healthy):**
```json
{
  "status": "healthy",
  "services": {
    "redis": {
      "status": "healthy"
    }
  }
}
```

**Response (503 Service Unavailable - Degraded):**
```json
{
  "status": "degraded",
  "services": {
    "redis": {
      "status": "unhealthy",
      "error": "Cache error: Connection refused"
    }
  }
}
```

**Status Codes:**

| Code | Meaning |
|------|---------|
| 200 | All services healthy |
| 503 | One or more services unhealthy |

**Use Cases:**
- Kubernetes liveness/readiness probes
- Load balancer health checks
- Monitoring and alerting
- Docker Compose health checks

## OpenAPI Documentation

Interactive API documentation available via Swagger UI at `/swagger-ui`.

**Endpoints:**
- `/swagger-ui` - Interactive Swagger UI
- `/api-docs/openapi.json` - OpenAPI 3.0 JSON specification

**Features:**
- Complete API documentation with request/response schemas
- JWT Bearer authentication support
- Try-it-out functionality for testing endpoints
- Request validation documentation

**Documented Endpoints:**

| Tag | Endpoint | Description |
|-----|----------|-------------|
| Authentication | `POST /auth/register` | Register new user |
| Authentication | `POST /auth/login` | Login and get JWT token |
| Users | `GET /users/me` | Get current user profile |
| Users | `GET /users` | List all users (admin) |
| Users | `GET /users/{id}` | Get user by ID |
| Users | `PUT /users/{id}` | Update user |
| Users | `DELETE /users/{id}` | Soft delete user (admin) |
| Users | `POST /users/{id}/restore` | Restore soft-deleted user (admin) |

**Adding New Endpoints:**

1. Add `ToSchema` derive to request/response types:
```rust
use utoipa::ToSchema;

#[derive(ToSchema)]
pub struct MyRequest {
    #[schema(example = "value")]
    pub field: String,
}
```

2. Add `#[utoipa::path]` annotation to handler:
```rust
#[utoipa::path(
    post,
    path = "/my/endpoint",
    tag = "MyTag",
    request_body = MyRequest,
    responses(
        (status = 200, description = "Success", body = MyResponse),
        (status = 400, description = "Validation error")
    )
)]
pub async fn my_handler(...) { ... }
```

3. Register in `api/openapi.rs`:
```rust
#[openapi(
    paths(
        my_handler::my_endpoint,
    ),
    components(
        schemas(MyRequest, MyResponse)
    )
)]
pub struct ApiDoc;
```

## Architecture (Clean Architecture + DDD)

Each microservice follows Clean Architecture internally:

```
crates/services/user-service/     # Example microservice
├── Cargo.toml
└── src/
    ├── main.rs                   # CLI entry point (serve, migrate)
    ├── lib.rs                    # Library root, exports
    ├── config.rs                 # Service-specific config
    │
    ├── grpc/                     # gRPC layer (Presentation)
    │   ├── mod.rs
    │   └── user_grpc.rs          # gRPC handlers
    │
    ├── service/                  # Application layer
    │   ├── mod.rs
    │   └── user_service.rs       # Business logic + trait
    │
    ├── repository/               # Data access layer
    │   ├── mod.rs
    │   ├── user_repository.rs    # Repository trait + impl
    │   └── entities/             # SeaORM entities
    │
    └── infra/                    # Infrastructure layer
        ├── mod.rs
        ├── db.rs                 # Database connection
        └── migrations/           # SeaORM migrations

crates/gateway/                   # API Gateway
└── src/
    ├── main.rs                   # CLI entry point
    ├── lib.rs
    ├── config.rs
    ├── state.rs                  # AppState (DI container)
    ├── routes.rs                 # Route configuration
    │
    ├── clients/                  # gRPC clients
    │   ├── mod.rs
    │   ├── auth_client.rs        # Calls auth-service
    │   └── user_client.rs        # Calls user-service
    │
    ├── handlers/                 # HTTP handlers
    │   ├── mod.rs
    │   ├── auth_handler.rs
    │   └── user_handler.rs
    │
    ├── middleware/               # HTTP middleware
    │   ├── mod.rs
    │   ├── auth.rs               # JWT validation
    │   ├── rate_limit.rs         # Rate limiting
    │   └── cache.rs              # Response caching
    │
    ├── extractors/               # Custom extractors
    │   └── validated_json.rs     # Auto-validation
    │
    └── openapi.rs                # Swagger/OpenAPI setup

crates/shared/                    # Shared crates
├── domain/                       # Domain entities (no dependencies)
│   └── src/{user.rs, password.rs, error.rs}
├── common/                       # Shared utilities
│   └── src/{error.rs, config.rs}
└── proto/                        # gRPC definitions
    ├── proto/{auth.proto, user.proto}
    └── build.rs                  # Tonic code generation
```

## Layer Dependencies (Clean Architecture)

**Within each microservice:**
```
┌─────────────────────────────────────────┐
│              main.rs                     │  Entry Point
├─────────────────────────────────────────┤
│              grpc/                       │  Presentation (gRPC)
│  (handlers, proto conversion)            │
├─────────────────────────────────────────┤
│             service/                     │  Application
│  (business logic, use cases)             │
├─────────────────────────────────────────┤
│            repository/                   │  Data Access
│  (trait + impl, entities)                │
├─────────────────────────────────────────┤
│              infra/                      │  Infrastructure
│  (db connection, migrations)             │
└─────────────────────────────────────────┘
```

**Shared crates:**
```
┌─────────────────────────────────────────┐
│ domain (shared entities, DTOs)           │  Pure domain, no deps
├─────────────────────────────────────────┤
│ common (errors, config)                  │  Utilities
├─────────────────────────────────────────┤
│ proto (gRPC definitions)                 │  Generated code
└─────────────────────────────────────────┘
```

Dependencies flow INWARD (outer layers depend on inner)
Domain layer has NO external dependencies

## SOLID Principles

**S - Single Responsibility:** Each service/module has one purpose
- `user-service` - User CRUD only
- `auth-service` - Authentication only
- `gateway` - HTTP routing/validation only

**O - Open/Closed:** Extend via new implementations
- Add new error variants without changing `AppError`
- Add new services implementing existing traits
- Add new gRPC methods without modifying existing ones

**L - Liskov Substitution:** All implementations interchangeable
- `MockUserRepository` can replace `UserStore`

**I - Interface Segregation:** Small, focused traits
- `UserService` trait separate from `UserRepository` trait

**D - Dependency Inversion:** Depend on abstractions
- Services depend on `dyn UserRepository`, not concrete `UserStore`
- Gateway depends on gRPC client traits, not service implementations

## DRY Patterns

**Shared Domain:** `User`, `UserRole`, `UserResponse` in `crates/shared/domain/`

**Shared Errors:** `AppError`, `AppResult<T>` in `crates/shared/common/`

**Proto Definitions:** gRPC types defined once in `crates/shared/proto/`

**Extractors:** `ValidatedJson<T>` auto-validates HTTP requests

**Errors:** `AppError::conflict()`, `option.ok_or_not_found()?`

## Adding New Features

**New Domain Entity:**
1. Create in `crates/shared/domain/src/` with entity, DTOs
2. Export in `crates/shared/domain/src/lib.rs`
3. Add `#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]` for OpenAPI

**New gRPC Method:**
1. Add to proto file in `crates/shared/proto/proto/`
2. Run `cargo build -p proto` to regenerate
3. Implement in service's `grpc/` module
4. Add client method in `crates/gateway/src/clients/`

**New Repository (in a service):**
1. Create entity in `repository/entities/`
2. Create trait + impl in `repository/`
3. Add `#[cfg_attr(test, automock)]` for testing

**New HTTP Endpoint (in gateway):**
1. Create handler in `crates/gateway/src/handlers/`
2. Add routes in handler module
3. Register in `crates/gateway/src/routes.rs`
4. Add OpenAPI documentation with `#[utoipa::path]`

**New Microservice:**
See "Adding New Microservices" section below

## Environment Variables

```bash
# Database (user-service)
DATABASE_URL=postgres://user:pass@localhost:5432/user_db

# Redis (auth-service, gateway)
REDIS_URL=redis://127.0.0.1:6379

# JWT (auth-service)
JWT_SECRET=your-secret-key-minimum-32-characters  # Required, min 32 chars
JWT_EXPIRATION_HOURS=24

# Gateway HTTP server
SERVER_HOST=0.0.0.0
SERVER_PORT=3000

# gRPC service addresses (gateway needs these)
AUTH_SERVICE_URL=http://auth-service:50051
USER_SERVICE_URL=http://user-service:50052

# Rate limiting (gateway)
RATE_LIMIT_REQUESTS=100
RATE_LIMIT_WINDOW_SECONDS=60
RATE_LIMIT_AUTH_REQUESTS=10
RATE_LIMIT_AUTH_WINDOW_SECONDS=60
```

## Security Features

- **JWT Secret**: Required in production, minimum 32 characters
- **Password Hashing**: Argon2 with random salt per password
- **Password Hash Isolation**: Password hashes only transmitted via internal gRPC methods (`GetUserByEmailInternal`), never exposed to gateway or external clients
- **Authorization**: Role-based access control (user/admin)
- **Input Validation**: All requests validated with validator crate
- **Sensitive Data**: Passwords and secrets never logged or exposed
- **Rate Limiting**: Fail-closed design - denies requests when Redis unavailable
- **Timing Attack Protection**: Constant-time password verification prevents email enumeration
- **Atomic Operations**: Semaphores use Lua scripts to prevent race conditions
- **Fresh Token Data**: Token refresh uses current user data, ensuring role changes are reflected immediately

## Security Hardening

The codebase implements several security best practices:

**Password Hash Isolation**
```protobuf
// Public response - NO password hash (used by gateway, external clients)
message UserResponse {
    string id = 1;
    string email = 2;
    string name = 3;
    string role = 4;
    string created_at = 5;
    string updated_at = 6;
    optional string deleted_at = 7;
}

// Internal response - includes password hash (auth-service only)
message InternalUserResponse {
    // ... same fields plus:
    string password_hash = 5;  // Only for auth-service
}
```

**Rate Limiting (Fail-Closed)**
```rust
// When Redis is unavailable, requests are DENIED (not allowed)
// This prevents attackers from bypassing rate limits by overwhelming Redis
let (count, allowed) = match cache.check_rate_limit(...).await {
    Ok(result) => result,
    Err(_) => return Err(RateLimitError { retry_after: 60 });
};
```

**Timing-Safe Authentication**
```rust
// Password verification always runs, even if user doesn't exist
// This prevents attackers from enumerating valid emails via response timing
let dummy_hash = "$argon2id$v=19$m=19456,t=2,p=1$...";
let password_hash = user.map(|u| u.password_hash).unwrap_or(dummy_hash);
let _ = Password::from_hash(password_hash).verify(&password);
```

**Fresh Token Refresh**
```rust
// Token refresh fetches current user data, not stale claims
// Ensures role changes are immediately reflected in new tokens
async fn refresh_token(&self, claims: &Claims) -> AppResult<TokenResponse> {
    let user = self.user_client.find_by_email(&claims.email).await?
        .ok_or(AppError::Unauthorized)?;
    self.generate_token(&user)  // Fresh user data, not old claims
}
```

**Atomic Semaphore Operations**
```rust
// Lua script ensures check-and-acquire is atomic
// Prevents race conditions that could exceed permit limits
let script = r#"
    local current = redis.call("SCARD", KEYS[1])
    if current < tonumber(ARGV[1]) then
        local added = redis.call("SADD", KEYS[1], ARGV[2])
        if added == 1 then
            redis.call("EXPIRE", KEYS[1], ARGV[3])
            return current + 1
        end
    end
    return -1
"#;
```

**Security Checklist:**
- [ ] Set strong `JWT_SECRET` (min 32 chars) in production
- [ ] Configure HTTPS/TLS termination at load balancer
- [ ] Set appropriate CORS headers for your domain
- [ ] Review rate limit thresholds for your traffic patterns
- [ ] Enable Redis authentication in production
- [ ] Use secrets management (not env vars) for sensitive data

## Service Container (Optional Pattern)

For complex services that need multiple dependencies, a Service Container pattern provides centralized service access with parallel execution utilities. This pattern can be used within individual microservices if needed.

**Features:**
- Centralized access to all application services
- Thread-safe concurrent access via `Arc`
- Parallel execution utilities for independent operations
- Batch processing for bulk operations
- Pipeline pattern for chaining operations

**Basic Usage:**
```rust
// Create via AppState (recommended)
let app_state = AppState::from_config(db, cache, config);

// Access services
let user = app_state.services().users().get_user(id).await?;
let token = app_state.services().auth().login(email, pass).await?;

// Or create directly
let services = Services::from_connection(db.get_connection(), config);
```

**Parallel Execution:**
```rust
use crate::services::parallel;

// Execute two operations in parallel
let (user, posts) = parallel::join2(
    services.users().get_user(id),
    services.posts().get_user_posts(id),
).await?;

// Execute many operations in parallel
let user_ids = vec![id1, id2, id3];
let futures: Vec<_> = user_ids
    .iter()
    .map(|id| services.users().get_user(*id))
    .collect();
let users = parallel::join_all(futures).await?;

// Limit concurrency (e.g., max 10 concurrent DB queries)
let users = parallel::join_all_limited(
    user_ids.into_iter().map(|id| services.users().get_user(id)),
    10,
).await?;
```

**Batch Processing:**
```rust
use crate::services::batch;

// Process items in batches of 100
let all_ids: Vec<Uuid> = get_all_user_ids();
let users = batch::process(
    all_ids,
    100, // batch size
    |id| services.users().get_user(id),
).await?;
```

**Pipeline Pattern:**
```rust
use crate::services::Pipeline;

let result = Pipeline::new(user_id)
    .then(|id| services.users().get_user(id))
    .await?
    .map(|user| user.email)
    .finish();
```

**Naming Convention:**
| Pattern | Example | Description |
|---------|---------|-------------|
| `*Service` | `AuthService`, `UserService` | Service trait for dependency injection |
| Fluent name | `Authenticator`, `UserManager` | Service concrete implementation |
| `*Repository` | `UserRepository` | Repository trait for data access |
| `*Store` | `UserStore` | Repository concrete implementation |
| `UnitOfWork` | `UnitOfWork` | Unit of Work trait |
| `Persistence` | `Persistence` | Unit of Work concrete implementation |
| `ServiceContainer` | `ServiceContainer` | Service container trait |
| `Services` | `Services` | Service container concrete implementation |

## Unit of Work Pattern (Optional)

For services that need transactional operations across multiple repositories, the Unit of Work pattern provides centralized repository access and transaction management.

**Benefits:**
- Centralizes access to all repositories through a single entry point
- Manages database transactions (begin, commit, rollback)
- Ensures consistency across multiple repository operations
- Simplifies testing with mockable UnitOfWork trait

**Basic Usage:**
```rust
// Create UnitOfWork
let uow = Arc::new(Persistence::new(db.get_connection()));

// Access repositories
let user = uow.users().find_by_id(id).await?;

// Use in services
let auth_service = Authenticator::new(uow.clone(), config);
let user_service = UserManager::new(uow);
```

**Transactional Operations:**
```rust
// Execute multiple operations atomically
let result = uow.transaction(|ctx| Box::pin(async move {
    // All operations use the same transaction
    let user = ctx.users().create(email, hash, name).await?;

    // If any operation fails, everything is rolled back
    ctx.users().update(user.id, Some("Updated".into()), None).await?;

    Ok(user)
})).await?;

// Or use the helper macro
use rust_api_starter::with_transaction;

let user = with_transaction!(uow, |ctx| {
    ctx.users().create(email, hash, name).await
})?;
```

**Architecture:**
```
┌─────────────────────────────────────────────┐
│              Service Layer                   │
│  (Authenticator, UserManager)        │
└──────────────────┬──────────────────────────┘
                   │
                   ▼
┌─────────────────────────────────────────────┐
│              Unit of Work                    │
│  - Centralizes repository access            │
│  - Manages transactions                     │
│  - Provides TransactionContext              │
└──────────────────┬──────────────────────────┘
                   │
        ┌──────────┴──────────┐
        ▼                     ▼
┌───────────────┐    ┌───────────────┐
│ UserRepository│    │ OtherRepository│
└───────────────┘    └───────────────┘
```

**Adding New Repositories:**

1. Add repository trait method to `UnitOfWork`:
```rust
#[async_trait]
pub trait UnitOfWork: Send + Sync {
    fn users(&self) -> Arc<dyn UserRepository>;
    fn products(&self) -> Arc<dyn ProductRepository>;  // New
}
```

2. Add transaction-aware implementation:
```rust
impl TransactionContext {
    pub fn products(&self) -> Arc<dyn ProductRepository> {
        self.product_repo.clone()
    }
}
```

3. Update `Persistence` to create repository instances.

**Testing with Mock:**
```rust
#[cfg(test)]
use crate::infra::MockUnitOfWork;

#[tokio::test]
async fn test_service() {
    let mut mock_uow = MockUnitOfWork::new();
    mock_uow.expect_users()
        .returning(|| Arc::new(MockUserRepository::new()));

    let service = UserManager::new(Arc::new(mock_uow));
    // ... test service
}
```

## Soft Delete

The application implements soft delete for user entities, allowing data recovery and audit trails.

**How It Works:**
- Records are not permanently deleted from the database
- A `deleted_at` timestamp marks when record was deleted
- `deleted_at = NULL` means active, `deleted_at = timestamp` means deleted
- By default, all queries exclude soft-deleted records
- Use `*_with_deleted` variants to include deleted records

**Domain Entity:**
```rust
pub struct User {
    // ... other fields
    pub deleted_at: Option<DateTime<Utc>>,
}

impl User {
    pub fn is_deleted(&self) -> bool { self.deleted_at.is_some() }
    pub fn is_active(&self) -> bool { self.deleted_at.is_none() }
    pub fn soft_delete(&mut self) { ... }
    pub fn restore(&mut self) { ... }
}
```

**Repository Methods:**
| Method | Description |
|--------|-------------|
| `find_by_id(id)` | Find active user (excludes deleted) |
| `find_by_id_with_deleted(id)` | Find user including deleted |
| `find_by_email(email)` | Find active user by email |
| `find_by_email_with_deleted(email)` | Find user by email including deleted |
| `list()` | List active users |
| `list_with_deleted()` | List all users including deleted |
| `list_deleted()` | List only deleted users |
| `delete(id)` | Soft delete (set deleted_at) |
| `hard_delete(id)` | Permanent deletion |
| `restore(id)` | Restore soft-deleted user |

**Service Methods:**
```rust
// Get active user (excludes deleted)
service.get_user(id).await?;

// Get user including deleted
service.get_user_with_deleted(id).await?;

// List methods
service.list_users().await?;           // Active only
service.list_users_with_deleted().await?; // All
service.list_deleted_users().await?;   // Deleted only

// Delete operations
service.delete_user(id).await?;        // Soft delete
service.hard_delete_user(id).await?;   // Permanent

// Restore
service.restore_user(id).await?;
```

**Transaction Context:**
```rust
uow.transaction(|ctx| Box::pin(async move {
    // Soft delete
    ctx.users().delete(id).await?;

    // Restore
    ctx.users().restore(id).await?;

    // Hard delete (permanent)
    ctx.users().hard_delete(id).await?;

    Ok(())
})).await?;
```

**Database Schema:**
```sql
-- Migration: m20240102_000001_add_soft_delete
ALTER TABLE users ADD COLUMN deleted_at TIMESTAMPTZ NULL;
CREATE INDEX idx_users_deleted_at ON users(deleted_at);
```

**Benefits:**
- **Data Recovery**: Accidentally deleted data can be restored
- **Audit Trail**: Know when records were deleted
- **Referential Integrity**: Related records don't break
- **Compliance**: Keep records for legal/regulatory requirements

**Adding Soft Delete to New Entities:**

1. Add `deleted_at` to domain entity:
```rust
pub struct Product {
    // ... fields
    pub deleted_at: Option<DateTime<Utc>>,
}
```

2. Add `deleted_at` to database entity:
```rust
pub struct Model {
    // ... fields
    pub deleted_at: Option<DateTimeUtc>,
}
```

3. Create migration:
```rust
.add_column(
    ColumnDef::new(Products::DeletedAt)
        .timestamp_with_time_zone()
        .null(),
)
```

4. Add index for performance:
```rust
Index::create()
    .name("idx_products_deleted_at")
    .table(Products::Table)
    .col(Products::DeletedAt)
```

5. Update repository with soft delete logic (filter by `deleted_at.is_null()`)

## Redis Caching

The `infra/cache.rs` module provides a type-safe Redis caching layer.

**Features:**
- Connection pooling via `ConnectionManager`
- Generic get/set/delete with JSON serialization
- TTL support (default: 1 hour)
- User-specific cache operations
- Session management
- Rate limiting
- Distributed locks
- Semaphores for concurrency control

**Cache Key Prefixes:**
```
user:          - Cached user entities
session:       - Session data
rate_limit:    - Rate limiting counters
lock:          - Distributed locks
semaphore:     - Semaphore permits
```

**Basic Usage:**
```rust
// Initialize
let cache = Cache::connect(&config).await;

// Generic operations
cache.set("key", &value).await?;
cache.get::<MyType>("key").await?;
cache.delete("key").await?;

// User caching
cache.set_user(&user).await?;
cache.get_user(&user_id).await?;
cache.invalidate_user(&user_id).await?;

// Rate limiting
let (count, allowed) = cache.check_rate_limit("ip:127.0.0.1", 100, 60).await?;
```

## Rate Limiting

Redis-based rate limiting middleware to protect against abuse.

**Configured Limits:**

| Endpoint | Limit | Window | Purpose |
|----------|-------|--------|---------|
| `/auth/*` | 10 requests | 60 seconds | Prevent brute-force attacks |
| `/users/*` | 100 requests | 60 seconds | General API protection |
| `/`, `/health` | No limit | - | Health checks always available |

**Response Headers:**
```
X-RateLimit-Limit: 100        # Max requests allowed
X-RateLimit-Remaining: 95     # Requests remaining
Retry-After: 60               # Seconds until reset (on 429)
```

**When Limit Exceeded:**
```http
HTTP/1.1 429 Too Many Requests
Retry-After: 60
X-RateLimit-Remaining: 0

Too many requests. Please try again later.
```

**IP Detection (in order):**
1. `X-Forwarded-For` header (first IP)
2. `X-Real-IP` header
3. Connection socket address
4. Fallback: "unknown"

**Middleware Usage:**
```rust
// In routes.rs
Router::new()
    .nest(
        "/auth",
        auth_routes().route_layer(middleware::from_fn_with_state(
            state.clone(),
            rate_limit_auth_middleware,  // Stricter: 10/min
        )),
    )
    .nest(
        "/users",
        user_routes()
            .route_layer(middleware::from_fn_with_state(
                state.clone(),
                rate_limit_middleware,  // General: 100/min
            )),
    )
```

**Cache-Level Rate Limiting:**
```rust
// Direct cache usage for custom limits
let (count, allowed) = cache.check_rate_limit(
    "api:user:123",  // Identifier
    50,              // Max requests
    3600,            // Window in seconds (1 hour)
).await?;

if !allowed {
    return Err(AppError::TooManyRequests);
}

// Check remaining
let remaining = cache.get_rate_limit_remaining("api:user:123", 50).await?;
```

**Constants:**
```rust
RATE_LIMIT_REQUESTS = 100              // General limit
RATE_LIMIT_WINDOW_SECONDS = 60         // General window
RATE_LIMIT_AUTH_REQUESTS = 10          // Auth limit
RATE_LIMIT_AUTH_WINDOW_SECONDS = 60    // Auth window
```

## Distributed Locks

Redis-based distributed locks for coordinating access across multiple instances.

**Features:**
- RAII guards (auto-release on drop)
- Atomic operations via Lua scripts
- TTL protection against deadlocks (default: 30s)
- Configurable retry logic
- Lock extension for long operations

**Usage:**
```rust
// Acquire lock (blocks with retries)
let lock = cache.acquire_lock("user:123:update").await?;
// ... critical section ...
lock.release().await?;  // or auto-releases on drop

// Try without blocking
if let Some(lock) = cache.try_acquire_lock("resource").await? {
    // Got the lock
    // Auto-releases when lock goes out of scope
}

// Custom options
let lock = cache.acquire_lock_with_options(
    "resource",
    60,    // TTL seconds
    5,     // max retries
    200,   // retry delay ms
).await?;

// Extend lock for long operations
lock.extend(60).await?;

// Check if locked
let is_locked = cache.is_locked("resource").await?;
```

**Defaults:**
```rust
DEFAULT_LOCK_TTL_SECONDS = 30
DEFAULT_LOCK_RETRIES = 10
DEFAULT_LOCK_RETRY_DELAY_MS = 100
```

## Semaphores

Distributed semaphores for limiting concurrent access to resources.

**Use Cases:**
- Limit concurrent file uploads
- Control parallel API calls to external services
- Database connection pooling
- Resource throttling

**Usage:**
```rust
// Limit to 5 concurrent uploads
let permit = cache.acquire_semaphore("file:upload", 5).await?;
// ... upload file ...
permit.release().await?;  // or auto-releases on drop

// Try without blocking
if let Some(permit) = cache.try_acquire_semaphore("uploads", 5).await? {
    // Got a permit
}

// Custom options
let permit = cache.acquire_semaphore_with_options(
    "resource",
    5,     // max permits
    60,    // TTL seconds
    10,    // max retries
    100,   // retry delay ms
).await?;

// Check current count
let count = cache.semaphore_count("file:upload").await?;
```

**Example - Limit Concurrent External API Calls:**
```rust
async fn call_external_api(cache: &Cache, data: &Data) -> AppResult<Response> {
    // Only allow 10 concurrent calls
    let permit = cache.acquire_semaphore("external:api", 10).await?;

    let response = external_client.call(data).await?;

    permit.release().await?;
    Ok(response)
}
```

## gRPC Communication

Inter-service communication uses Protocol Buffers (protobuf) with Tonic gRPC.

**Proto Definitions (`crates/shared/proto/proto/`):**

```protobuf
// auth.proto - Authentication service
service AuthService {
    rpc Register(RegisterRequest) returns (RegisterResponse);
    rpc Login(LoginRequest) returns (LoginResponse);
    rpc VerifyToken(VerifyTokenRequest) returns (VerifyTokenResponse);
    rpc RefreshToken(RefreshTokenRequest) returns (LoginResponse);
}

// user.proto - User management service
service UserService {
    // Public methods (no password hash in response)
    rpc GetUser(GetUserRequest) returns (UserResponse);
    rpc GetUserByEmail(GetUserByEmailRequest) returns (UserResponse);
    rpc ListUsers(ListUsersRequest) returns (ListUsersResponse);
    rpc CreateUser(CreateUserRequest) returns (UserResponse);
    rpc UpdateUser(UpdateUserRequest) returns (UserResponse);
    rpc DeleteUser(DeleteUserRequest) returns (DeleteUserResponse);
    rpc RestoreUser(RestoreUserRequest) returns (UserResponse);

    // Internal methods (includes password hash - for auth-service only)
    rpc GetUserByEmailInternal(GetUserByEmailRequest) returns (InternalUserResponse);
    rpc GetUserByEmailInternalWithDeleted(GetUserByEmailRequest) returns (InternalUserResponse);
}
```

**Proto Message Security Model:**

| Message | Contains Password Hash | Used By |
|---------|----------------------|---------|
| `UserResponse` | No | Gateway, external clients |
| `InternalUserResponse` | Yes | Auth-service only |

This separation ensures password hashes never leave the user-service except to auth-service for verification.

**Adding New gRPC Methods:**

1. Define in proto file (`crates/shared/proto/proto/*.proto`)
2. Run `cargo build -p proto` to regenerate code
3. Implement server trait in service crate
4. Add client method in gateway

**Service Communication Pattern:**
```
Gateway (HTTP) → AuthClient (gRPC) → Auth Service
                                        ↓
Gateway (HTTP) → UserClient (gRPC) → User Service ← Auth Service (gRPC)
```

## Adding New Microservices

**To add a new service (e.g., `product-service`):**

1. **Create the crate:**
```bash
mkdir -p crates/services/product-service/src/{grpc,service,repository}
```

2. **Add to workspace (`Cargo.toml`):**
```toml
[workspace]
members = [
    # ... existing
    "crates/services/product-service",
]
```

3. **Create proto definition (`crates/shared/proto/proto/product.proto`):**
```protobuf
syntax = "proto3";
package product;

service ProductService {
    rpc GetProduct(GetProductRequest) returns (ProductResponse);
}
```

4. **Implement gRPC server (`src/grpc/product_grpc.rs`):**
```rust
#[tonic::async_trait]
impl ProductServiceProto for ProductGrpcService {
    async fn get_product(...) -> Result<Response<ProductResponse>, Status> {
        // Implementation
    }
}
```

5. **Add client in gateway (`crates/gateway/src/clients/product_client.rs`):**
```rust
pub struct ProductClient {
    client: ProductServiceClient<Channel>,
}
```

6. **Wire into combined binary and docker-compose**
