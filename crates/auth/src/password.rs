use argon2::{
    Argon2,
    password_hash::{PasswordHasher, PasswordVerifier, phc::PasswordHash},
};
use wyrm_utils::{error::AuthError, result::WyrmResult};

/// Hashes a password with Argon2id.
pub fn hash_password(password: &str) -> WyrmResult<String> {
    let hash = Argon2::default()
        .hash_password(password.as_bytes())
        .map_err(|e| AuthError::PasswordHashing(e.to_string()))?;
    Ok(hash.to_string())
}

/// False for a wrong password, and for a stored hash that can't be read.
pub fn verify_password(password: &str, stored_hash: &str) -> bool {
    PasswordHash::new(stored_hash).is_ok_and(|hash| {
        Argon2::default()
            .verify_password(password.as_bytes(), &hash)
            .is_ok()
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn verifies_the_right_password_only() {
        let hash = hash_password("correct horse").unwrap();
        assert!(verify_password("correct horse", &hash));
        assert!(!verify_password("wrong horse", &hash));
    }

    #[test]
    fn same_password_hashes_differently() {
        assert_ne!(
            hash_password("same").unwrap(),
            hash_password("same").unwrap()
        );
    }

    #[test]
    fn unreadable_hash_never_verifies() {
        assert!(!verify_password("anything", "not a hash"));
    }
}
