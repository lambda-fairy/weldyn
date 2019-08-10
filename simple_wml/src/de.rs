use crate::parser::Parser;

pub fn from_slice<'de, T>(
    input: &'de [u8],
    action: impl FnOnce(AttributeDeserializer<'_, 'de>) -> Option<T>,
) -> Option<T> {
    let mut parser = Parser::new(input);
    let result = action(AttributeDeserializer::new(&mut parser))?;
    parser.assert_end()?;
    Some(result)
}

pub struct AttributeDeserializer<'a, 'de: 'a> {
    parser: &'a mut Parser<'de>,
    last_key: Vec<u8>,
}

impl<'a, 'de> AttributeDeserializer<'a, 'de> {
    fn new(parser: &'a mut Parser<'de>) -> Self {
        AttributeDeserializer { parser, last_key: Vec::new() }
    }

    pub fn next_attribute(&mut self) -> Option<(Vec<u8>, Vec<u8>)> {
        let (key, value) = self.parser.next_attribute()?;
        if self.last_key >= key {
            return None;
        }
        self.last_key.clear();
        self.last_key.extend(&key);
        Some((key, value))
    }

    pub fn start_children(self) -> ChildrenDeserializer<'a, 'de> {
        ChildrenDeserializer::new(self.parser)
    }
}

pub struct ChildrenDeserializer<'a, 'de: 'a> {
    parser: &'a mut Parser<'de>,
}

impl<'a, 'de> ChildrenDeserializer<'a, 'de> {
    fn new(parser: &'a mut Parser<'de>) -> Self {
        ChildrenDeserializer { parser }
    }

    pub fn next_child<F, T>(&mut self, action: F) -> Option<T>
    where
        F: FnOnce(&[u8], AttributeDeserializer<'_, 'de>) -> Option<T>,
    {
        let open_key = self.parser.next_open()?;
        let result = action(&open_key, AttributeDeserializer::new(self.parser))?;
        let close_key = self.parser.next_close()?;
        if open_key != close_key {
            return None;
        }
        Some(result)
    }
}
