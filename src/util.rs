use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

/// 🔢 Berechnet einen konsistenten u64 Hash für beliebige hashbare Daten.
///
/// # Beispiel
/// ```
/// let h = util::hash_data(&"hello");
/// ```
pub fn hash_data<T: Hash>(data: &T) -> u64 {
    let mut hasher = DefaultHasher::new();
    data.hash(&mut hasher);
    hasher.finish()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash_data_consistency() {
        let input = "test input";
        let h1 = hash_data(&input);
        let h2 = hash_data(&input);
        assert_eq!(h1, h2, "Hash sollte für gleichen Input konsistent sein.");
    }

    #[test]
    fn test_hash_data_difference() {
        let input1 = "foo";
        let input2 = "bar";
        let h1 = hash_data(&input1);
        let h2 = hash_data(&input2);
        assert_ne!(
            h1, h2,
            "Hashes für verschiedene Inputs sollten unterschiedlich sein."
        );
    }

    #[test]
    fn test_hash_data_u64_range() {
        let input = "some data";
        let h = hash_data(&input);
        assert!(h <= u64::MAX, "Hash sollte innerhalb von u64 liegen.");
    }
}
