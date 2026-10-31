//! The meaning of an event: an open-vocabulary label ("preso", "fallito",
//! "filo aperto"). The labels are defined by the author's rules; the kernel
//! treats them as opaque text and never interprets them.

/// An open-vocabulary event label. Who defines the labels is the author's
/// rules; the kernel compares and orders them, nothing more.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Meaning(String);

impl Meaning {
    /// Creates a label. Takes ownership of the text: a label is kept, not
    /// borrowed.
    pub fn new(label: impl Into<String>) -> Self {
        Self(label.into())
    }

    /// Returns the label text.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn meaning_compares_and_orders() {
        let opened = Meaning::new("aperto");
        let closed = Meaning::new("chiuso");
        assert_eq!(opened, Meaning::new("aperto"));
        assert_ne!(opened, closed);
        assert!(opened < closed);
        assert_eq!(opened.as_str(), "aperto");
        assert!(opened.clone() == opened);
    }
}
