use std::fmt;

use crate::doc::core::RationalTime;

use super::key::NodeKey;

/// Why a value could not be represented in a stable node identity.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CanonicalError {
    NonFiniteF32,
    NonFiniteF64,
    LengthOverflow,
}

impl fmt::Display for CanonicalError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NonFiniteF32 => write!(f, "canonical f32 must be finite"),
            Self::NonFiniteF64 => write!(f, "canonical f64 must be finite"),
            Self::LengthOverflow => write!(f, "canonical value is too long"),
        }
    }
}

impl std::error::Error for CanonicalError {}

// Tags are part of the identity format. Never reuse one for a different type.
#[repr(u8)]
enum Tag {
    Bool = 1,
    U8 = 2,
    U16 = 3,
    U32 = 4,
    U64 = 5,
    I8 = 6,
    I16 = 7,
    I32 = 8,
    I64 = 9,
    F32 = 10,
    F64 = 11,
    String = 12,
    Bytes = 13,
    RationalTime = 14,
    NodeKey = 15,
    Parameters = 16,
    SourceVersions = 17,
    NodeInputs = 18,
}

/// A self-delimiting, platform-independent byte encoding for node identity.
///
/// Integers and float bit patterns are big-endian. Variable-length fields use
/// an unsigned 64-bit byte/item count, so concatenated fields are unambiguous.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct CanonicalEncoder {
    bytes: Vec<u8>,
}

impl CanonicalEncoder {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn as_bytes(&self) -> &[u8] {
        &self.bytes
    }
    pub fn into_bytes(self) -> Vec<u8> {
        self.bytes
    }
    pub fn bool(&mut self, value: bool) -> &mut Self {
        self.tag(Tag::Bool);
        self.bytes.push(u8::from(value));
        self
    }
    pub fn u8(&mut self, value: u8) -> &mut Self {
        self.tag(Tag::U8);
        self.bytes.push(value);
        self
    }
    pub fn u16(&mut self, value: u16) -> &mut Self {
        self.fixed(Tag::U16, &value.to_be_bytes())
    }
    pub fn u32(&mut self, value: u32) -> &mut Self {
        self.fixed(Tag::U32, &value.to_be_bytes())
    }
    pub fn u64(&mut self, value: u64) -> &mut Self {
        self.fixed(Tag::U64, &value.to_be_bytes())
    }

    pub fn i8(&mut self, value: i8) -> &mut Self {
        self.fixed(Tag::I8, &value.to_be_bytes())
    }

    pub fn i16(&mut self, value: i16) -> &mut Self {
        self.fixed(Tag::I16, &value.to_be_bytes())
    }

    pub fn i32(&mut self, value: i32) -> &mut Self {
        self.fixed(Tag::I32, &value.to_be_bytes())
    }

    pub fn i64(&mut self, value: i64) -> &mut Self {
        self.fixed(Tag::I64, &value.to_be_bytes())
    }

    pub fn f32(&mut self, value: f32) -> Result<&mut Self, CanonicalError> {
        if !value.is_finite() {
            return Err(CanonicalError::NonFiniteF32);
        }
        let normalized = if value == 0.0 { 0.0 } else { value };
        Ok(self.fixed(Tag::F32, &normalized.to_bits().to_be_bytes()))
    }

    pub fn f64(&mut self, value: f64) -> Result<&mut Self, CanonicalError> {
        if !value.is_finite() {
            return Err(CanonicalError::NonFiniteF64);
        }
        let normalized = if value == 0.0 { 0.0 } else { value };
        Ok(self.fixed(Tag::F64, &normalized.to_bits().to_be_bytes()))
    }

    pub fn string(&mut self, value: &str) -> Result<&mut Self, CanonicalError> {
        self.variable(Tag::String, value.as_bytes())
    }

    pub fn bytes(&mut self, value: &[u8]) -> Result<&mut Self, CanonicalError> {
        self.variable(Tag::Bytes, value)
    }

    pub fn rational_time(&mut self, value: RationalTime) -> &mut Self {
        self.tag(Tag::RationalTime);
        self.bytes.extend_from_slice(&value.num().to_be_bytes());
        self.bytes.extend_from_slice(&value.den().to_be_bytes());
        self
    }

    pub fn node_key(&mut self, value: NodeKey) -> &mut Self {
        self.fixed(Tag::NodeKey, &value.as_u64().to_be_bytes())
    }

    pub fn parameters(&mut self, value: &[u8]) -> Result<&mut Self, CanonicalError> {
        self.variable(Tag::Parameters, value)
    }

    pub fn source_versions(&mut self, versions: &[u64]) -> Result<&mut Self, CanonicalError> {
        self.tagged_u64s(Tag::SourceVersions, versions.iter().copied())
    }

    pub fn node_inputs(&mut self, inputs: &[NodeKey]) -> Result<&mut Self, CanonicalError> {
        self.tagged_u64s(Tag::NodeInputs, inputs.iter().map(|key| key.as_u64()))
    }

    fn tag(&mut self, tag: Tag) {
        self.bytes.push(tag as u8);
    }

    fn fixed(&mut self, tag: Tag, value: &[u8]) -> &mut Self {
        self.tag(tag);
        self.bytes.extend_from_slice(value);
        self
    }

    fn variable(&mut self, tag: Tag, value: &[u8]) -> Result<&mut Self, CanonicalError> {
        let len = u64::try_from(value.len()).map_err(|_| CanonicalError::LengthOverflow)?;
        self.tag(tag);
        self.bytes.extend_from_slice(&len.to_be_bytes());
        self.bytes.extend_from_slice(value);
        Ok(self)
    }

    fn tagged_u64s(
        &mut self,
        tag: Tag,
        values: impl ExactSizeIterator<Item = u64>,
    ) -> Result<&mut Self, CanonicalError> {
        let len = u64::try_from(values.len()).map_err(|_| CanonicalError::LengthOverflow)?;
        self.tag(tag);
        self.bytes.extend_from_slice(&len.to_be_bytes());
        for value in values {
            self.bytes.extend_from_slice(&value.to_be_bytes());
        }
        Ok(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::frame_graph::key::{NodeIdentity, NodeKind};

    fn key(parameter: u8) -> NodeKey {
        let mut identity = NodeIdentity::new(NodeKind::TextContent, Vec::new());
        identity.parameters.push(parameter);
        NodeKey::for_identity(&identity)
    }

    #[test]
    fn integers_are_tagged_and_big_endian() {
        let mut encoded = CanonicalEncoder::new();
        encoded.bool(true).u16(0x1234).i32(-2);
        assert_eq!(
            encoded.as_bytes(),
            &[
                Tag::Bool as u8,
                1,
                Tag::U16 as u8,
                0x12,
                0x34,
                Tag::I32 as u8,
                0xff,
                0xff,
                0xff,
                0xfe
            ]
        );
    }

    #[test]
    fn variable_fields_are_typed_and_length_delimited() {
        let mut strings = CanonicalEncoder::new();
        strings.string("ab").unwrap().string("c").unwrap();
        let mut bytes = CanonicalEncoder::new();
        bytes.bytes(b"ab").unwrap().bytes(b"c").unwrap();
        assert_ne!(strings.as_bytes(), bytes.as_bytes());
        assert_eq!(&strings.as_bytes()[1..9], &2u64.to_be_bytes());
    }

    #[test]
    fn rational_time_and_identity_lists_keep_exact_components() {
        let time = RationalTime::try_new(-3, 5).unwrap();
        let inputs = [key(1), key(2)];
        let mut encoded = CanonicalEncoder::new();
        encoded
            .rational_time(time)
            .source_versions(&[7, 9])
            .unwrap()
            .node_inputs(&inputs)
            .unwrap();
        assert!(encoded
            .as_bytes()
            .windows(8)
            .any(|b| b == (-3i64).to_be_bytes()));
        assert!(encoded
            .as_bytes()
            .windows(8)
            .any(|b| b == 5i64.to_be_bytes()));
        assert!(encoded
            .as_bytes()
            .ends_with(&inputs[1].as_u64().to_be_bytes()));
    }

    #[test]
    fn negative_zero_is_zero_and_non_finite_values_do_not_mutate() {
        let mut positive = CanonicalEncoder::new();
        let mut negative = CanonicalEncoder::new();
        positive.f32(0.0).unwrap().f64(0.0).unwrap();
        negative.f32(-0.0).unwrap().f64(-0.0).unwrap();
        assert_eq!(positive, negative);

        let before = positive.clone();
        assert!(matches!(
            positive.f32(f32::NAN),
            Err(CanonicalError::NonFiniteF32)
        ));
        assert!(matches!(
            positive.f64(f64::INFINITY),
            Err(CanonicalError::NonFiniteF64)
        ));
        assert_eq!(positive, before);
    }
}
