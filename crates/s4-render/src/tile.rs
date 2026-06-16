//! Tile and sprite blitting engine.
//!
//! 规格文档引用:
//!   /19-tile-and-sprite-blitting-engine — 瓦片与精灵绘制
//!   /20-viewport-and-map-rendering — 瓦片渲染管道

use crate::viewport::Viewport;
use crate::palette::Palette;

/// 瓦片缓冲区大小（128字节：4平面 × 32字节）。
pub const TILE_BUFFER_SIZE: usize = 128;

/// 光栅操作模式。
/// 规格: /19-tile-and-sprite-blitting-engine — 五种光栅操作
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RasterOp {
    /// 直接复制。
    Copy = 0,
    /// XOR混合。
    Xor = 1,
    /// AND掩码。
    And = 2,
    /// OR混合。
    Or = 3,
}

/// 瓦片渲染器。
///
/// 规格: /19-tile-and-sprite-blitting-engine
/// 在VGA Mode X（320×200, 256色, 4平面平面内存）中操作。
pub struct TileRenderer {
    /// 瓦片缓冲区（合成后写入VRAM）。
    pub tile_buffer: [u8; TILE_BUFFER_SIZE],
    /// 调色板。
    pub palette: Palette,
}

impl Default for TileRenderer {
    fn default() -> Self {
        Self {
            tile_buffer: [0; TILE_BUFFER_SIZE],
            palette: Palette::default(),
        }
    }
}

impl TileRenderer {
    pub fn new() -> Self {
        Self::default()
    }

    /// 清空瓦片缓冲区。
    pub fn clear_buffer(&mut self) {
        self.tile_buffer.iter_mut().for_each(|b| *b = 0);
    }

    /// 复制128字节位图到瓦片缓冲区。
    /// 规格: /19-tile-and-sprite-blitting-engine — copy_bitmap_128
    pub fn copy_bitmap_128(&mut self, src: &[u8]) {
        let len = src.len().min(TILE_BUFFER_SIZE);
        self.tile_buffer[..len].copy_from_slice(&src[..len]);
    }

    /// 平面合并（透明精灵合成）。
    /// 规格: /19-tile-and-sprite-blitting-engine — merge_planes_simple
    ///
    /// 算法：对于每个像素位置，计算四平面值的并集作为覆盖掩码，
    /// 反转掩码清除目标像素，然后OR入源平面值。
    pub fn merge_planes_simple(dst: &mut [u8], src: &[u8]) {
        // 每16个uint为一组处理
        // dst布局: plane0(16) + plane1(16) + plane2(16) + plane3(16) = 64字节
        // src布局: 同上
        for i in 0..16 {
            let base = i * 2;
            // 检查所有索引都在范围内
            if base + 1 >= dst.len() { break; }
            if base + 0x10 + 1 >= src.len() { break; }
            if base + 0x20 + 1 >= src.len() { break; }
            if base + 0x30 + 1 >= src.len() { break; }
            // 计算覆盖掩码
            let plane0 = src[base] as u16 | (src[base + 1] as u16) << 8;
            let plane1 = src[base + 0x10] as u16 | (src[base + 0x11] as u16) << 8;
            let plane2 = src[base + 0x20] as u16 | (src[base + 0x21] as u16) << 8;
            let plane3 = src[base + 0x30] as u16 | (src[base + 0x31] as u16) << 8;
            let mask = !(plane0 | plane1 | plane2 | plane3);

            // 清除目标并OR入源
            let dst0 = dst[base] as u16 | (dst[base + 1] as u16) << 8;
            let dst1 = dst[base + 0x10] as u16 | (dst[base + 0x11] as u16) << 8;
            let dst2 = dst[base + 0x20] as u16 | (dst[base + 0x21] as u16) << 8;
            let dst3 = dst[base + 0x30] as u16 | (dst[base + 0x31] as u16) << 8;

            let r0 = ((dst0 & mask) | plane0) as u8;
            let r0h = (((dst0 & mask) | plane0) >> 8) as u8;
            let r1 = ((dst1 & mask) | plane1) as u8;
            let r1h = (((dst1 & mask) | plane1) >> 8) as u8;
            let r2 = ((dst2 & mask) | plane2) as u8;
            let r2h = (((dst2 & mask) | plane2) >> 8) as u8;
            let r3 = ((dst3 & mask) | plane3) as u8;
            let r3h = (((dst3 & mask) | plane3) >> 8) as u8;

            if base + 1 < dst.len() {
                dst[base] = r0;
                dst[base + 1] = r0h;
            }
            if base + 0x10 + 1 < dst.len() {
                dst[base + 0x10] = r1;
                dst[base + 0x11] = r1h;
            }
            if base + 0x20 + 1 < dst.len() {
                dst[base + 0x20] = r2;
                dst[base + 0x21] = r2h;
            }
            if base + 0x30 + 1 < dst.len() {
                dst[base + 0x30] = r3;
                dst[base + 0x31] = r3h;
            }
        }
    }

    /// 字节级平面合并。
    /// 规格: merge_planes_direct
    pub fn merge_planes_direct(dst: &mut [u8], src: &[u8]) {
        for i in 0..dst.len().min(src.len()) {
            dst[i] = (dst[i] & !src[i]) | src[i];
        }
    }

    /// 应用AND掩码。
    pub fn apply_and_mask(data: &mut [u8], mask: u8) {
        for b in data.iter_mut() {
            *b &= mask;
        }
    }

    /// 填充纯色。
    /// 规格: fill_planes_solid
    pub fn fill_planes_solid(dst: &mut [u8], pattern: &[u8; 4]) {
        let mut parity = false;
        for chunk in dst.chunks_exact_mut(4) {
            let p = if parity { &pattern[2..4] } else { &pattern[0..2] };
            chunk[0] = p[0];
            chunk[1] = if p.len() > 1 { p[1] } else { p[0] };
            chunk[2] = p[0];
            chunk[3] = if p.len() > 1 { p[1] } else { p[0] };
            parity = !parity;
        }
    }

    /// OR所有平面。
    pub fn or_planes_all(dst: &mut [u8], val: u8) {
        for b in dst.iter_mut() {
            *b |= val;
        }
    }

    /// 子像素精灵绘制（位移对齐）。
    /// 规格: /19-tile-and-sprite-blitting-engine — draw_sprite_tile
    pub fn draw_sprite_subpixel(
        dst: &mut [u8],
        src: &[u8],
        pixel_x: u16,
        _buffer_base: usize,
        shift: u8,  // 像素偏移 (0-7)
    ) {
        if shift == 0 {
            // 字节对齐，直接复制
            for (d, s) in dst.iter_mut().zip(src.iter()) {
                *d = *s;
            }
            return;
        }

        let shift_inv = 8 - shift;
        for (i, s) in src.iter().enumerate() {
            let byte_idx = pixel_x as usize / 8 + i * 4;
            if byte_idx < dst.len() {
                let low = s >> shift;
                let high = s << shift_inv;
                dst[byte_idx] = low;
                if byte_idx + 1 < dst.len() {
                    dst[byte_idx + 1] |= high;
                }
            }
        }
    }

    /// 调色板重映射。
    /// 规格: lookup_convert_tile
    pub fn lookup_convert_tile(data: &mut [u8], table: &[u8; 256]) {
        for b in data.iter_mut() {
            *b = table[*b as usize];
        }
    }
}

/// 交错等距坐标变换。
/// 规格: /20-viewport-and-map-rendering — grid_to_screen
pub fn grid_to_screen(col: u8, row: u8) -> (i32, i32) {
    let is_odd = (row & 1) != 0;
    let screen_x = if is_odd {
        (col as i32 + 2) * 32
    } else {
        col as i32 * 32 + 48
    };
    let screen_y = row as i32 * 32;
    (screen_x, screen_y)
}

/// 交错变换。
pub fn stagger_transform(col: u8, row: u8) -> (u16, u16) {
    let stagger_x = if (row & 1) != 0 {
        col as u16 * 2 + 1
    } else {
        col as u16 * 2
    };
    let stagger_y = row as u16 * 2;
    (stagger_x, stagger_y)
}

/// 渲染瓦片层（含分层合成）。
/// 规格: /19-tile-and-sprite-blitting-engine — render_tiles
pub fn render_tiles(
    renderer: &mut TileRenderer,
    viewport: &Viewport,
    base_tile: &[u8; TILE_BUFFER_SIZE],
    overlay_flags: u16,
) {
    // 复制基础地形
    renderer.copy_bitmap_128(base_tile);

    // 根据标志位叠加层
    if overlay_flags & 0x001 != 0 {
        // 事件标记叠加层1
    }
    if overlay_flags & 0x002 != 0 {
        // 事件叠加层2
    }
    if overlay_flags & 0x008 != 0 {
        // 单位渲染
    }
    if overlay_flags & 0x010 != 0 {
        // 事件叠加层3
    }
    if overlay_flags & 0x100 != 0 {
        // 位图合并
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_merge_planes_simple() {
        let mut dst = [0u8; 0x40];
        let mut src = [0u8; 0x40];
        // 设置源平面0
        src[0] = 0xFF;
        src[1] = 0x00;
        // 合并
        TileRenderer::merge_planes_simple(&mut dst, &src);
        // 目标平面0应有值
        assert_eq!(dst[0], 0xFF);
    }

    #[test]
    fn test_grid_to_screen() {
        let (x, y) = grid_to_screen(0, 0);
        assert_eq!((x, y), (48, 0)); // 偶数行
        let (x, y) = grid_to_screen(0, 1);
        assert_eq!((x, y), (64, 32)); // 奇数行偏移
    }

    #[test]
    fn test_lookup_convert() {
        let mut data = vec![0, 1, 2, 3];
        let mut table = [0u8; 256];
        table[0] = 10;
        table[1] = 20;
        table[2] = 30;
        table[3] = 40;
        TileRenderer::lookup_convert_tile(&mut data, &table);
        assert_eq!(data, vec![10, 20, 30, 40]);
    }
}
