# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

**rust-api-starter** - A production-ready Rust web API built with Axum, following Domain-Driven Design, Clean Architecture, SOLID, and DRY principles.

**Architecture Type:** Modular Monolith (microservice-ready)

## Tech Stack

- **Axum 0.7** - Web framework
- **SeaORM 1.0** - Async ORM for PostgreSQL
- **Redis** - Caching, rate limiting, distributed locks, semaphores
- **Apalis** - Background job processing
- **JWT** - Authentication (jsonwebtoken + Argon2)
- **Tokio** - Async runtime
- **Docker** - Containerization
- **Mockall** - Testing with mocks

## CLI Commands

```bash
# Start server
cargo run -- serve
cargo run -- serve --host 127.0.0.1 --port 8080

# Database migrations
cargo run -- migrate up          # Run pending migrations
cargo run -- migrate down        # Rollback last migration
cargo run -- migrate status      # Check migration status
cargo run -- migrate fresh       # Reset and re-run all

# Background jobs
cargo run -- jobs work           # Start job worker
cargo run -- jobs list           # List pending jobs

# Code generation
cargo run -- generate entity product
cargo run -- generate service payment
cargo run -- generate migration create_orders

# Verbose mode
cargo run -- -v serve
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
docker-compose up -d             # Start app + PostgreSQL + Redis
docker-compose down              # Stop containers
docker-compose logs -f           # View logs
docker-compose exec redis redis-cli  # Redis CLI
```

## Health Check Endpoint

The `/health` endpoint checks connectivity to all external services.

**Request:**
```bash
curl http://localhost:3000/health
```

**Response (200 OK - All healthy):**
```json
{
  "status": "healthy",
  "services": {
    "database": {
      "status": "healthy"
    },
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
    "database": {
      "status": "unhealthy",
      "error": "Connection refused"
    },
    "redis": {
      "status": "healthy"
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

```
rust-api-starter/
├── Cargo.toml
├── Dockerfile
├── docker-compose.yml
├── .env.example
│
├── src/
│   ├── main.rs                  # CLI entry point
│   ├── lib.rs                   # Library root, exports
│   │
│   ├── cli/                     # CLI argument parsing
│   │   ├── mod.rs
│   │   └── args.rs              # Clap definitions
│   │
│   ├── commands/                # CLI command implementations
│   │   ├── mod.rs
│   │   ├── serve.rs             # Start server
│   │   ├── migrate.rs           # Database migrations
│   │   ├── jobs.rs              # Background jobs
│   │   └── generate.rs          # Code generation
│   │
│   ├── config/                  # Configuration layer
│   │   ├── mod.rs
│   │   ├── constants.rs         # Named constants
│   │   └── settings.rs          # Environment config
│   │
│   ├── domain/                  # Domain layer (DDD)
│   │   ├── mod.rs
│   │   ├── user.rs              # User entity, DTOs
│   │   └── password.rs          # Password value object
│   │
│   ├── services/                # Application layer
│   │   ├── mod.rs
│   │   ├── container.rs         # Service Container + parallel utils
│   │   ├── auth_service.rs      # Authentication use cases
│   │   └── user_service.rs      # User use cases
│   │
│   ├── infra/                   # Infrastructure layer
│   │   ├── mod.rs
│   │   ├── db.rs                # Database + migrations
│   │   ├── cache.rs             # Redis caching
│   │   ├── unit_of_work.rs      # Unit of Work pattern
│   │   └── repositories/        # Data access
│   │
│   ├── api/                     # Presentation layer
│   │   ├── mod.rs
│   │   ├── routes.rs            # Route configuration
│   │   ├── state.rs             # AppState (DI container)
│   │   ├── handlers/            # HTTP handlers
│   │   ├── middleware/          # Auth middleware
│   │   └── extractors/          # Custom extractors
│   │
│   ├── jobs/                    # Background jobs
│   ├── types/                   # Shared types (DRY)
│   ├── utils/                   # Utilities (templates)
│   └── errors.rs                # Centralized errors
│
└── tests/                       # Integration tests
```

## Layer Dependencies (Clean Architecture)

```
┌─────────────────────────────────────────┐
│              main.rs                     │  Entry Point
├─────────────────────────────────────────┤
│                api/                      │  Presentation
│  (handlers, middleware, routes)          │
├─────────────────────────────────────────┤
│              services/                   │  Application
│  (use cases, orchestration)              │
├─────────────────────────────────────────┤
│               domain/                    │  Domain (Core)
│  (entities, value objects, rules)        │
├─────────────────────────────────────────┤
│               infra/                     │  Infrastructure
│  (db, repositories, external APIs)       │
└─────────────────────────────────────────┘

Dependencies flow INWARD (outer layers depend on inner)
Domain layer has NO external dependencies
```

## SOLID Principles

**S - Single Responsibility:** Each module has one purpose
- `domain/user.rs` - User entity only
- `services/auth_service.rs` - Authentication only
- `api/handlers/` - HTTP handling only

**O - Open/Closed:** Extend via new implementations
- Add new error variants without changing `AppError`
- Add new services implementing existing traits

**L - Liskov Substitution:** All implementations interchangeable
- `MockUserRepository` can replace `UserStore`

**I - Interface Segregation:** Small, focused traits
- `ReadRepository`, `WriteRepository`, `DeleteRepository` separate

**D - Dependency Inversion:** Depend on abstractions
- Services depend on `dyn UserRepository`, not `UserStore`

## DRY Patterns

**Constants:** All magic values in `config/constants.rs`

**Domain types:** `User`, `UserRole`, `UserResponse` in `domain/`

**Extractors:** `ValidatedJson<T>` auto-validates requests

**Pagination:** `Paginated<T>`, `PaginationParams` reusable

**Errors:** `AppError::conflict()`, `option.ok_or_not_found()?`

## Adding New Features

**New Domain Entity:**
1. Create in `domain/` with entity, value objects, DTOs
2. Export in `domain/mod.rs`

**New Repository:**
1. Create entity in `infra/repositories/entities/`
2. Create trait + impl in `infra/repositories/`
3. Add `#[cfg_attr(test, automock)]` for testing

**New Service:**
1. Create trait + impl in `services/`
2. Inject repository via constructor
3. Wire in `main.rs`, add to `AppState`

**New API Endpoint:**
1. Create handler in `api/handlers/`
2. Add routes in handler module
3. Register in `api/routes.rs`

**New Background Job:**
1. Create in `jobs/` implementing `Job` trait
2. Export in `jobs/mod.rs`

## Environment Variables

```bash
DATABASE_URL=postgres://user:pass@localhost:5432/db
REDIS_URL=redis://127.0.0.1:6379
JWT_SECRET=your-secret-key-minimum-32-characters  # Required, min 32 chars
JWT_EXPIRATION_HOURS=24
SERVER_HOST=0.0.0.0
SERVER_PORT=3000
```

## Security Features

- **JWT Secret**: Required in production, minimum 32 characters
- **Password Hashing**: Argon2 with random salt per password
- **Authorization**: Role-based access control (user/admin)
- **Input Validation**: All requests validated with validator crate
- **Sensitive Data**: Passwords and secrets never logged or exposed
- **Rate Limiting**: Fail-closed design - denies requests when Redis unavailable
- **Timing Attack Protection**: Constant-time password verification prevents email enumeration
- **Atomic Operations**: Semaphores use Lua scripts to prevent race conditions

## Security Hardening

The codebase implements several security best practices:

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

## Service Container

The `services/container.rs` module provides centralized service access with parallel execution utilities.

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

## Unit of Work Pattern

The `infra/unit_of_work.rs` module provides centralized repository access and transaction management.

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

## Modular Monolith Architecture

This project uses a **Modular Monolith** pattern, not microservices.

```
┌─────────────────────────────────────────────────────────┐
│                    SINGLE BINARY                        │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐     │
│  │ AuthService │  │ UserService │  │  EmailJob   │     │
│  └──────┬──────┘  └──────┬──────┘  └──────┬──────┘     │
│         └────────────────┼────────────────┘             │
│                          ▼                              │
│          ┌───────────────┐  ┌───────────────┐          │
│          │   Database    │  │     Redis     │          │
│          └───────────────┘  └───────────────┘          │
└─────────────────────────────────────────────────────────┘
```

**Why Modular Monolith:**

| Aspect | Monolith | Microservices |
|--------|----------|---------------|
| Deployment | Single binary | Multiple binaries |
| Database | Shared | Per service |
| Communication | Function calls | HTTP/gRPC/Queue |
| Complexity | Lower | Higher |
| Team size | 1-10 developers | 10+ developers |

**Benefits:**
- Simple deployment (one container)
- No network latency between services
- Easier debugging (single process)
- Lower infrastructure cost
- Shared code without duplication

**Microservice-Ready Features:**
- Clean service boundaries via traits
- Dependency injection via `Arc<dyn Trait>`
- Each service can be extracted to separate crate
- Domain layer has no infrastructure dependencies

**When to Migrate to Microservices:**
- Team grows beyond 10 developers
- Need independent scaling per service
- Different tech stacks needed
- Deployment independence required
