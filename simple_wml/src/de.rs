use crate::token::{Token, Tokens};

pub fn from_slice<'de>(
    input: &'de [u8],
    visitor: impl AttributeVisitor<'de>,
) -> Option<()> {
    let mut tokens = Tokens::new(input);
    accept(&mut tokens, visitor, None)?;
    tokens.assert_end()?;
    Some(())
}

enum State<'de, A: AttributeVisitor<'de>> {
    Attributes { visitor: A, last_key: Vec<u8> },
    Children { visitor: A::ChildrenVisitor },
}

fn accept<'de>(
    tokens: &mut Tokens<'de>,
    visitor: impl AttributeVisitor<'de>,
    outer_open_key: Option<&[u8]>,
) -> Option<()> {
    let mut state = State::Attributes { visitor, last_key: Vec::new() };
    loop {
        match tokens.next() {
            Some(Token::Attr { key, value }) => {
                if let State::Attributes { visitor, last_key } = &mut state {
                    if *last_key >= key {
                        return None;
                    }
                    last_key.clear();
                    last_key.extend(&key);
                    visitor.visit_attribute(key, value)?;
                } else {
                    return None;
                }
            }
            Some(Token::Open { open_key }) => {
                state = match state {
                    State::Attributes { visitor, .. } => {
                        let mut visitor = visitor.start_children();
                        accept_child(tokens, &open_key, &mut visitor)?;
                        State::Children { visitor }
                    }
                    State::Children { mut visitor } => {
                        accept_child(tokens, &open_key, &mut visitor)?;
                        State::Children { visitor }
                    }
                };
            }
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
    tokens: &mut Tokens<'de>,
    open_key: &[u8],
    visitor: &mut impl ChildrenVisitor<'de>,
) -> Option<()> {
    let visitor = visitor.visit_child(&open_key)?;
    accept(tokens, visitor, Some(&open_key))
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
