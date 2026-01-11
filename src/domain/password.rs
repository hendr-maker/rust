//! Password value object - Domain layer password handling.
//!
//! DDD: Encapsulates password hashing as a domain value object.
//! SOLID (SRP): Single responsibility - password operations only.
//! DRY: Centralized Argon2 configuration.

use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};

use crate::config::MIN_PASSWORD_LENGTH;
use crate::errors::{AppError, AppResult};

/// Password value object that handles hashing and verification.
///
/// DDD: Value object - immutable, compared by value.
/// Encapsulates all password-related domain logic.
#[derive(Clone)]
pub struct Password {
    hash: String,
}

// Don't expose hash in debug output (security)
impl std::fmt::Debug for Password {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Password")
            .field("hash", &"[REDACTED]")
            .finish()
    }
}

impl Password {
    /// Create a new password by hashing the plain text.
    ///
    /// # Arguments
    /// * `plain_text` - The raw password to hash (minimum 8 characters)
    ///
    /// # Returns
    /// * `AppResult<Password>` - Hashed password value object
    ///
    /// # Errors
    /// Returns validation error if password is too short.
    pub fn new(plain_text: &str) -> AppResult<Self> {
        // Validate password length
        if plain_text.len() < MIN_PASSWORD_LENGTH as usize {
            return Err(AppError::validation(format!(
                "Password must be at least {} characters",
                MIN_PASSWORD_LENGTH
            )));
        }

        let hash = Self::hash(plain_text)?;
        Ok(Self { hash })
    }

    /// Create a Password from an existing hash (from database).
    ///
    /// # Arguments
    /// * `hash` - The existing password hash
    pub fn from_hash(hash: String) -> Self {
        Self { hash }
    }

    /// Get the hash string for storage.
    pub fn as_str(&self) -> &str {
        &self.hash
    }

    /// Consume and return the hash string.
    pub fn into_string(self) -> String {
        self.hash
    }

    /// Verify a plain text password against this hash.
    ///
    /// # Arguments
    /// * `plain_text` - The password to verify
    ///
    /// # Returns
    /// * `bool` - True if password matches
    pub fn verify(&self, plain_text: &str) -> bool {
        Self::verify_hash(plain_text, &self.hash).unwrap_or(false)
    }

    /// Hash a password using Argon2.
    /// DRY: Single hashing implementation.
    fn hash(plain_text: &str) -> AppResult<String> {
        let salt = SaltString::generate(&mut OsRng);
        let hash = Self::argon2()
            .hash_password(plain_text.as_bytes(), &salt)
            .map_err(|e| AppError::internal(format!("Password hash failed: {}", e)))?;
        Ok(hash.to_string())
    }

    /// Verify password against hash.
    /// DRY: Single verification implementation.
    fn verify_hash(plain_text: &str, hash: &str) -> AppResult<bool> {
        let parsed = PasswordHash::new(hash)
            .map_err(|e| AppError::internal(format!("Invalid hash format: {}", e)))?;
        Ok(Self::argon2()
            .verify_password(plain_text.as_bytes(), &parsed)
            .is_ok())
    }

    /// Get Argon2 instance with default config.
    /// DRY: Single configuration point.
    #[inline]
    fn argon2() -> Argon2<'static> {
        Argon2::default()
    }
}

impl From<Password> for String {
    fn from(password: Password) -> Self {
        password.hash
    }
}

impl PartialEq for Password {
    fn eq(&self, other: &Self) -> bool {
        self.hash == other.hash
    }
}

impl Eq for Password {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_password_hash_and_verify() {
        let plain = "SecurePassword123!";
        let password = Password::new(plain).unwrap();

        assert!(password.verify(plain));
        assert!(!password.verify("WrongPassword123"));
    }

    #[test]
    fn test_password_from_hash() {
        let plain = "TestPassword123";
        let password = Password::new(plain).unwrap();
        let hash = password.as_str().to_string();

        let restored = Password::from_hash(hash);
        assert!(restored.verify(plain));
    }

    #[test]
    fn test_different_passwords_different_hashes() {
        let pass1 = Password::new("Password123!").unwrap();
        let pass2 = Password::new("Password456!").unwrap();

        assert_ne!(pass1.as_str(), pass2.as_str());
    }

    #[test]
    fn test_same_password_different_salts() {
        let plain = "SamePassword123";
        let pass1 = Password::new(plain).unwrap();
        let pass2 = Password::new(plain).unwrap();

        // Different salts produce different hashes
        assert_ne!(pass1.as_str(), pass2.as_str());
        // But both verify correctly
        assert!(pass1.verify(plain));
        assert!(pass2.verify(plain));
    }

    #[test]
    fn test_password_too_short() {
        let result = Password::new("short");
        assert!(result.is_err());
    }

    #[test]
    fn test_password_minimum_length() {
        // Exactly 8 characters should work
        let result = Password::new("12345678");
        assert!(result.is_ok());
    }
}
