use crate::error::{Error, Result};

pub fn to_vec<'a>(
    output: &'a mut Vec<u8>,
    serialize: impl FnOnce(AttributeSerializer<'a>) -> Result<()>,
) -> Result<()> {
    serialize(AttributeSerializer::new(output))
}

pub struct AttributeSerializer<'a> {
    output: &'a mut Vec<u8>,
    last_key: Vec<u8>,
}

impl<'a> AttributeSerializer<'a> {
    fn new(output: &'a mut Vec<u8>) -> Self {
        AttributeSerializer {
            output,
            last_key: Vec::new(),
        }
    }

    pub fn push(&mut self, key: &[u8], value: &[u8]) -> Result<()> {
        if self.last_key.as_slice() >= key {
            return Err(Error::AttributeOrder);
        }
        self.last_key.clear();
        self.last_key.extend(key);

        self.output.extend(key);
        self.output.extend(b"=\"");

        for &b in value {
            self.output.push(b);
            if b == b'"' {
                self.output.push(b'"');
            }
        }

        self.output.extend(b"\"\n");

        Ok(())
    }

    pub fn start_children(self) -> ChildrenSerializer<'a> {
        ChildrenSerializer::new(self.output)
    }
}

pub struct ChildrenSerializer<'a> {
    output: &'a mut Vec<u8>,
}

impl<'a> ChildrenSerializer<'a> {
    fn new(output: &'a mut Vec<u8>) -> Self {
        ChildrenSerializer { output }
    }

    pub fn push(
        &mut self,
        name: &[u8],
        serialize: impl FnOnce(AttributeSerializer<'_>) -> Result<()>,
    ) -> Result<()> {
        self.output.push(b'[');
        self.output.extend(name);
        self.output.extend(b"]\n");

        serialize(AttributeSerializer::new(self.output))?;

        self.output.extend(b"[/");
        self.output.extend(name);
        self.output.extend(b"]\n");

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simple() {
        let mut output = Vec::new();

        to_vec(&mut output, |mut attrs| {
            attrs.push(b"baz", b"quux")?;
            attrs.push(b"foo", b"bar \"bar\"")?;

            let mut children = attrs.start_children();
            children.push(b"user", |mut attrs| {
                attrs.push(b"name", b"Li'sar")?;
                Ok(())
            })?;
            children.push(b"user", |mut attrs| {
                attrs.push(b"name", b"Konrad")?;
                Ok(())
            })?;

            Ok(())
        }).unwrap();

        assert_eq!(output, &br#"baz="quux"
foo="bar ""bar"""
[user]
name="Li'sar"
[/user]
[user]
name="Konrad"
[/user]
"#[..]);
    }
}
