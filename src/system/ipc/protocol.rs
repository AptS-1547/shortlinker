//! IPC protocol encoding and decoding
//!
//! Message format:
//! - 4 bytes: message length (big-endian u32)
//! - N bytes: JSON payload
//!
//! This module provides functions for encoding and decoding IPC messages.

use bytes::{Buf, BufMut, BytesMut};
use serde::{de::DeserializeOwned, Serialize};
use std::fmt;

/// Maximum allowed message size (64KB)
pub const MAX_MESSAGE_SIZE: usize = 64 * 1024;

/// Protocol errors
#[derive(Debug)]
pub enum ProtocolError {
    /// Message exceeds maximum allowed size
    MessageTooLarge(usize),
    /// JSON serialization/deserialization error
    JsonError(serde_json::Error),
    /// Incomplete message (need more data)
    Incomplete,
}

impl fmt::Display for ProtocolError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ProtocolError::MessageTooLarge(size) => {
                write!(
                    f,
                    "Message too large: {} bytes (max: {})",
                    size, MAX_MESSAGE_SIZE
                )
            }
            ProtocolError::JsonError(e) => write!(f, "JSON error: {}", e),
            ProtocolError::Incomplete => write!(f, "Incomplete message"),
        }
    }
}

impl std::error::Error for ProtocolError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            ProtocolError::JsonError(e) => Some(e),
            _ => None,
        }
    }
}

impl From<serde_json::Error> for ProtocolError {
    fn from(err: serde_json::Error) -> Self {
        ProtocolError::JsonError(err)
    }
}

/// Encode a message for transmission
///
/// Returns a byte vector containing:
/// - 4 bytes: message length (big-endian u32)
/// - N bytes: JSON-encoded payload
pub fn encode<T: Serialize>(msg: &T) -> Result<Vec<u8>, ProtocolError> {
    let json = serde_json::to_vec(msg)?;

    if json.len() > MAX_MESSAGE_SIZE {
        return Err(ProtocolError::MessageTooLarge(json.len()));
    }

    let mut buf = Vec::with_capacity(4 + json.len());
    buf.put_u32(json.len() as u32);
    buf.extend_from_slice(&json);
    Ok(buf)
}

/// Decode a message from a buffer
///
/// Returns:
/// - `Ok(Some(msg))` - Complete message decoded, buffer advanced
/// - `Ok(None)` - Need more data (buffer unchanged)
/// - `Err(e)` - Protocol error
///
/// The buffer is only modified when a complete message is successfully decoded.
pub fn decode<T: DeserializeOwned>(buf: &mut BytesMut) -> Result<Option<T>, ProtocolError> {
    // Need at least 4 bytes for the length header
    if buf.len() < 4 {
        return Ok(None);
    }

    // Peek at the length without consuming
    let length = (&buf[..4]).get_u32() as usize;

    // Validate message size
    if length > MAX_MESSAGE_SIZE {
        return Err(ProtocolError::MessageTooLarge(length));
    }

    // Check if we have the complete message
    if buf.len() < 4 + length {
        return Ok(None);
    }

    // Now consume the length header
    buf.advance(4);

    // Extract the JSON payload
    let json_bytes = buf.split_to(length);

    // Deserialize
    let msg = serde_json::from_slice(&json_bytes)?;
    Ok(Some(msg))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode_decode_roundtrip() {
        use crate::system::ipc::types::IpcCommand;

        let cmd = IpcCommand::Ping;
        let encoded = encode(&cmd).unwrap();

        let mut buf = BytesMut::from(&encoded[..]);
        let decoded: Option<IpcCommand> = decode(&mut buf).unwrap();

        assert!(matches!(decoded, Some(IpcCommand::Ping)));
        assert!(buf.is_empty());
    }

    #[test]
    fn test_incomplete_message() {
        let mut buf = BytesMut::from(&[0, 0, 0, 10][..]); // Length says 10 bytes, but no payload
        let result: Result<Option<String>, _> = decode(&mut buf);
        assert!(matches!(result, Ok(None)));
    }

    #[test]
    fn test_message_too_large() {
        let mut buf = BytesMut::with_capacity(8);
        buf.put_u32((MAX_MESSAGE_SIZE + 1) as u32);
        buf.put_u32(0); // Dummy data

        let result: Result<Option<String>, _> = decode(&mut buf);
        assert!(matches!(result, Err(ProtocolError::MessageTooLarge(_))));
    }
}
