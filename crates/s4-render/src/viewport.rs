//! Viewport management.
//!
//! 规格文档引用:
//!   /20-viewport-and-map-rendering — 视口初始化与画布几何
//!   /18-vga-mode-and-display-setup — 显示设置

/// 视口结构。
///
/// 规格: /20-viewport-and-map-rendering
/// 640×480显示表面，可配置裁剪区域。
#[derive(Debug, Clone)]
pub struct Viewport {
    /// 显示宽度（像素）。
    pub width: u16,       // 640 (0x280)
    /// 显示高度（像素）。
    pub height: u16,      // 480 (0x1E0)
    /// 逻辑瓦片宽度。
    pub tile_w: u8,       // 15 (0x0F)
    /// 逻辑瓦片高度。
    pub tile_h: u8,       // 16 (0x10)
    /// 滚动X偏移（像素）。
    pub scroll_x: i16,
    /// 滚动Y偏移（像素）。
    pub scroll_y: i16,
    /// 裁剪区域。
    pub clip: ClipRect,
    /// 内容缓冲区。
    pub content_buffer: Option<Vec<u8>>,
    /// 缓冲区偏移。
    pub buffer_offset: u32,
    /// 是否脏（需要重绘）。
    pub dirty: bool,
}

/// 裁剪矩形。
#[derive(Debug, Clone, Copy)]
pub struct ClipRect {
    pub x: i16,
    pub y: i16,
    pub w: u16,
    pub h: u16,
}

impl ClipRect {
    pub fn new(x: i16, y: i16, w: u16, h: u16) -> Self {
        Self { x, y, w, h }
    }

    /// 点是否在矩形内。
    pub fn contains(&self, px: i16, py: i16) -> bool {
        px >= self.x && px < self.x + self.w as i16 &&
        py >= self.y && py < self.y + self.h as i16
    }

    /// 与另一个矩形是否有交集。
    pub fn intersects(&self, other: &ClipRect) -> bool {
        self.x < other.x + other.w as i16 &&
        self.x + self.w as i16 > other.x &&
        self.y < other.y + other.h as i16 &&
        self.y + self.h as i16 > other.y
    }

    /// 计算交集。
    pub fn intersection(&self, other: &ClipRect) -> Option<ClipRect> {
        let x = self.x.max(other.x);
        let y = self.y.max(other.y);
        let x2 = (self.x + self.w as i16).min(other.x + other.w as i16);
        let y2 = (self.y + self.h as i16).min(other.y + other.h as i16);
        if x < x2 && y < y2 {
            Some(ClipRect::new(x, y, (x2 - x) as u16, (y2 - y) as u16))
        } else {
            None
        }
    }
}

impl Default for Viewport {
    fn default() -> Self {
        Self {
            width: 640,
            height: 480,
            tile_w: 15,
            tile_h: 16,
            scroll_x: 0,
            scroll_y: 0,
            clip: ClipRect::new(0, 0, 640, 480),
            content_buffer: None,
            buffer_offset: 0,
            dirty: true,
        }
    }
}

impl Viewport {
    /// 创建默认视口。
    pub fn new() -> Self {
        Self::default()
    }

    /// 创建带边界约束的视口。
    /// 规格: /20-viewport-and-map-rendering — init_bounded
    pub fn new_bounded(x: i16, y: i16, w: u16, h: u16) -> Self {
        Self {
            clip: ClipRect::new(x, y, w, h),
            width: w,
            height: h,
            ..Self::default()
        }
    }

    /// 设置边界。
    /// 规格: /20-viewport-and-map-rendering — set_bounds
    pub fn set_bounds(&mut self, x: i16, y: i16, w: u16, h: u16) {
        self.clip = ClipRect::new(x, y, w, h);
        self.width = w;
        self.height = h;
        self.scroll_x = 0;
        self.scroll_y = 0;
        self.dirty = true;
    }

    /// 重置滚动到原点。
    /// 规格: /21-fade-and-scroll-animation — reset_scroll
    pub fn reset_scroll(&mut self) {
        self.scroll_x = 0;
        self.scroll_y = 0;
        self.dirty = true;
    }

    /// 获取当前滚动位置。
    pub fn get_scroll(&self) -> (i16, i16) {
        (self.scroll_x, self.scroll_y)
    }

    /// 设置滚动位置。
    pub fn set_scroll(&mut self, x: i16, y: i16) {
        if self.scroll_x != x || self.scroll_y != y {
            self.scroll_x = x;
            self.scroll_y = y;
            self.dirty = true;
        }
    }

    /// 清除视口内容。
    pub fn clear(&mut self) {
        if let Some(ref mut buf) = self.content_buffer {
            buf.iter_mut().for_each(|b| *b = 0);
        }
        self.scroll_x = 0;
        self.scroll_y = 0;
        self.dirty = true;
    }

    /// 计算字节对齐的步长。
    pub fn byte_aligned_stride(&self) -> usize {
        ((self.width as usize + 7) >> 3) << 3
    }

    /// 构建内容缓冲区。
    /// 规格: /20-viewport-and-map-rendering — build_buffer
    pub fn build_buffer(&mut self) {
        let stride = self.byte_aligned_stride();
        let size = stride * self.height as usize;
        self.content_buffer = Some(vec![0u8; size]);
        self.buffer_offset = 0;
    }

    /// 获取缓冲区切片。
    pub fn buffer_mut(&mut self) -> Option<&mut [u8]> {
        self.content_buffer.as_deref_mut()
    }

    /// 视口变换：将世界坐标转换为屏幕坐标。
    pub fn transform(&self, wx: i16, wy: i16) -> (i16, i16) {
        (wx - self.scroll_x, wy - self.scroll_y)
    }

    /// 逆变换：将屏幕坐标转换为世界坐标。
    pub fn inverse_transform(&self, sx: i16, sy: i16) -> (i16, i16) {
        (sx + self.scroll_x, sy + self.scroll_y)
    }

    /// 坐标是否在视口内。
    pub fn is_visible(&self, sx: i16, sy: i16) -> bool {
        self.clip.contains(sx, sy)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_viewport_default() {
        let vp = Viewport::new();
        assert_eq!(vp.width, 640);
        assert_eq!(vp.height, 480);
    }

    #[test]
    fn test_clip_rect_contains() {
        let r = ClipRect::new(10, 20, 100, 50);
        assert!(r.contains(50, 30));
        assert!(!r.contains(5, 30));
        assert!(!r.contains(110, 30));
    }

    #[test]
    fn test_viewport_scroll() {
        let mut vp = Viewport::new();
        vp.dirty = false;  // 手动重置dirty
        vp.set_scroll(100, 200);
        assert!(vp.dirty);
        assert_eq!(vp.get_scroll(), (100, 200));
    }

    #[test]
    fn test_viewport_transform() {
        let mut vp = Viewport::new();
        vp.set_scroll(50, 30);
        let (sx, sy) = vp.transform(100, 60);
        assert_eq!((sx, sy), (50, 30));
    }
}
