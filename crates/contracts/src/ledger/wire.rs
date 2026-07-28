use super::*;

#[derive(Default)]
pub(super) struct LedgerWriter {
    bytes: Vec<u8>,
}

impl LedgerWriter {
    pub(super) fn finish(self) -> Vec<u8> {
        self.bytes
    }

    pub(super) fn bytes(&mut self, bytes: &[u8]) {
        self.bytes.extend_from_slice(bytes);
    }

    pub(super) fn u8(&mut self, value: u8) {
        self.bytes.push(value);
    }

    pub(super) fn u16(&mut self, value: u16) {
        self.bytes.extend_from_slice(&value.to_le_bytes());
    }

    pub(super) fn u32(&mut self, value: u32) {
        self.bytes.extend_from_slice(&value.to_le_bytes());
    }

    pub(super) fn u64(&mut self, value: u64) {
        self.bytes.extend_from_slice(&value.to_le_bytes());
    }

    pub(super) fn count(&mut self, value: usize) -> Result<(), CommandLedgerError> {
        self.u32(u32::try_from(value).map_err(|_| CommandLedgerError::CountOverflow)?);
        Ok(())
    }

    pub(super) fn sized_bytes(&mut self, value: &[u8]) -> Result<(), CommandLedgerError> {
        self.count(value.len())?;
        self.bytes(value);
        Ok(())
    }

    pub(super) fn text(&mut self, value: &str) -> Result<(), CommandLedgerError> {
        self.sized_bytes(value.as_bytes())
    }

    pub(super) fn principal(
        &mut self,
        principal: &IssuerPrincipal,
    ) -> Result<(), CommandLedgerError> {
        self.sized_bytes(&principal.canonical_bytes()?)
    }

    pub(super) fn option_u64(&mut self, value: Option<u64>) {
        match value {
            None => self.u8(0),
            Some(value) => {
                self.u8(1);
                self.u64(value);
            }
        }
    }

    pub(super) fn option_hash(&mut self, value: Option<ContentHash>) {
        match value {
            None => self.u8(0),
            Some(value) => {
                self.u8(1);
                self.bytes(value.as_bytes());
            }
        }
    }
}

pub(super) struct LedgerReader<'a> {
    cursor: CanonicalCursor<'a>,
    limits: CanonicalDecodeLimits,
}

impl<'a> LedgerReader<'a> {
    pub(super) fn new(bytes: &'a [u8], limits: CanonicalDecodeLimits) -> Self {
        Self {
            cursor: CanonicalCursor::new(bytes),
            limits,
        }
    }

    pub(super) fn finish(self) -> Result<(), CommandLedgerError> {
        self.cursor.finish().map_err(Into::into)
    }

    pub(super) fn u8(&mut self) -> Result<u8, CommandLedgerError> {
        self.cursor.read_u8().map_err(Into::into)
    }

    pub(super) fn u16(&mut self) -> Result<u16, CommandLedgerError> {
        self.cursor.read_u16().map_err(Into::into)
    }

    pub(super) fn u32(&mut self) -> Result<u32, CommandLedgerError> {
        self.cursor.read_u32().map_err(Into::into)
    }

    pub(super) fn u64(&mut self) -> Result<u64, CommandLedgerError> {
        self.cursor.read_u64().map_err(Into::into)
    }

    pub(super) fn array<const N: usize>(&mut self) -> Result<[u8; N], CommandLedgerError> {
        read_array(&mut self.cursor)
    }

    pub(super) fn count(&mut self) -> Result<usize, CommandLedgerError> {
        self.cursor
            .read_count(self.limits.max_sequence_items, |actual, limit| {
                CanonicalDecodeError::TooManyFields { actual, limit }
            })
            .map_err(Into::into)
    }

    pub(super) fn sized_bytes(&mut self) -> Result<&'a [u8], CommandLedgerError> {
        self.cursor
            .read_u32_length_prefixed(self.limits.max_field_payload_bytes)
            .map_err(Into::into)
    }

    pub(super) fn text(&mut self) -> Result<&'a str, CommandLedgerError> {
        std::str::from_utf8(self.sized_bytes()?)
            .map_err(|_| CommandLedgerError::Decode(CanonicalDecodeError::InvalidUtf8))
    }

    pub(super) fn principal(&mut self) -> Result<IssuerPrincipal, CommandLedgerError> {
        let limits = self.limits;
        let bytes = self.sized_bytes()?;
        IssuerPrincipal::from_canonical_bytes(bytes, limits).map_err(Into::into)
    }

    pub(super) fn option_u64(&mut self) -> Result<Option<u64>, CommandLedgerError> {
        match self.u8()? {
            0 => Ok(None),
            1 => Ok(Some(self.u64()?)),
            tag => Err(CommandLedgerError::UnknownTag(tag)),
        }
    }

    pub(super) fn option_hash(&mut self) -> Result<Option<ContentHash>, CommandLedgerError> {
        match self.u8()? {
            0 => Ok(None),
            1 => Ok(Some(ContentHash::from_bytes(self.array()?))),
            tag => Err(CommandLedgerError::UnknownTag(tag)),
        }
    }
}

pub(super) fn read_array<const N: usize>(
    cursor: &mut CanonicalCursor<'_>,
) -> Result<[u8; N], CommandLedgerError> {
    cursor
        .read_exact(N)?
        .try_into()
        .map_err(|_| CommandLedgerError::Decode(CanonicalDecodeError::UnexpectedEnd))
}
