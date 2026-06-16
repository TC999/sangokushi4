//! Bitstream reader for LZSS decompression.
//!
//! 规格文档引用:
//!   /26-lzss-decompression — bitstream_read

/// 伽马编码比特流读取器。
///
/// 规格: /26-lzss-decompression
/// 使用变长伽马编码：一元前缀 + 二进制后缀。
pub struct BitstreamReader {
    current_byte: u8,
    bits_remaining: i8,
}

impl BitstreamReader {
    pub fn new() -> Self {
        Self {
            current_byte: 0,
            bits_remaining: 0,
        }
    }

    /// 读取一个比特。
    pub fn read_bit<F>(&mut self, mut byte_source: F) -> Option<bool>
    where
        F: FnMut() -> Option<u8>,
    {
        if self.bits_remaining <= 0 {
            self.current_byte = byte_source()?;
            self.bits_remaining = 7;
        }
        let bit = (self.current_byte & 0x80) != 0;
        self.current_byte <<= 1;
        self.bits_remaining -= 1;
        Some(bit)
    }

    /// 读取指定数量的比特。
    pub fn read_bits<F>(&mut self, count: u8, byte_source: &mut F) -> Option<u8>
    where
        F: FnMut() -> Option<u8>,
    {
        let mut value: u8 = 0;
        for _ in 0..count {
            let bit = self.read_bit(&mut *byte_source)?;
            value = (value << 1) | if bit { 1 } else { 0 };
        }
        Some(value)
    }
}
