// @specre 01JMBJK7QRVX3N4P5G6H8W9Y0Z
use ulid::Ulid;

pub fn is_valid(s: &str) -> bool {
    s.len() == 26
        && s.chars()
            .all(|c| c.is_ascii_uppercase() || c.is_ascii_digit())
}

pub fn generate() -> String {
    Ulid::new().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use ulid::Ulid;

    #[test]
    fn generated_ulid_is_valid_and_26_chars() {
        let id = generate();
        assert_eq!(id.len(), 26);
        assert!(Ulid::from_string(&id).is_ok());
    }

    #[test]
    fn generated_ulids_are_unique() {
        let a = generate();
        let b = generate();
        assert_ne!(a, b);
    }
}
