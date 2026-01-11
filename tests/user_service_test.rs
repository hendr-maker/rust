//! User service unit tests.

use std::sync::Arc;

use async_trait::async_trait;
use chrono::Utc;
use mockall::predicate::eq;
use uuid::Uuid;

use rust_api_starter::domain::{User, UserRole};
use rust_api_starter::errors::{AppError, AppResult};
use rust_api_starter::infra::{UserRepository, UnitOfWork, TransactionContext};
use rust_api_starter::infra::repositories::MockUserRepository;
use rust_api_starter::services::{UserService, UserManager};

fn create_test_user(id: Uuid) -> User {
    User {
        id,
        email: "test@example.com".to_string(),
        password_hash: "hashed".to_string(),
        name: "Test User".to_string(),
        role: UserRole::User,
        created_at: Utc::now(),
        updated_at: Utc::now(),
        deleted_at: None,
    }
}

/// Test mock for UnitOfWork that wraps a MockUserRepository
struct TestUnitOfWork {
    user_repo: Arc<MockUserRepository>,
}

impl TestUnitOfWork {
    fn new(user_repo: MockUserRepository) -> Self {
        Self {
            user_repo: Arc::new(user_repo),
        }
    }
}

#[async_trait]
impl UnitOfWork for TestUnitOfWork {
    fn users(&self) -> Arc<dyn UserRepository> {
        self.user_repo.clone()
    }

    async fn transaction<F, T>(&self, _f: F) -> AppResult<T>
    where
        F: for<'a> FnOnce(TransactionContext<'a>) -> std::pin::Pin<
                Box<dyn std::future::Future<Output = AppResult<T>> + Send + 'a>,
            > + Send,
        T: Send,
    {
        // Transaction not supported in test mock
        Err(AppError::internal("Transactions not supported in test mock"))
    }

    async fn transaction_serializable<F, T>(&self, _f: F) -> AppResult<T>
    where
        F: for<'a> FnOnce(TransactionContext<'a>) -> std::pin::Pin<
                Box<dyn std::future::Future<Output = AppResult<T>> + Send + 'a>,
            > + Send,
        T: Send,
    {
        // Transaction not supported in test mock
        Err(AppError::internal("Transactions not supported in test mock"))
    }
}

#[tokio::test]
async fn test_get_user_success() {
    let user_id = Uuid::new_v4();

    let mut repo = MockUserRepository::new();
    let uid = user_id;
    repo.expect_find_by_id()
        .with(eq(uid))
        .returning(move |id| Ok(Some(create_test_user(id))));

    let uow = TestUnitOfWork::new(repo);
    let service = UserManager::new(Arc::new(uow));
    let result = service.get_user(user_id).await;

    assert!(result.is_ok());
    assert_eq!(result.unwrap().id, user_id);
}

#[tokio::test]
async fn test_get_user_not_found() {
    let user_id = Uuid::new_v4();

    let mut repo = MockUserRepository::new();
    repo.expect_find_by_id()
        .returning(|_| Ok(None));

    let uow = TestUnitOfWork::new(repo);
    let service = UserManager::new(Arc::new(uow));
    let result = service.get_user(user_id).await;

    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), AppError::NotFound));
}

#[tokio::test]
async fn test_list_users_success() {
    let mut repo = MockUserRepository::new();
    repo.expect_list()
        .returning(|| Ok(vec![
            create_test_user(Uuid::new_v4()),
            create_test_user(Uuid::new_v4()),
        ]));

    let uow = TestUnitOfWork::new(repo);
    let service = UserManager::new(Arc::new(uow));
    let result = service.list_users().await;

    assert!(result.is_ok());
    assert_eq!(result.unwrap().len(), 2);
}

#[tokio::test]
async fn test_delete_user_success() {
    let user_id = Uuid::new_v4();

    let mut repo = MockUserRepository::new();
    repo.expect_delete()
        .returning(|_| Ok(()));

    let uow = TestUnitOfWork::new(repo);
    let service = UserManager::new(Arc::new(uow));
    let result = service.delete_user(user_id).await;

    assert!(result.is_ok());
}

#[tokio::test]
async fn test_restore_user_success() {
    let user_id = Uuid::new_v4();

    let mut repo = MockUserRepository::new();
    repo.expect_restore()
        .returning(move |id| Ok(create_test_user(id)));

    let uow = TestUnitOfWork::new(repo);
    let service = UserManager::new(Arc::new(uow));
    let result = service.restore_user(user_id).await;

    assert!(result.is_ok());
    assert_eq!(result.unwrap().id, user_id);
}
