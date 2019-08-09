use crate::parser::{Parser, Token};

pub fn accept_attributes<'de>(
    parser: &mut Parser<'de>,
    mut visitor: impl AttributeVisitor<'de>,
    outer_open_key: Option<&[u8]>
) -> Option<()> {
    let mut last_key: Vec<u8> = Vec::new();
    loop {
        match parser.next() {
            Some(Token::Attr { key, value }) => {
                if last_key >= key {
                    return None;
                }
                last_key.clear();
                last_key.extend(&key);
                visitor.visit_attribute(key, value)?;
            }
            Some(Token::Open { open_key: first_open_key }) => {
                return accept_children(
                    parser,
                    visitor.start_children(),
                    first_open_key,
                    outer_open_key,
                );
            }
            // TODO duplication
            Some(Token::Close { close_key }) => {
                return outer_open_key
                    .filter(|&open_key| open_key == close_key.as_slice())
                    .map(drop);
            }
            None => {
                return if outer_open_key.is_some() { None } else { Some(()) };
            }
        }
    }
}

pub fn accept_children<'de>(
    parser: &mut Parser<'de>,
    mut visitor: impl ChildrenVisitor<'de>,
    first_open_key: Vec<u8>,
    outer_open_key: Option<&[u8]>,
) -> Option<()> {
    accept_child(parser, &first_open_key, &mut visitor)?;
    loop {
        match parser.next() {
            Some(Token::Attr { .. }) => {
                return None;
            }
            Some(Token::Open { open_key }) => {
                accept_child(parser, &open_key, &mut visitor)?;
            }
            // TODO duplication
            Some(Token::Close { close_key }) => {
                return outer_open_key
                    .filter(|&open_key| open_key == close_key.as_slice())
                    .map(drop);
            }
            None => {
                return if outer_open_key.is_some() { None } else { Some(()) };
            }
        }
    }
}

fn accept_child<'de>(
    parser: &mut Parser<'de>,
    open_key: &[u8],
    visitor: &mut impl ChildrenVisitor<'de>,
) -> Option<()> {
    accept_attributes(parser, visitor.visit_child(&open_key)?, Some(&open_key))
}

pub trait AttributeVisitor<'de> {
    type ChildrenVisitor: ChildrenVisitor<'de>;
    fn visit_attribute(&mut self, key: Vec<u8>, value: Vec<u8>) -> Option<()>;
    fn start_children(self) -> Self::ChildrenVisitor;
}

pub trait ChildrenVisitor<'de> {
    type AttributeVisitor: AttributeVisitor<'de>;
    fn visit_child(&mut self, open_key: &[u8]) -> Option<Self::AttributeVisitor>;
}
