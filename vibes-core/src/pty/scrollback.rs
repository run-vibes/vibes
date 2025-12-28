//! Scrollback buffer for PTY output

use std::collections::VecDeque;

/// Default buffer capacity: 1MB
pub const DEFAULT_CAPACITY: usize = 1_048_576;

/// Ring buffer for PTY output with fixed byte capacity.
pub struct ScrollbackBuffer {
    buffer: VecDeque<u8>,
    capacity: usize,
}

impl ScrollbackBuffer {
    /// Create buffer with specified capacity
    pub fn new(capacity: usize) -> Self {
        Self {
            buffer: VecDeque::with_capacity(capacity),
            capacity,
        }
    }

    /// Append data, dropping oldest bytes if over capacity
    pub fn append(&mut self, data: &[u8]) {
        for &byte in data {
            if self.buffer.len() >= self.capacity {
                self.buffer.pop_front();
            }
            self.buffer.push_back(byte);
        }
    }

    /// Get all buffered data for replay
    pub fn get_all(&self) -> Vec<u8> {
        self.buffer.iter().copied().collect()
    }

    /// Current buffer size in bytes
    pub fn len(&self) -> usize {
        self.buffer.len()
    }

    /// Check if buffer is empty
    pub fn is_empty(&self) -> bool {
        self.buffer.is_empty()
    }
}

impl Default for ScrollbackBuffer {
    fn default() -> Self {
        Self::new(DEFAULT_CAPACITY)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_creates_empty_buffer() {
        let buf = ScrollbackBuffer::new(1024);
        assert!(buf.is_empty());
        assert_eq!(buf.len(), 0);
    }

    #[test]
    fn append_stores_data() {
        let mut buf = ScrollbackBuffer::new(1024);
        buf.append(b"hello");
        assert_eq!(buf.len(), 5);
        assert_eq!(buf.get_all(), b"hello");
    }

    #[test]
    fn append_multiple_times() {
        let mut buf = ScrollbackBuffer::new(1024);
        buf.append(b"hello ");
        buf.append(b"world");
        assert_eq!(buf.get_all(), b"hello world");
    }

    #[test]
    fn overflow_drops_oldest_bytes() {
        let mut buf = ScrollbackBuffer::new(10);
        buf.append(b"hello"); // 5 bytes
        buf.append(b"world!"); // 6 bytes, total 11, drops 1
        assert_eq!(buf.len(), 10);
        assert_eq!(buf.get_all(), b"elloworld!");
    }

    #[test]
    fn large_append_keeps_only_capacity() {
        let mut buf = ScrollbackBuffer::new(5);
        buf.append(b"hello world"); // 11 bytes
        assert_eq!(buf.len(), 5);
        assert_eq!(buf.get_all(), b"world");
    }

    #[test]
    fn default_uses_1mb_capacity() {
        let buf = ScrollbackBuffer::default();
        assert_eq!(buf.capacity, DEFAULT_CAPACITY);
    }
}
