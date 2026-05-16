#[cfg(feature = "unicode-width")]
use console::AnsiCodeIterator;
use portable_atomic::AtomicBool;
#[cfg(feature = "unicode-width")]
use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

static IS_AMBIGUOUS_WIDE: AtomicBool = AtomicBool::new(false);

/// Helper for calculating text width.
///
/// This structure encapsulates width calculation logic, allowing configuration
/// of how ambiguous-width characters are treated.
pub struct Width;

impl Width {
    /// Sets whether [UAX#11] ambiguous characters are treated as wide (CJK).
    ///
    /// [UAX#11]: https://www.unicode.org/reports/tr11/
    pub fn set_ambiguous_wide(val: bool) {
        IS_AMBIGUOUS_WIDE.store(val, portable_atomic::Ordering::SeqCst);
    }

    #[cfg(feature = "unicode-width")]
    fn is_ambiguous_wide() -> bool {
        IS_AMBIGUOUS_WIDE.load(portable_atomic::Ordering::Relaxed)
    }

    #[cfg(feature = "unicode-width")]
    pub(crate) fn char(c: char) -> usize {
        if Self::is_ambiguous_wide() {
            c.width_cjk()
        } else {
            c.width()
        }
        .unwrap_or(0) // Make control characters zero-width.
    }

    #[cfg(not(feature = "unicode-width"))]
    pub(crate) fn char(_c: char) -> usize {
        1
    }

    #[cfg(feature = "unicode-width")]
    pub(crate) fn str(s: &str) -> usize {
        if Self::is_ambiguous_wide() {
            UnicodeWidthStr::width_cjk(s)
        } else {
            UnicodeWidthStr::width(s)
        }
    }

    #[cfg(not(feature = "unicode-width"))]
    pub(crate) fn str(s: &str) -> usize {
        s.chars().count()
    }

    #[cfg(feature = "unicode-width")]
    pub(crate) fn ansi_str(s: &str) -> usize {
        let is_ambiguous_wide = Self::is_ambiguous_wide();
        let mut width = 0;
        for (chunk, is_ansi) in AnsiCodeIterator::new(s) {
            if !is_ansi {
                width += if is_ambiguous_wide {
                    UnicodeWidthStr::width_cjk(chunk)
                } else {
                    UnicodeWidthStr::width(chunk)
                };
            }
        }
        width
    }

    #[cfg(not(feature = "unicode-width"))]
    pub(crate) fn ansi_str(s: &str) -> usize {
        console::measure_text_width(s)
    }
}

#[cfg(test)]
// Serializes tests that modify or rely on the global CJK ambiguous width setting
// (via `Width::set_ambiguous_wide`) to avoid flakiness in parallel execution.
pub(crate) static WIDTH_TEST_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

#[cfg(test)]
#[cfg(feature = "unicode-width")]
mod tests {
    use super::*;

    #[test]
    fn char() {
        let _guard = WIDTH_TEST_LOCK.lock().unwrap();
        // Default (false)
        assert_eq!(Width::char('A'), 1);
        assert_eq!(Width::char('█'), 1);
        assert_eq!(Width::char('あ'), 2);
        assert_eq!(Width::char('\r'), 0);

        // Set to true
        Width::set_ambiguous_wide(true);
        assert_eq!(Width::char('A'), 1);
        assert_eq!(Width::char('█'), 2);
        assert_eq!(Width::char('あ'), 2);
        assert_eq!(Width::char('\r'), 0);

        // Reset
        Width::set_ambiguous_wide(false);
    }

    #[test]
    #[cfg(feature = "unicode-width")]
    fn str() {
        let _guard = WIDTH_TEST_LOCK.lock().unwrap();
        // Default (false)
        assert_eq!(Width::str("A"), 1);
        assert_eq!(Width::str("█"), 1);
        assert_eq!(Width::str("あ"), 2);

        // Set to true
        Width::set_ambiguous_wide(true);
        assert_eq!(Width::str("A"), 1);
        assert_eq!(Width::str("█"), 2);
        assert_eq!(Width::str("あ"), 2);

        // Reset
        Width::set_ambiguous_wide(false);
    }

    #[test]
    #[cfg(feature = "unicode-width")]
    fn ansi_str() {
        let _guard = WIDTH_TEST_LOCK.lock().unwrap();
        // Default (false)
        assert_eq!(Width::ansi_str("A"), 1);
        assert_eq!(Width::ansi_str("█"), 1);
        assert_eq!(Width::ansi_str("あ"), 2);
        assert_eq!(Width::ansi_str("\u{1b}[31m█\u{1b}[0m"), 1); // with ANSI

        // Set to true
        Width::set_ambiguous_wide(true);
        assert_eq!(Width::ansi_str("A"), 1);
        assert_eq!(Width::ansi_str("█"), 2);
        assert_eq!(Width::ansi_str("あ"), 2);
        assert_eq!(Width::ansi_str("\u{1b}[31m█\u{1b}[0m"), 2); // with ANSI

        // Reset
        Width::set_ambiguous_wide(false);
    }
}
