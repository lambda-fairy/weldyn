#[derive(Clone)]
pub struct Tokens<'de> {
    input: &'de [u8],
}

impl<'de> Tokens<'de> {
    pub fn new(input: &'de [u8]) -> Self {
        let mut tokens = Tokens { input };
        tokens.space();
        tokens
    }

    pub fn assert_end(&self) -> Option<()> {
        if self.input.len() > 0 {
            return None;
        }
        Some(())
    }

    fn attempt<F, T>(&mut self, action: F) -> Option<T> where F: FnOnce(&mut Tokens<'de>) -> Option<T> {
        let mut clone = self.clone();
        let result = action(&mut clone)?;
        *self = clone;
        Some(result)
    }

    /// Attempts to match a single byte matching the pattern. Does not consume
    /// on failure.
    fn consume(&mut self, pattern: impl BytePattern) -> Option<u8> {
        let mut input = self.input.iter();
        match input.next().copied() {
            Some(b) if pattern.matches(b) => {
                self.input = input.as_slice();
                Some(b)
            }
            _ => None,
        }
    }

    /// Consumes one or more spaces.
    ///
    /// By convention, all parsers (methods starting with `parse_` and `next_`)
    /// handle trailing whitespace.
    fn space(&mut self) {
        while self.consume(|b| b == b'\t' || b == b'\n' || b == b' ').is_some() {}
    }

    fn identifier(&mut self) -> Option<Vec<u8>> {
        fn is_key_byte(b: u8) -> bool {
            (b'a' <= b && b <= b'z') || b == b'_'
        }

        let first_byte = self.consume(is_key_byte)?;
        let mut result = vec![first_byte];
        while let Some(b) = self.consume(is_key_byte) {
            result.push(b);
        }

        Some(result)
    }

    pub fn next_open(&mut self) -> Option<Vec<u8>> {
        self.attempt(|this| {
            this.consume(b'[')?;
            let key = this.identifier()?;
            this.consume(b']')?;
            this.space();
            Some(key)
        })
    }

    pub fn next_close(&mut self) -> Option<Vec<u8>> {
        self.attempt(|this| {
            this.consume(b'[')?;
            this.consume(b'/')?;
            let key = this.identifier()?;
            this.consume(b']')?;
            this.space();
            Some(key)
        })
    }

    pub fn next_attribute(&mut self) -> Option<(Vec<u8>, Vec<u8>)> {
        self.attempt(|this| {
            let key = this.identifier()?;
            this.space();
            this.consume(b'=')?;
            let value = this.parse_string()?;
            Some((key, value))
        })
    }

    /// Parses a translatable marker (`_`).
    fn parse_translatable_marker(&mut self) -> Option<()> {
        self.consume(b'_')?;
        self.space();
        Some(())
    }

    /// Parses a string.
    fn parse_string(&mut self) -> Option<Vec<u8>> {
        self.consume(b'"')?;
        let mut result = Vec::new();
        loop {
            match self.consume(())? {
                b'"' => {
                    if self.consume(b'"').is_some() {
                        result.push(b'"');
                    } else {
                        break;
                    }
                }
                b => result.push(b),
            }
        }
        self.space();
        Some(result)
    }
}

trait BytePattern {
    fn matches(&self, byte: u8) -> bool;
}

impl<F> BytePattern for F where F: Fn(u8) -> bool {
    fn matches(&self, byte: u8) -> bool {
        self(byte)
    }
}

impl BytePattern for u8 {
    fn matches(&self, byte: u8) -> bool {
        *self == byte
    }
}

impl BytePattern for () {
    fn matches(&self, _byte: u8) -> bool {
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn trans() {
        let mut de = Tokens::new(br#"_ "#);
        let result = de.parse_translatable_marker().unwrap();
        de.assert_end().unwrap();
        assert_eq!(result, ());
    }

    #[test]
    fn cis() {
        let mut de = Tokens::new(br#""hello""#);
        let result = de.parse_translatable_marker();
        assert!(result.is_none());
    }

    #[test]
    fn string() {
        let mut de = Tokens::new(br#""hello""#);
        let result = de.parse_string().unwrap();
        assert_eq!(result, b"hello");
    }

    #[test]
    fn string_escapes() {
        let mut de = Tokens::new(br#""hello ""world""""#);
        let result = de.parse_string().unwrap();
        assert_eq!(result, br#"hello "world""#);
    }
}
