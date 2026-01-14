//! Core observability types.

use serde::{Deserialize, Serialize};
use std::fmt;

/// A 128-bit trace identifier.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TraceId(pub [u8; 16]);

/// A 64-bit span identifier.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SpanId(pub [u8; 8]);

impl TraceId {
    /// Generate a new random trace ID.
    #[must_use]
    pub fn new() -> Self {
        let mut bytes = [0u8; 16];
        getrandom::fill(&mut bytes).expect("failed to generate random bytes");
        Self(bytes)
    }

    /// Convert to a 32-character lowercase hex string.
    #[must_use]
    pub fn to_hex(&self) -> String {
        self.0.iter().fold(String::with_capacity(32), |mut acc, b| {
            use fmt::Write;
            let _ = write!(acc, "{b:02x}");
            acc
        })
    }

    /// Parse from a 32-character hex string.
    pub fn from_hex(s: &str) -> Result<Self, ParseIdError> {
        if s.len() != 32 {
            return Err(ParseIdError::InvalidLength {
                expected: 32,
                got: s.len(),
            });
        }
        let mut bytes = [0u8; 16];
        for (i, chunk) in s.as_bytes().chunks(2).enumerate() {
            let high = hex_digit(chunk[0])?;
            let low = hex_digit(chunk[1])?;
            bytes[i] = (high << 4) | low;
        }
        Ok(Self(bytes))
    }
}

impl Default for TraceId {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for TraceId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_hex())
    }
}

impl SpanId {
    /// Generate a new random span ID.
    #[must_use]
    pub fn new() -> Self {
        let mut bytes = [0u8; 8];
        getrandom::fill(&mut bytes).expect("failed to generate random bytes");
        Self(bytes)
    }

    /// Convert to a 16-character lowercase hex string.
    #[must_use]
    pub fn to_hex(&self) -> String {
        self.0.iter().fold(String::with_capacity(16), |mut acc, b| {
            use fmt::Write;
            let _ = write!(acc, "{b:02x}");
            acc
        })
    }

    /// Parse from a 16-character hex string.
    pub fn from_hex(s: &str) -> Result<Self, ParseIdError> {
        if s.len() != 16 {
            return Err(ParseIdError::InvalidLength {
                expected: 16,
                got: s.len(),
            });
        }
        let mut bytes = [0u8; 8];
        for (i, chunk) in s.as_bytes().chunks(2).enumerate() {
            let high = hex_digit(chunk[0])?;
            let low = hex_digit(chunk[1])?;
            bytes[i] = (high << 4) | low;
        }
        Ok(Self(bytes))
    }
}

impl Default for SpanId {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for SpanId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_hex())
    }
}

/// Error parsing a hex ID string.
#[derive(Debug, Clone, thiserror::Error)]
pub enum ParseIdError {
    #[error("invalid hex digit: {0}")]
    InvalidHexDigit(char),
    #[error("invalid length: expected {expected}, got {got}")]
    InvalidLength { expected: usize, got: usize },
}

fn hex_digit(c: u8) -> Result<u8, ParseIdError> {
    match c {
        b'0'..=b'9' => Ok(c - b'0'),
        b'a'..=b'f' => Ok(c - b'a' + 10),
        b'A'..=b'F' => Ok(c - b'A' + 10),
        _ => Err(ParseIdError::InvalidHexDigit(c as char)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn trace_id_roundtrip() {
        let id = TraceId::new();
        let hex = id.to_hex();
        assert_eq!(hex.len(), 32);
        let parsed = TraceId::from_hex(&hex).unwrap();
        assert_eq!(id, parsed);
    }

    #[test]
    fn span_id_roundtrip() {
        let id = SpanId::new();
        let hex = id.to_hex();
        assert_eq!(hex.len(), 16);
        let parsed = SpanId::from_hex(&hex).unwrap();
        assert_eq!(id, parsed);
    }

    #[test]
    fn trace_id_display() {
        let id = TraceId([0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15]);
        assert_eq!(id.to_string(), "000102030405060708090a0b0c0d0e0f");
    }

    #[test]
    fn span_id_display() {
        let id = SpanId([0, 1, 2, 3, 4, 5, 6, 7]);
        assert_eq!(id.to_string(), "0001020304050607");
    }
}
