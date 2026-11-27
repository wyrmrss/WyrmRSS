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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_and_normalizes_valid_usernames() {
        let cases = [
            ("  Alice_01 ", "alice_01"),
            ("abc", "abc"),
            (&"a".repeat(MAX_LENGTH), &"a".repeat(MAX_LENGTH)),
        ];
        for (input, expected) in cases {
            assert_eq!(
                normalize_username(input).expect("should be valid"),
                expected,
                "input {input:?}"
            );
        }
    }

    #[test]
    fn rejects_invalid_usernames() {
        let too_long = "a".repeat(MAX_LENGTH + 1);
        let cases = ["ab", "   ", &too_long, "al ice", "al-ice", "élise"];
        for input in cases {
            assert!(
                matches!(normalize_username(input), Err(WyrmError::InvalidInput(_))),
                "input {input:?} should be rejected"
            );
        }
    }
}
