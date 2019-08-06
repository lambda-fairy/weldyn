use crate::error::{Error, Result};

pub struct Parser<'de> {
    input: &'de [u8],
}

impl<'de> Parser<'de> {
    pub fn new(input: &'de [u8]) -> Self {
        Parser { input }
    }

    fn peek(&mut self) -> Option<u8> {
        self.input.first().copied()
    }

    fn next(&mut self) -> Option<u8> {
        let b = self.peek()?;
        self.input = &self.input[1..];
        Some(b)
    }

    pub fn assert_end(&self) -> Result<()> {
        if self.input.len() > 0 {
            return Err(Error::TrailingCharacters);
        }
        Ok(())
    }

    /// Attempts to match a single byte matching the pattern. Does not consume
    /// on failure.
    fn consume(&mut self, pattern: impl Fn(u8) -> bool) -> Option<u8> {
        match self.peek() {
            Some(b) if pattern(b) => {
                self.next().unwrap();
                Some(b)
            }
            _ => None,
        }
    }

    /// Consumes one or more spaces.
    ///
    /// By convention, all parsers (methods starting with `parse_`) handle
    /// trailing whitespace.
    pub fn space(&mut self) {
        while self.consume(|b| b == b'\t' || b == b'\n' || b == b' ').is_some() {}
    }

    fn identifier(&mut self, eof_error: Error) -> Result<Vec<u8>> {
        fn is_key_byte(b: u8) -> bool {
            (b'a' <= b && b <= b'z') || b == b'_'
        }

        let first_byte = self.next().ok_or(eof_error)?;
        if !is_key_byte(first_byte) {
            return Err(Error::ExpectedIdent);
        }

        let mut result = vec![first_byte];
        while let Some(b) = self.consume(is_key_byte) {
            result.push(b);
        }

        Ok(result)
    }

    pub fn parse_attribute(&mut self) -> Result<(Vec<u8>, Vec<u8>)> {
        let key = self.identifier(Error::EofWhileParsingAttribute)?;
        self.space();
        if self.next().ok_or(Error::EofWhileParsingAttribute)? != b'=' {
            return Err(Error::ExpectedEquals);
        }
        let value = self.parse_string()?;
        Ok((key, value))
    }

    /// Parses a translatable marker (`_`).
    pub fn parse_translatable_marker(&mut self) -> Result<()> {
        if self.consume(|b| b == b'_').is_none() {
            return Err(Error::ExpectedTranslatable);
        }
        self.space();
        Ok(())
    }

    /// Parses a string.
    pub fn parse_string(&mut self) -> Result<Vec<u8>> {
        if self.next().ok_or(Error::EofWhileParsingString)? != b'"' {
            return Err(Error::ExpectedString);
        }
        let mut result = Vec::new();
        loop {
            match self.next().ok_or(Error::EofWhileParsingString)? {
                b'"' => {
                    if self.consume(|b| b == b'"').is_some() {
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
        let result = de.parse_translatable_marker().unwrap_err();
        assert_eq!(result, Error::ExpectedTranslatable);
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
