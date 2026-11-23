use wyrm_utils::{error::WyrmError, result::WyrmResult};

const MIN_LENGTH: usize = 3;
const MAX_LENGTH: usize = 32;

/// Trims and lowercases a username, then checks it's 3 to 32 characters of a-z, 0-9 and _.
pub fn normalize_username(input: &str) -> WyrmResult<String> {
    let username = input.trim().to_ascii_lowercase();

    let valid_length = (MIN_LENGTH..=MAX_LENGTH).contains(&username.len());
    let valid_characters = username
        .chars()
        .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_');

    if !valid_length || !valid_characters {
        return Err(WyrmError::InvalidInput(
            "username must be 3 to 32 characters: letters, digits and _".into(),
        ));
    }
    Ok(username)
}
