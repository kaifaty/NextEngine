use std::collections::BTreeMap;

use super::ManifestCodecError;
use crate::{CanonicalDecodeLimits, ContentHash, content_hash_from_bytes};

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum JcsValue {
    String(String),
    Number(u64),
    Array(Vec<JcsValue>),
    Object(BTreeMap<String, JcsValue>),
}

pub(crate) fn encode_canonical_jcs(value: &JcsValue) -> Vec<u8> {
    encode_value(value).into_bytes()
}

pub(crate) fn decode_canonical_jcs(
    bytes: &[u8],
    limits: CanonicalDecodeLimits,
) -> Result<JcsValue, ManifestCodecError> {
    if bytes.len() > limits.max_total_bytes {
        return Err(ManifestCodecError::InputTooLarge {
            actual: bytes.len(),
            limit: limits.max_total_bytes,
        });
    }
    let mut parser = Parser::new(bytes, limits.max_sequence_items);
    let value = parser.parse_value(0)?;
    parser.finish()?;
    if encode_value(&value).as_bytes() != bytes {
        return Err(ManifestCodecError::NonCanonicalJcs);
    }
    Ok(value)
}

pub(super) fn string(value: impl Into<String>) -> JcsValue {
    JcsValue::String(value.into())
}

pub(super) fn take(
    object: &mut BTreeMap<String, JcsValue>,
    field: &'static str,
) -> Result<JcsValue, ManifestCodecError> {
    object
        .remove(field)
        .ok_or_else(|| ManifestCodecError::MissingField(field.to_owned()))
}

pub(super) fn next(
    values: &mut impl Iterator<Item = JcsValue>,
    field: &'static str,
) -> Result<JcsValue, ManifestCodecError> {
    values
        .next()
        .ok_or_else(|| ManifestCodecError::MissingField(field.to_owned()))
}

pub(super) fn ensure_no_more(
    mut values: impl Iterator<Item = JcsValue>,
    field: &'static str,
) -> Result<(), ManifestCodecError> {
    if values.next().is_some() {
        Err(ManifestCodecError::UnknownField(field.to_owned()))
    } else {
        Ok(())
    }
}

pub(super) fn into_object(
    value: JcsValue,
    field: &'static str,
) -> Result<BTreeMap<String, JcsValue>, ManifestCodecError> {
    if let JcsValue::Object(object) = value {
        Ok(object)
    } else {
        Err(ManifestCodecError::WrongType {
            field: field.to_owned(),
            expected: "an object",
        })
    }
}

pub(super) fn into_array(
    value: JcsValue,
    field: &'static str,
) -> Result<Vec<JcsValue>, ManifestCodecError> {
    if let JcsValue::Array(array) = value {
        Ok(array)
    } else {
        Err(ManifestCodecError::WrongType {
            field: field.to_owned(),
            expected: "an array",
        })
    }
}

pub(super) fn into_string(
    value: JcsValue,
    field: &'static str,
) -> Result<String, ManifestCodecError> {
    if let JcsValue::String(value) = value {
        Ok(value)
    } else {
        Err(ManifestCodecError::WrongType {
            field: field.to_owned(),
            expected: "a string",
        })
    }
}

pub(super) fn decode_u32(value: JcsValue, field: &'static str) -> Result<u32, ManifestCodecError> {
    let JcsValue::Number(value) = value else {
        return Err(ManifestCodecError::WrongType {
            field: field.to_owned(),
            expected: "an unsigned JSON integer",
        });
    };
    u32::try_from(value).map_err(|_| ManifestCodecError::InvalidInteger(field.to_owned()))
}

pub(super) fn decode_u64_string(
    value: JcsValue,
    field: &'static str,
) -> Result<u64, ManifestCodecError> {
    let value = into_string(value, field)?;
    if value.is_empty()
        || (value.len() > 1 && value.starts_with('0'))
        || !value.bytes().all(|byte| byte.is_ascii_digit())
    {
        return Err(ManifestCodecError::InvalidInteger(field.to_owned()));
    }
    value
        .parse()
        .map_err(|_| ManifestCodecError::InvalidInteger(field.to_owned()))
}

pub(super) fn decode_i64_string(
    value: JcsValue,
    field: &'static str,
) -> Result<i64, ManifestCodecError> {
    let value = into_string(value, field)?;
    validate_signed_integer_string(&value, field)?;
    value
        .parse()
        .map_err(|_| ManifestCodecError::InvalidInteger(field.to_owned()))
}

pub(super) fn decode_i32_string(
    value: JcsValue,
    field: &'static str,
) -> Result<i32, ManifestCodecError> {
    let value = into_string(value, field)?;
    validate_signed_integer_string(&value, field)?;
    value
        .parse()
        .map_err(|_| ManifestCodecError::InvalidInteger(field.to_owned()))
}

fn validate_signed_integer_string(
    value: &str,
    field: &'static str,
) -> Result<(), ManifestCodecError> {
    let digits = value.strip_prefix('-').unwrap_or(value);
    if digits.is_empty()
        || (digits.len() > 1 && digits.starts_with('0'))
        || (value.starts_with('-') && digits == "0")
        || !digits.bytes().all(|byte| byte.is_ascii_digit())
    {
        return Err(ManifestCodecError::InvalidInteger(field.to_owned()));
    }
    Ok(())
}

pub(super) fn decode_hash(
    value: JcsValue,
    field: &'static str,
) -> Result<ContentHash, ManifestCodecError> {
    Ok(content_hash_from_bytes(decode_fixed_hex::<32>(
        value, field,
    )?))
}

pub(super) fn decode_fixed_hex<const LENGTH: usize>(
    value: JcsValue,
    field: &'static str,
) -> Result<[u8; LENGTH], ManifestCodecError> {
    let bytes = decode_hex(value, field)?;
    bytes
        .try_into()
        .map_err(|_| ManifestCodecError::InvalidHex(field.to_owned()))
}

pub(super) fn decode_hex(
    value: JcsValue,
    field: &'static str,
) -> Result<Vec<u8>, ManifestCodecError> {
    let value = into_string(value, field)?;
    if !value.len().is_multiple_of(2)
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        return Err(ManifestCodecError::InvalidHex(field.to_owned()));
    }
    let mut output = Vec::with_capacity(value.len() / 2);
    for pair in value.as_bytes().chunks_exact(2) {
        output.push((hex_nibble(pair[0]) << 4) | hex_nibble(pair[1]));
    }
    Ok(output)
}

pub(super) fn hex_bytes(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut encoded = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        encoded.push(char::from(HEX[usize::from(byte >> 4)]));
        encoded.push(char::from(HEX[usize::from(byte & 0x0f)]));
    }
    encoded
}

pub(super) fn encode_optional_id_hex<const LENGTH: usize>(
    value: Option<&[u8; LENGTH]>,
) -> JcsValue {
    match value {
        Some(value) => JcsValue::Array(vec![string(hex_bytes(value))]),
        None => JcsValue::Array(Vec::new()),
    }
}

pub(super) fn decode_optional_id_hex<const LENGTH: usize>(
    value: JcsValue,
    field: &'static str,
) -> Result<Option<[u8; LENGTH]>, ManifestCodecError> {
    let values = into_array(value, field)?;
    match values.as_slice() {
        [] => Ok(None),
        [_] => Ok(Some(decode_fixed_hex::<LENGTH>(
            values.into_iter().next().expect("single value"),
            field,
        )?)),
        _ => Err(ManifestCodecError::InvalidHex(field.to_owned())),
    }
}

fn hex_nibble(byte: u8) -> u8 {
    match byte {
        b'0'..=b'9' => byte - b'0',
        b'a'..=b'f' => byte - b'a' + 10,
        _ => 0,
    }
}

pub(super) fn encode_value(value: &JcsValue) -> String {
    match value {
        JcsValue::String(value) => encode_string(value),
        JcsValue::Number(value) => value.to_string(),
        JcsValue::Array(values) => {
            let mut output = String::from("[");
            for (index, value) in values.iter().enumerate() {
                if index != 0 {
                    output.push(',');
                }
                output.push_str(&encode_value(value));
            }
            output.push(']');
            output
        }
        JcsValue::Object(object) => {
            let mut output = String::from("{");
            for (index, (key, value)) in object.iter().enumerate() {
                if index != 0 {
                    output.push(',');
                }
                output.push_str(&encode_string(key));
                output.push(':');
                output.push_str(&encode_value(value));
            }
            output.push('}');
            output
        }
    }
}

fn encode_string(value: &str) -> String {
    let mut output = String::with_capacity(value.len() + 2);
    output.push('"');
    for character in value.chars() {
        match character {
            '"' => output.push_str("\\\""),
            '\\' => output.push_str("\\\\"),
            '\u{08}' => output.push_str("\\b"),
            '\t' => output.push_str("\\t"),
            '\n' => output.push_str("\\n"),
            '\u{0c}' => output.push_str("\\f"),
            '\r' => output.push_str("\\r"),
            '\u{00}'..='\u{1f}' => {
                output.push_str(&format!("\\u{:04x}", u32::from(character)));
            }
            _ => output.push(character),
        }
    }
    output.push('"');
    output
}

pub(super) struct Parser<'a> {
    bytes: &'a [u8],
    position: usize,
    item_limit: usize,
    remaining_items: usize,
}

impl<'a> Parser<'a> {
    pub(super) const fn new(bytes: &'a [u8], max_items: usize) -> Self {
        Self {
            bytes,
            position: 0,
            item_limit: max_items,
            remaining_items: max_items,
        }
    }

    pub(super) fn parse_value(&mut self, depth: usize) -> Result<JcsValue, ManifestCodecError> {
        if depth > 32 {
            return Err(ManifestCodecError::NestingTooDeep);
        }
        match self.peek()? {
            b'"' => Ok(JcsValue::String(self.parse_string()?)),
            b'[' => self.parse_array(depth + 1),
            b'{' => self.parse_object(depth + 1),
            b'0'..=b'9' => self.parse_number(),
            _ => Err(ManifestCodecError::InvalidSyntax),
        }
    }

    fn parse_array(&mut self, depth: usize) -> Result<JcsValue, ManifestCodecError> {
        self.expect(b'[')?;
        let mut values = Vec::new();
        if self.consume(b']') {
            return Ok(JcsValue::Array(values));
        }
        loop {
            self.consume_item()?;
            values.push(self.parse_value(depth)?);
            if self.consume(b']') {
                break;
            }
            self.expect(b',')?;
        }
        Ok(JcsValue::Array(values))
    }

    fn parse_object(&mut self, depth: usize) -> Result<JcsValue, ManifestCodecError> {
        self.expect(b'{')?;
        let mut object = BTreeMap::new();
        if self.consume(b'}') {
            return Ok(JcsValue::Object(object));
        }
        loop {
            self.consume_item()?;
            let key = self.parse_string()?;
            self.expect(b':')?;
            let value = self.parse_value(depth)?;
            if object.insert(key.clone(), value).is_some() {
                return Err(ManifestCodecError::DuplicateObjectKey(key));
            }
            if self.consume(b'}') {
                break;
            }
            self.expect(b',')?;
        }
        Ok(JcsValue::Object(object))
    }

    fn parse_string(&mut self) -> Result<String, ManifestCodecError> {
        self.expect(b'"')?;
        let mut output = String::new();
        loop {
            let byte = self.peek()?;
            match byte {
                b'"' => {
                    self.position += 1;
                    return Ok(output);
                }
                b'\\' => {
                    self.position += 1;
                    self.parse_escape(&mut output)?;
                }
                0x00..=0x1f => return Err(ManifestCodecError::InvalidSyntax),
                0x20..=0x7f => {
                    output.push(char::from(byte));
                    self.position += 1;
                }
                _ => {
                    let remaining = std::str::from_utf8(&self.bytes[self.position..])
                        .map_err(|_| ManifestCodecError::InvalidUtf8)?;
                    let character = remaining
                        .chars()
                        .next()
                        .ok_or(ManifestCodecError::UnexpectedEnd)?;
                    output.push(character);
                    self.position += character.len_utf8();
                }
            }
        }
    }

    fn parse_escape(&mut self, output: &mut String) -> Result<(), ManifestCodecError> {
        let escaped = self.take()?;
        match escaped {
            b'"' => output.push('"'),
            b'\\' => output.push('\\'),
            b'/' => output.push('/'),
            b'b' => output.push('\u{08}'),
            b'f' => output.push('\u{0c}'),
            b'n' => output.push('\n'),
            b'r' => output.push('\r'),
            b't' => output.push('\t'),
            b'u' => {
                let first = self.parse_hex_quad()?;
                let scalar = if (0xd800..=0xdbff).contains(&first) {
                    self.expect(b'\\')?;
                    self.expect(b'u')?;
                    let second = self.parse_hex_quad()?;
                    if !(0xdc00..=0xdfff).contains(&second) {
                        return Err(ManifestCodecError::InvalidUnicodeEscape);
                    }
                    0x1_0000 + ((u32::from(first) - 0xd800) << 10) + (u32::from(second) - 0xdc00)
                } else if (0xdc00..=0xdfff).contains(&first) {
                    return Err(ManifestCodecError::InvalidUnicodeEscape);
                } else {
                    u32::from(first)
                };
                output
                    .push(char::from_u32(scalar).ok_or(ManifestCodecError::InvalidUnicodeEscape)?);
            }
            _ => return Err(ManifestCodecError::InvalidEscape),
        }
        Ok(())
    }

    fn parse_hex_quad(&mut self) -> Result<u16, ManifestCodecError> {
        let mut value = 0_u16;
        for _ in 0..4 {
            let byte = self.take()?;
            let nibble = match byte {
                b'0'..=b'9' => u16::from(byte - b'0'),
                b'a'..=b'f' => u16::from(byte - b'a' + 10),
                b'A'..=b'F' => u16::from(byte - b'A' + 10),
                _ => return Err(ManifestCodecError::InvalidUnicodeEscape),
            };
            value = (value << 4) | nibble;
        }
        Ok(value)
    }

    fn parse_number(&mut self) -> Result<JcsValue, ManifestCodecError> {
        let start = self.position;
        while self
            .bytes
            .get(self.position)
            .is_some_and(u8::is_ascii_digit)
        {
            self.position += 1;
        }
        let value = std::str::from_utf8(&self.bytes[start..self.position])
            .map_err(|_| ManifestCodecError::InvalidUtf8)?
            .parse()
            .map_err(|_| ManifestCodecError::InvalidSyntax)?;
        Ok(JcsValue::Number(value))
    }

    fn consume_item(&mut self) -> Result<(), ManifestCodecError> {
        if self.remaining_items == 0 {
            return Err(ManifestCodecError::TooManyItems {
                limit: self.item_limit,
            });
        }
        self.remaining_items -= 1;
        Ok(())
    }

    fn peek(&self) -> Result<u8, ManifestCodecError> {
        self.bytes
            .get(self.position)
            .copied()
            .ok_or(ManifestCodecError::UnexpectedEnd)
    }

    fn take(&mut self) -> Result<u8, ManifestCodecError> {
        let value = self.peek()?;
        self.position += 1;
        Ok(value)
    }

    fn expect(&mut self, expected: u8) -> Result<(), ManifestCodecError> {
        if self.take()? == expected {
            Ok(())
        } else {
            Err(ManifestCodecError::InvalidSyntax)
        }
    }

    fn consume(&mut self, expected: u8) -> bool {
        if self.bytes.get(self.position) == Some(&expected) {
            self.position += 1;
            true
        } else {
            false
        }
    }

    pub(super) fn finish(self) -> Result<(), ManifestCodecError> {
        if self.position == self.bytes.len() {
            Ok(())
        } else {
            Err(ManifestCodecError::InvalidSyntax)
        }
    }
}
