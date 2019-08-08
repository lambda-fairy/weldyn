#[derive(Debug, Eq, PartialEq)]
pub enum Token {
    Tag { key: Vec<u8>, is_open: bool },
    Attr { key: Vec<u8>, value: Vec<u8> },
}

pub struct Parser<'de> {
    input: &'de [u8],
}

impl<'de> Iterator for Parser<'de> {
    type Item = Token;

    fn next(&mut self) -> Option<Self::Item> {
        self.parse_attribute().map(|(key, value)| Token::Attr { key, value })
            .or_else(|| self.parse_tag().map(|(key, is_open)| Token::Tag { key, is_open }))
    }
}

impl<'de> Parser<'de> {
    pub fn new(input: &'de [u8]) -> Self {
        let mut parser = Parser { input };
        parser.space();
        parser
    }

    fn peek(&mut self) -> Option<u8> {
        self.input.first().copied()
    }

    pub fn assert_end(&self) -> Option<()> {
        if self.input.len() > 0 {
            return None;
        }
        Some(())
    }

    /// Attempts to match a single byte matching the pattern. Does not consume
    /// on failure.
    fn consume(&mut self, pattern: impl BytePattern) -> Option<u8> {
        match self.peek() {
            Some(b) if pattern.matches(b) => {
                self.input = &self.input[1..];
                Some(b)
            }
            _ => None,
        }
    }

    /// Consumes one or more spaces.
    ///
    /// By convention, all parsers (methods starting with `parse_`) handle
    /// trailing whitespace.
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

    fn parse_tag(&mut self) -> Option<(Vec<u8>, bool)> {
        self.consume(b'[')?;
        let is_open = self.consume(b'/').is_some();
        let key = self.identifier()?;
        self.consume(b']')?;
        Some((key, is_open))
    }

    fn parse_attribute(&mut self) -> Option<(Vec<u8>, Vec<u8>)> {
        let key = self.identifier()?;
        self.space();
        self.consume(b'=')?;
        let value = self.parse_string()?;
        Some((key, value))
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
        let mut de = Parser::new(br#"_ "#);
        let result = de.parse_translatable_marker().unwrap();
        de.assert_end().unwrap();
        assert_eq!(result, ());
    }

    #[test]
    fn cis() {
        let mut de = Parser::new(br#""hello""#);
        let result = de.parse_translatable_marker();
        assert!(result.is_none());
    }

    #[test]
    fn string() {
        let mut de = Parser::new(br#""hello""#);
        let result = de.parse_string().unwrap();
        assert_eq!(result, b"hello");
    }

    #[test]
    fn string_escapes() {
        let mut de = Parser::new(br#""hello ""world""""#);
        let result = de.parse_string().unwrap();
        assert_eq!(result, br#"hello "world""#);
    }
}
