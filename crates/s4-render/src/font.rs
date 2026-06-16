//! Font rendering.
//!
//! 规格文档引用:
//!   /18-vga-mode-and-display-setup — VGA文本模式与光标渲染
//!   /19-tile-and-sprite-blitting-engine — 字体位图扩展

/// 字体渲染器。
pub struct FontRenderer {
    /// 字体宽度（像素）。
    pub char_width: u8,
    /// 字体高度（像素）。
    pub char_height: u8,
    /// 字体位图数据。
    pub bitmap_data: Vec<u8>,
    /// 当前光标位置。
    pub cursor_x: u16,
    pub cursor_y: u16,
    /// 光标可见标志。
    pub cursor_visible: bool,
}

impl Default for FontRenderer {
    fn default() -> Self {
        Self {
            char_width: 8,
            char_height: 16,
            bitmap_data: Vec::new(),
            cursor_x: 0,
            cursor_y: 0,
            cursor_visible: false,
        }
    }
}

impl FontRenderer {
    pub fn new() -> Self {
        Self::default()
    }

    /// 位图扩展（2倍水平/垂直）。
    /// 规格: /19-tile-and-sprite-blitting-engine — expand_bitmap
    /// 8像素模式：每个字节通过右移-OR操作加倍水平分辨率。
    pub fn expand_bitmap_8px(src: &[u8], dst: &mut [u8]) {
        for (s, d) in src.iter().zip(dst.chunks_exact_mut(2)) {
            // 右移1位，保留LSB
            d[0] = *s >> 1 | (*s & 1);
            // 左移1位
            d[1] = *s << 1;
        }
    }

    /// 16像素模式位图扩展。
    pub fn expand_bitmap_16px(src: &[u8], dst: &mut [u8]) {
        // 每16字节处理为32字节
        for i in 0..src.len().min(16) {
            let byte = src[i];
            dst[i * 2] = byte;
            // 传播溢出位
            if i > 0 {
                dst[i * 2] |= if src[i - 1] & 1 != 0 { 0x80 } else { 0 };
            }
            dst[i * 2 + 1] = byte << 1;
            if i + 1 < src.len() {
                dst[i * 2 + 1] |= if src[i + 1] & 0x80 != 0 { 1 } else { 0 };
            }
        }
    }

    /// Shift-JIS字符范围检测。
    /// 规格: /9-main-game-loop — char_is_sjis_range
    pub fn is_sjis_range(ch: u8) -> bool {
        (0x81..=0x9F).contains(&ch) || (0xE0..=0xFC).contains(&ch)
    }

    /// 切换光标闪烁。
    /// 规格: /18-vga-mode-and-display-setup — toggle_cursor_blink
    pub fn toggle_cursor_blink(&mut self) {
        self.cursor_visible = !self.cursor_visible;
    }

    /// 设置光标位置。
    pub fn set_cursor_pos(&mut self, x: u16, y: u16) {
        self.cursor_x = x;
        self.cursor_y = y;
    }
}

/// 字体宽度查找表（256项）。
pub struct FontWidthTable {
    widths: [u8; 256],
}

impl Default for FontWidthTable {
    fn default() -> Self {
        Self { widths: [8; 256] }
    }
}

impl FontWidthTable {
    pub fn new() -> Self { Self::default() }

    /// 获取字符宽度。
    pub fn get_width(&self, ch: u8) -> u8 {
        self.widths[ch as usize]
    }

    /// 设置字符宽度。
    pub fn set_width(&mut self, ch: u8, w: u8) {
        self.widths[ch as usize] = w;
    }

    /// 计算字符串总宽度。
    pub fn string_width(&self, s: &[u8]) -> u16 {
        s.iter().map(|&ch| self.get_width(ch) as u16).sum()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_expand_bitmap_8px() {
        let src = [0b10101010];
        let mut dst = [0u8; 2];
        FontRenderer::expand_bitmap_8px(&src, &mut dst);
        assert_eq!(dst[0], 0b01010101);
        assert_eq!(dst[1], 0b01010100);
    }

    #[test]
    fn test_sjis_range() {
        assert!(FontRenderer::is_sjis_range(0x81));
        assert!(FontRenderer::is_sjis_range(0x9F));
        assert!(FontRenderer::is_sjis_range(0xE0));
        assert!(!FontRenderer::is_sjis_range(0x7F));
    }

    #[test]
    fn test_font_width_table() {
        let mut table = FontWidthTable::new();
        table.set_width(b'A', 8);
        table.set_width(b'i', 4);
        assert_eq!(table.get_width(b'A'), 8);
        assert_eq!(table.get_width(b'i'), 4);
    }
}
