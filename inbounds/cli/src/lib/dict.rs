use spellbook::Dictionary;
use std::sync::LazyLock;

static AFF: &str = include_str!("dict/en_US.aff");
static DIC: &str = include_str!("dict/en_US.dic");
static DICT: LazyLock<Dictionary> =
    LazyLock::new(|| Dictionary::new(AFF, DIC).expect("Failed to load dictionary"));

pub fn is_english_word(word: &str) -> bool {
    DICT.check(word)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn recognizes_common_word() {
        assert!(is_english_word("information"));
        assert!(is_english_word("comfortable"));
        assert!(is_english_word("programming"));
    }

    #[test]
    fn rejects_gibberish() {
        assert!(!is_english_word("abcdefghijk"));
        assert!(!is_english_word("xyzqwertabc"));
    }
}
