pub struct Wait {
    symbols: Vec<&'static str>,
    current: usize,
}

impl Wait {
    // Constructor for the Wait struct
    pub fn new(symbols: Vec<&'static str>) -> Wait {
        Wait {
            symbols,
            current: 0,
        }
    }

    // cycle through symbols
    pub fn next(&mut self) -> &str {
        let symbol = self.symbols[self.current];
        self.current = (self.current + 1) % self.symbols.len();
        symbol
    }
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wait_next() {
        let mut wait = Wait::new(vec!["-", "\\", "|", "/"]);
        assert_eq!(wait.next(), "-"); // First call to next() should return "-"
        assert_eq!(wait.next(), "\\"); // Second call to next() should return "\\"
        assert_eq!(wait.next(), "|"); // Third call to next() should return "|"
        assert_eq!(wait.next(), "/"); // Fourth call to next() should return "/"
        assert_eq!(wait.next(), "-"); // Fifth call, should wrap around to "-"
                                      // If you call next() more, it should continue to cycle through the symbols
    }
}
