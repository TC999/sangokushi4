//! LZSS decompression engine.
//!
//! 规格文档引用:
//!   /26-lzss-decompression — LZSS解压缩
//!   /25-resource-loading-and-decompression — 资源管道

/// LZSS解压缩器。
///
/// 规格: /26-lzss-decompression
/// 使用伽马编码比特流的变种LZSS实现。
pub struct LzssDecompressor {
    current_byte: u8,
    bit_counter: i8,
    buffer: Vec<u8>,
    buffer_pos: usize,
    eof: bool,
}

impl LzssDecompressor {
    /// 创建新的解压缩器。
    pub fn new(data: Vec<u8>) -> Self {
        Self {
            current_byte: 0,
            bit_counter: 0,
            buffer: data,
            buffer_pos: 0,
            eof: false,
        }
    }

    /// 读取一个比特（MSB first）。
    fn read_bit(&mut self) -> Option<bool> {
        if self.bit_counter <= 0 {
            self.current_byte = self.read_byte()?;
            self.bit_counter = 7;
        }
        let bit = (self.current_byte & 0x80) != 0;
        self.current_byte <<= 1;
        self.bit_counter -= 1;
        Some(bit)
    }

    /// 从缓冲区读取一个字节。
    fn read_byte(&mut self) -> Option<u8> {
        if self.buffer_pos >= self.buffer.len() {
            self.eof = true;
            return None;
        }
        let byte = self.buffer[self.buffer_pos];
        self.buffer_pos += 1;
        Some(byte)
    }

    /// 伽马编码比特流读取。
    /// 规格: /26-lzss-decompression — bitstream_read
    /// 阶段1: 一元前缀（连续1-bit计数）
    /// 阶段2: 二进制后缀
    pub fn gamma_read(&mut self) -> Option<i32> {
        let mut prefix_value: i32 = 0;
        let mut prefix_bits: i32 = 0;

        // 阶段1: 读取一元前缀
        loop {
            let bit = self.read_bit()?;
            if bit {
                prefix_value = prefix_value * 2 + 1;
                prefix_bits += 1;
            } else {
                prefix_bits += 1;
                break;
            }
        }

        // 阶段2: 读取二进制后缀
        let mut suffix_value: i32 = 0;
        for _ in 0..prefix_bits {
            let bit = self.read_bit()?;
            suffix_value = suffix_value * 2 + if bit { 1 } else { 0 };
        }

        Some(prefix_value * 2 + suffix_value)
    }

    /// LZSS解压缩循环。
    /// 规格: /26-lzss-decompression — lzss_decompress
    /// 文字标记(< 0x100): 查表写入
    /// 后向引用(>= 0x100): 从历史位置复制
    pub fn decompress(&mut self, output: &mut Vec<u8>, size: usize) -> bool {
        let mut remaining = size;

        // 256字节文字查找表（初始化为恒等映射）
        let mut literal_table = [0u8; 256];
        for i in 0..256 {
            literal_table[i] = i as u8;
        }

        while remaining > 0 {
            let token = match self.gamma_read() {
                Some(v) => v,
                None => return false,
            };

            if token < 0x100 {
                // 文字标记
                let byte = literal_table[token as usize];
                output.push(byte);
                remaining -= 1;
            } else {
                // 后向引用
                let distance = (token - 0x100) as usize;
                let length_code = self.gamma_read().unwrap_or(0);
                let length = (length_code as usize) + 3; // 最小匹配长度3

                if distance == 0 || output.len() < distance {
                    return false;
                }

                let src_pos = output.len() - distance;
                for i in 0..length {
                    let byte = output[src_pos + (i % distance)];
                    output.push(byte);
                }
                remaining -= length;
            }
        }

        true
    }
}

/// RLE图像解码器。
/// 规格: /25-resource-loading-and-decompression — rle_image
pub struct RleDecoder<'a> {
    data: &'a [u8],
    pos: usize,
}

impl<'a> RleDecoder<'a> {
    pub fn new(data: &'a [u8]) -> Self {
        Self { data, pos: 0 }
    }

    fn read_byte(&mut self) -> Option<u8> {
        if self.pos >= self.data.len() {
            None
        } else {
            let b = self.data[self.pos];
            self.pos += 1;
            Some(b)
        }
    }

    /// 解码RLE图像数据（4-bpp平面格式）。
    pub fn decode(&mut self, output: &mut Vec<u8>) -> bool {
        // 读取头部
        let _width = match self.read_byte() {
            Some(b) => b as u16,
            None => return false,
        };
        let _height = match self.read_byte() {
            Some(b) => b as u16,
            None => return false,
        };

        // 解码像素数据
        while let Some(control) = self.read_byte() {
            if control & 0x80 == 0 {
                // 文字填充
                let value = control & 0x0F;
                let count = ((control >> 4) & 0x07) + 1;
                for _ in 0..count {
                    output.push(value);
                }
            } else if control & 0x40 == 0 {
                // 近距离后向引用
                let distance = ((control >> 2) & 0x03) as usize + 1;
                let length = (control & 0x03) as usize + 1;
                let src = output.len().saturating_sub(distance);
                for i in 0..length {
                    let byte = output[src + (i % distance)];
                    output.push(byte);
                }
            } else {
                // 远距离后向引用
                let distance = ((control & 0x3F) as usize + 1) * 3;
                let length = 4;
                let src = output.len().saturating_sub(distance);
                for i in 0..length {
                    if src + (i % distance) < output.len() {
                        let byte = output[src + (i % distance)];
                        output.push(byte);
                    }
                }
            }
        }

        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gamma_read() {
        // 编码 "0" = 前缀0(1位) + 后缀0(1位) = 00
        let data = vec![0b00000000];
        let mut decoder = LzssDecompressor::new(data);
        let val = decoder.gamma_read();
        assert_eq!(val, Some(0));
    }

    #[test]
    fn test_gamma_read_1() {
        // 编码 "1" = 前缀0(1位) + 后缀1(1位) = 01
        // 0 = prefix terminator bit (1 bit)
        // 1 = suffix value (1 bit)
        // MSB first: 0 1 ... = 0b01000000
        let data = vec![0b01000000];
        let mut decoder = LzssDecompressor::new(data);
        let val = decoder.gamma_read();
        assert_eq!(val, Some(1));
    }
}
