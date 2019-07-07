use crate::error::{Error, Result};

pub struct Deserializer<'de> {
    input: &'de [u8],
}

impl<'de> Deserializer<'de> {
    pub fn new(input: &'de [u8]) -> Self {
        Deserializer { input }
    }

    fn peek(&mut self) -> Result<u8> {
        self.input.first().copied().ok_or(Error::EndOfInput)
    }

    fn next(&mut self) -> Result<u8> {
        let b = self.peek()?;
        self.input = &self.input[1..];
        Ok(b)
    }

    pub fn assert_end(&self) -> Result<()> {
        if self.input.len() > 0 {
            return Err(Error::TrailingCharacters);
        }
        Ok(())
    }

    /// Attempts to match a single byte matching the pattern. Does not consume
    /// on failure.
    fn consume(&mut self, pattern: impl BytePattern) -> bool {
        match self.peek() {
            Ok(b) if pattern.matches(b) => {
                self.next().unwrap();
                true
            }
            _ => false,
        }
    }

    /// Consumes one or more spaces.
    ///
    /// By convention, all parsers (methods starting with `parse_`) handle
    /// trailing whitespace.
    pub fn space(&mut self) {
        while self.consume(b"\t\n ") {}
    }

    /// Parses a translatable marker (`_`).
    pub fn parse_translatable_marker(&mut self) -> Result<()> {
        if !self.consume(b'_') {
            return Err(Error::ExpectedTranslatable);
        }
        self.space();
        Ok(())
    }

    /// Parses a string.
    pub fn parse_string(&mut self) -> Result<Vec<u8>> {
        if self.next()? != b'"' {
            return Err(Error::ExpectedString);
        }
        let mut result = Vec::new();
        loop {
            match self.next()? {
                b'"' => {
                    if self.consume(b'"') {
                        result.push(b'"');
                    } else {
                        break;
                    }
                }
                b => result.push(b),
            }
        }
        self.space();
        Ok(result)
    }
}

trait BytePattern {
    fn matches(&self, input: u8) -> bool;
}

impl BytePattern for u8 {
    fn matches(&self, input: u8) -> bool {
        *self == input
    }
}

impl<const LEN: usize> BytePattern for [u8; LEN] {
    fn matches(&self, input: u8) -> bool {
        self.contains(&input)
    }
}

impl<'a, P> BytePattern for &'a P where P: BytePattern {
    fn matches(&self, input: u8) -> bool {
        <P as BytePattern>::matches(*self, input)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn trans() {
        let mut de = Deserializer::new(br#"_ "#);
        let result = de.parse_translatable_marker().unwrap();
        de.assert_end().unwrap();
        assert_eq!(result, ());
    }

    #[test]
    fn cis() {
        let mut de = Deserializer::new(br#""hello""#);
        let result = de.parse_translatable_marker().unwrap_err();
        assert_eq!(result, Error::ExpectedTranslatable);
    }

    #[test]
    fn string() {
        let mut de = Deserializer::new(br#""hello""#);
        let result = de.parse_string().unwrap();
        assert_eq!(result, b"hello");
    }

    #[test]
    fn string_escapes() {
        let mut de = Deserializer::new(br#""hello ""world""""#);
        let result = de.parse_string().unwrap();
        assert_eq!(result, br#"hello "world""#);
    }
}
