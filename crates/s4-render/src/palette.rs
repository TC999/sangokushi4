//! Palette management.
//!
//! 规格文档引用:
//!   /18-vga-mode-and-display-setup — VGA调色板设置

/// RGB颜色。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Color {
    pub const fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }

    /// 从16位颜色值解析。
    pub fn from_u16(val: u16) -> Self {
        Self {
            r: ((val >> 8) & 0x0F) as u8 * 17,
            g: ((val >> 4) & 0x0F) as u8 * 17,
            b: (val & 0x0F) as u8 * 17,
        }
    }

    /// 转为16位颜色值。
    pub fn to_u16(&self) -> u16 {
        ((self.r as u16 / 17) << 8) |
        ((self.g as u16 / 17) << 4) |
        (self.b as u16 / 17)
    }

    /// 线性插值。
    pub fn lerp(&self, other: &Color, t: f32) -> Color {
        let inv_t = 1.0 - t;
        Color::new(
            (self.r as f32 * inv_t + other.r as f32 * t) as u8,
            (self.g as f32 * inv_t + other.g as f32 * t) as u8,
            (self.b as f32 * inv_t + other.b as f32 * t) as u8,
        )
    }
}

/// 调色板。
#[derive(Debug, Clone)]
pub struct Palette {
    pub colors: [Color; 256],
}

impl Default for Palette {
    fn default() -> Self {
        Self {
            colors: [Color::new(0, 0, 0); 256],
        }
    }
}

impl Palette {
    pub fn new() -> Self {
        Self::default()
    }

    /// 设置颜色。
    pub fn set_color(&mut self, index: u8, color: Color) {
        self.colors[index as usize] = color;
    }

    /// 获取颜色。
    pub fn get_color(&self, index: u8) -> Color {
        self.colors[index as usize]
    }

    /// 从16位值批量加载。
    pub fn load_from_u16(&mut self, start: u8, values: &[u16]) {
        for (i, &val) in values.iter().enumerate() {
            let idx = start as usize + i;
            if idx < 256 {
                self.colors[idx] = Color::from_u16(val);
            }
        }
    }

    /// 淡入淡出插值。
    pub fn fade_interpolate(&self, target: &Palette, t: f32) -> Palette {
        let mut result = Palette::new();
        for i in 0..256 {
            result.colors[i] = self.colors[i].lerp(&target.colors[i], t);
        }
        result
    }

    /// VGA标准16色调色板。
    pub fn vga16_default() -> Self {
        let mut p = Palette::new();
        let colors = [
            Color::new(0, 0, 0),       // 0: 黑
            Color::new(0, 0, 170),     // 1: 蓝
            Color::new(0, 170, 0),     // 2: 绿
            Color::new(0, 170, 170),   // 3: 青
            Color::new(170, 0, 0),     // 4: 红
            Color::new(170, 0, 170),   // 5: 品红
            Color::new(170, 85, 0),    // 6: 棕
            Color::new(170, 170, 170), // 7: 亮灰
            Color::new(85, 85, 85),    // 8: 暗灰
            Color::new(85, 85, 255),   // 9: 亮蓝
            Color::new(85, 255, 85),   // 10: 亮绿
            Color::new(85, 255, 255),  // 11: 亮青
            Color::new(255, 85, 85),   // 12: 亮红
            Color::new(255, 85, 255),  // 13: 亮品红
            Color::new(255, 255, 85),  // 14: 黄
            Color::new(255, 255, 255), // 15: 白
        ];
        for (i, c) in colors.iter().enumerate() {
            p.colors[i] = *c;
        }
        p
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_color_lerp() {
        let black = Color::new(0, 0, 0);
        let white = Color::new(255, 255, 255);
        let mid = black.lerp(&white, 0.5);
        assert_eq!(mid.r, 127);
    }

    #[test]
    fn test_palette_fade() {
        let p1 = Palette::vga16_default();
        let p2 = Palette::new(); // 全黑
        let faded = p1.fade_interpolate(&p2, 0.5);
        assert_eq!(faded.colors[15].r, 127);
    }
}
