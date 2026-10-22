//! A small XML reader and writer helper for the interchange formats.
//!
//! Microsoft Project writes a very regular dialect of XML: elements, text and
//! attributes, with no mixed content in the fields Cadence cares about. This
//! module implements just enough of XML 1.0 to read that dialect safely —
//! processing instructions and comments are skipped, predefined entities and
//! numeric character references are decoded, and mismatched or unterminated
//! tags are rejected. Keeping the reader here avoids pulling a full XML stack
//! (and its dependency tree) into a pure-Rust, offline application.

use crate::InterchangeError;

/// One element of a parsed document.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Element {
    /// Element name as written, without any namespace prefix.
    pub name: String,
    /// Text found directly inside the element, with entities decoded.
    pub text: String,
    /// Child elements in document order.
    pub children: Vec<Element>,
}

impl Element {
    fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            text: String::new(),
            children: Vec::new(),
        }
    }

    /// First child element with the given name.
    pub fn child(&self, name: &str) -> Option<&Element> {
        self.children.iter().find(|child| child.name == name)
    }

    /// Every child element with the given name, in document order.
    pub fn children_named(&self, name: &'static str) -> impl Iterator<Item = &Element> {
        self.children.iter().filter(move |child| child.name == name)
    }

    /// Text directly inside the element, with surrounding whitespace removed.
    pub fn text_trimmed(&self) -> &str {
        self.text.trim()
    }

    /// Trimmed text of the first child with the given name.
    pub fn text_of(&self, name: &str) -> Option<&str> {
        self.child(name).map(Element::text_trimmed)
    }

    /// Parse the trimmed text of the first child with the given name.
    pub fn parse_of<T: std::str::FromStr>(&self, name: &str) -> Option<T> {
        self.text_of(name)?.parse().ok()
    }
}

/// Parse an XML document and return its root element.
pub fn parse_document(input: &str) -> Result<Element, InterchangeError> {
    build_tree(tokenize(input)?)
}

/// Escape text for use in element content or an attribute value.
pub fn escape(text: &str) -> String {
    let mut escaped = String::with_capacity(text.len());
    for character in text.chars() {
        match character {
            '&' => escaped.push_str("&amp;"),
            '<' => escaped.push_str("&lt;"),
            '>' => escaped.push_str("&gt;"),
            '"' => escaped.push_str("&quot;"),
            '\'' => escaped.push_str("&apos;"),
            other => escaped.push(other),
        }
    }
    escaped
}

enum Token {
    Open(String),
    Empty(String),
    Close(String),
    Text(String),
}

fn malformed(message: impl Into<String>) -> InterchangeError {
    InterchangeError::Xml {
        message: message.into(),
    }
}

fn tokenize(input: &str) -> Result<Vec<Token>, InterchangeError> {
    let mut tokens = Vec::new();
    let mut position = 0usize;

    while position < input.len() {
        let rest = &input[position..];

        if rest.starts_with("<?") {
            let end = rest
                .find("?>")
                .ok_or_else(|| malformed("unterminated processing instruction"))?;
            position += end + 2;
        } else if rest.starts_with("<!--") {
            let end = rest
                .find("-->")
                .ok_or_else(|| malformed("unterminated comment"))?;
            position += end + 3;
        } else if rest.starts_with("<!") {
            let end = rest
                .find('>')
                .ok_or_else(|| malformed("unterminated declaration"))?;
            position += end + 1;
        } else if rest.starts_with("</") {
            let end = rest
                .find('>')
                .ok_or_else(|| malformed("unterminated closing tag"))?;
            let name = rest[2..end].trim().to_string();
            if name.is_empty() {
                return Err(malformed("closing tag without a name"));
            }
            tokens.push(Token::Close(name));
            position += end + 1;
        } else if rest.starts_with('<') {
            let end = tag_end(rest)?;
            let inner = rest[1..end].trim();
            if inner.is_empty() {
                return Err(malformed("tag without a name"));
            }
            let (inner, empty) = match inner.strip_suffix('/') {
                Some(stripped) => (stripped.trim_end(), true),
                None => (inner, false),
            };
            let name = inner
                .split(|character: char| character.is_whitespace())
                .next()
                .unwrap_or_default()
                .to_string();
            if name.is_empty() {
                return Err(malformed("tag without a name"));
            }
            tokens.push(if empty {
                Token::Empty(name)
            } else {
                Token::Open(name)
            });
            position += end + 1;
        } else {
            let end = rest.find('<').unwrap_or(rest.len());
            let text = decode_entities(&rest[..end]);
            if !text.trim().is_empty() {
                tokens.push(Token::Text(text));
            }
            position += end;
        }
    }

    Ok(tokens)
}

/// Find the index of the `>` that closes a tag, ignoring `>` inside quotes.
fn tag_end(rest: &str) -> Result<usize, InterchangeError> {
    let mut quote = None;
    for (index, character) in rest.char_indices() {
        match quote {
            Some(open) => {
                if character == open {
                    quote = None;
                }
            }
            None => match character {
                '"' | '\'' => quote = Some(character),
                '>' => return Ok(index),
                _ => {}
            },
        }
    }
    Err(malformed("unterminated tag"))
}

fn build_tree(tokens: Vec<Token>) -> Result<Element, InterchangeError> {
    let mut stack = vec![Element::new("#document")];

    for token in tokens {
        match token {
            Token::Open(name) => stack.push(Element::new(name)),
            Token::Empty(name) => {
                let parent = stack
                    .last_mut()
                    .ok_or_else(|| malformed("document has no root"))?;
                parent.children.push(Element::new(name));
            }
            Token::Text(text) => {
                let current = stack
                    .last_mut()
                    .ok_or_else(|| malformed("document has no root"))?;
                current.text.push_str(&text);
            }
            Token::Close(name) => {
                if stack.len() < 2 {
                    return Err(malformed(format!("unexpected closing tag </{name}>")));
                }
                let element = stack.pop().expect("stack has at least two elements");
                if element.name != name {
                    return Err(malformed(format!(
                        "closing tag </{name}> does not match <{}>",
                        element.name
                    )));
                }
                let parent = stack.last_mut().expect("stack has a parent");
                parent.children.push(element);
            }
        }
    }

    if stack.len() != 1 {
        return Err(malformed("document has unclosed elements"));
    }

    let document = stack.pop().expect("stack has the document element");
    let mut roots = document.children.into_iter();
    let root = roots
        .next()
        .ok_or_else(|| malformed("document has no root element"))?;
    if roots.next().is_some() {
        return Err(malformed("document has more than one root element"));
    }
    Ok(root)
}

fn decode_entities(text: &str) -> String {
    let mut decoded = String::with_capacity(text.len());
    let mut rest = text;

    while let Some(index) = rest.find('&') {
        decoded.push_str(&rest[..index]);
        rest = &rest[index..];

        let Some(end) = rest.find(';') else {
            decoded.push_str(rest);
            return decoded;
        };

        let entity = &rest[1..end];
        let replacement = match entity {
            "amp" => Some('&'),
            "lt" => Some('<'),
            "gt" => Some('>'),
            "quot" => Some('"'),
            "apos" => Some('\''),
            _ => {
                if let Some(hex) = entity.strip_prefix("#x").or_else(|| entity.strip_prefix("#X")) {
                    u32::from_str_radix(hex, 16).ok().and_then(char::from_u32)
                } else if let Some(decimal) = entity.strip_prefix('#') {
                    decimal.parse::<u32>().ok().and_then(char::from_u32)
                } else {
                    None
                }
            }
        };

        match replacement {
            Some(character) => {
                decoded.push(character);
                rest = &rest[end + 1..];
            }
            None => {
                decoded.push('&');
                rest = &rest[1..];
            }
        }
    }

    decoded.push_str(rest);
    decoded
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_nested_elements_and_text() {
        let document = parse_document(
            r#"<?xml version="1.0"?>
            <!-- a comment -->
            <Project>
                <Name>Bridge &amp; road</Name>
                <Tasks>
                    <Task><UID>1</UID><Name>Excavate</Name></Task>
                    <Task><UID>2</UID></Task>
                </Tasks>
                <Empty />
            </Project>"#,
        )
        .expect("parse");

        assert_eq!(document.name, "Project");
        assert_eq!(document.text_of("Name"), Some("Bridge & road"));
        let tasks = document.child("Tasks").expect("tasks");
        assert_eq!(tasks.children_named("Task").count(), 2);
        assert_eq!(
            tasks.children_named("Task").next().unwrap().text_of("Name"),
            Some("Excavate")
        );
        assert!(document.child("Empty").is_some());
    }

    #[test]
    fn decodes_character_references() {
        assert_eq!(
            decode_entities("caf&eacute;&#233;&#x41;"),
            "caf&eacute;\u{e9}A"
        );
        assert_eq!(decode_entities("50 &lt; 60 &gt; 40"), "50 < 60 > 40");
    }

    #[test]
    fn unknown_entities_are_kept_verbatim() {
        assert_eq!(decode_entities("a &nbsp; b"), "a &nbsp; b");
        assert_eq!(decode_entities("trailing &"), "trailing &");
    }

    #[test]
    fn attributes_with_angle_brackets_do_not_end_the_tag() {
        let document =
            parse_document(r#"<Project note="a > b"><Name>x</Name></Project>"#).expect("parse");
        assert_eq!(document.text_of("Name"), Some("x"));
    }

    #[test]
    fn mismatched_tags_are_rejected() {
        let error = parse_document("<Project><Name>x</Project></Name>").expect_err("mismatch");
        assert!(matches!(error, InterchangeError::Xml { .. }));
    }

    #[test]
    fn unclosed_tags_are_rejected() {
        let error = parse_document("<Project><Name>x</Name>").expect_err("unclosed");
        assert!(matches!(error, InterchangeError::Xml { .. }));
    }

    #[test]
    fn an_empty_document_is_rejected() {
        let error = parse_document("   ").expect_err("empty");
        assert!(matches!(error, InterchangeError::Xml { .. }));
    }

    #[test]
    fn escaping_round_trips_through_the_reader() {
        let original = "A & B <c> \"d\" 'e'";
        let document = format!(
            "<Project><Name>{}</Name></Project>",
            escape(original)
        );
        let parsed = parse_document(&document).expect("parse");
        assert_eq!(parsed.text_of("Name"), Some(original));
    }
}
