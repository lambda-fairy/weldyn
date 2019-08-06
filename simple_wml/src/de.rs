use crate::error::{Error, Result};
use crate::parser::Parser;

pub struct AttributeDeserializer<'de> {
    parser: Parser<'de>,
    last_key: Vec<u8>,
}

impl<'de> AttributeDeserializer<'de> {
    pub fn new(input: &'de [u8]) -> Self {
        let mut parser = Parser::new(input);
        parser.space();
        AttributeDeserializer {
            parser,
            last_key: Vec::new(),
        }
    }

    pub fn next(mut self) -> Result<AttributeResult<'de>> {
        if let Ok((key, value)) = self.parser.parse_attribute() {
            // TODO check more errors
            if self.last_key >= key {
                return Err(Error::AttributeOrder);
            }
            self.last_key.clear();
            self.last_key.extend(&key);
            Ok(AttributeResult::Continue { key, value, de: self })
        } else {
            Ok(AttributeResult::Children { de: ChildrenDeserializer::new(self.parser) })
        }
    }
}

pub enum AttributeResult<'de> {
    Continue {
        key: Vec<u8>,
        value: Vec<u8>,
        de: AttributeDeserializer<'de>,
    },
    Children {
        de: ChildrenDeserializer<'de>,
    },
}

pub struct ChildrenDeserializer<'de> {
    parser: Parser<'de>,
}

impl<'de> ChildrenDeserializer<'de> {
    fn new(parser: Parser<'de>) -> Self {
        ChildrenDeserializer { parser }
    }
}
