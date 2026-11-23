use argon2::{
    Argon2,
    password_hash::{PasswordHasher, PasswordVerifier, phc::PasswordHash},
};
use wyrm_utils::{
    error::{AuthError, WyrmError},
    result::WyrmResult,
};

const MIN_LENGTH: usize = 8;
const MAX_LENGTH: usize = 128;

/// Checks a new password's length, counted in characters.
pub fn validate_password(password: &str) -> WyrmResult<()> {
    let length = password.chars().count();
    if length < MIN_LENGTH {
        return Err(WyrmError::InvalidInput(format!(
            "password must be at least {MIN_LENGTH} characters"
        )));
    }
    if length > MAX_LENGTH {
        return Err(WyrmError::InvalidInput(format!(
            "password must be at most {MAX_LENGTH} characters"
        )));
    }
    Ok(())
}

/// Hashes a password with Argon2id.
/// Argon2 is slow on purpose, so both functions run it on tokio's blocking
/// threads rather than holding up an async worker.
pub async fn hash_password(password: String) -> WyrmResult<String> {
    let hashing_failed = |e: String| WyrmError::from(AuthError::PasswordHashing(e));

    tokio::task::spawn_blocking(move || {
        Argon2::default()
            .hash_password(password.as_bytes())
            .map(|hash| hash.to_string())
            .map_err(|e| hashing_failed(e.to_string()))
    })
    .await
    .map_err(|e| hashing_failed(e.to_string()))?
}

/// False for a wrong password, and for a stored hash that can't be read.
pub async fn verify_password(password: String, stored_hash: String) -> bool {
    tokio::task::spawn_blocking(move || {
        PasswordHash::new(&stored_hash).is_ok_and(|hash| {
            Argon2::default()
                .verify_password(password.as_bytes(), &hash)
                .is_ok()
        })
    })
    .await
    .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn verifies_the_right_password_only() {
        let hash = hash_password("correct horse".into()).await.unwrap();
        assert!(verify_password("correct horse".into(), hash.clone()).await);
        assert!(!verify_password("wrong horse".into(), hash).await);
    }

    #[tokio::test]
    async fn same_password_hashes_differently() {
        assert_ne!(
            hash_password("same".into()).await.unwrap(),
            hash_password("same".into()).await.unwrap()
        );
    }

    #[tokio::test]
    async fn unreadable_hash_never_verifies() {
        assert!(!verify_password("anything".into(), "not a hash".into()).await);
    }

    #[test]
    fn password_length_is_checked_in_characters() {
        assert!(validate_password("1234567").is_err());
        assert!(validate_password("12345678").is_ok());
        assert!(validate_password(&"a".repeat(MAX_LENGTH)).is_ok());
        assert!(validate_password(&"a".repeat(MAX_LENGTH + 1)).is_err());
        // 8 characters but 16 bytes: long enough.
        assert!(validate_password("éééééééé").is_ok());
    }

    #[test]
    fn spaces_count_and_are_kept() {
        assert!(validate_password("        ").is_ok());
    }
}
