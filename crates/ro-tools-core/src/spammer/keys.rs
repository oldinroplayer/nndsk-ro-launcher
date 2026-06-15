const SPAMMER_KEY_LABELS: &[&str] = &[
    "F1", "F2", "F3", "F4", "F5", "F6", "F7", "F8", "F9", "1", "2", "3", "4", "5", "6", "7", "8",
    "9", "0", "Q", "W", "E", "R", "T", "Y", "U", "I", "O", "P", "A", "S", "D", "F", "G", "H", "J",
    "K", "L", "Z", "X", "C", "V", "B", "N", "M",
];

pub fn is_valid_spammer_key(label: &str) -> bool {
    let upper = label.trim().to_ascii_uppercase();
    SPAMMER_KEY_LABELS.contains(&upper.as_str())
}

/// Dedup, uppercase labels, stable sort by UI order.
pub fn normalize_spammer_keys(keys: &[String]) -> Vec<String> {
    let mut out: Vec<String> = keys
        .iter()
        .map(|k| k.trim().to_ascii_uppercase())
        .filter(|k| is_valid_spammer_key(k))
        .collect();
    out.sort_by_key(|k| spammer_key_order(k));
    out.dedup();
    out
}

fn spammer_key_order(label: &str) -> u8 {
    SPAMMER_KEY_LABELS
        .iter()
        .position(|k| *k == label)
        .map(|i| i as u8)
        .unwrap_or(u8::MAX)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalize_dedups_and_orders() {
        let keys = normalize_spammer_keys(&[
            "z".into(),
            "F2".into(),
            "f1".into(),
            "F2".into(),
            "9".into(),
            "q".into(),
        ]);
        assert_eq!(keys, vec!["F1", "F2", "9", "Q", "Z"]);
    }

    #[test]
    fn rejects_invalid_labels() {
        for letter in 'A'..='Z' {
            assert!(is_valid_spammer_key(&letter.to_string()), "{letter}");
        }
        assert!(is_valid_spammer_key("f8"));
        assert!(!is_valid_spammer_key("SPACE"));
    }
}
