//! Fade and scroll animation.
//!
//! 规格文档引用:
//!   /21-fade-and-scroll-animation — 淡入淡出与滚动动画
//!   /18-vga-mode-and-display-setup — 显示设置

/// 淡入淡出动画器。
///
/// 规格: /21-fade-and-scroll-animation
/// 使用32位定点算术驱动视口滚动和调色板渐变。
pub struct FadeAnimator {
    /// 32位定点累加器。
    accumulator: u32,
    /// 速度向量（低16位=速度, 高16位=分数部分）。
    velocity: u32,
    /// 常数增量（每帧添加到累加器）。
    constant: u32,
}

impl Default for FadeAnimator {
    fn default() -> Self {
        Self {
            accumulator: 0,
            velocity: 0x0003_43FD,  // 对应原版 3:0x43FD
            constant: 0x0026_9EC3,  // 对应原版 0x269EC3
        }
    }
}

impl FadeAnimator {
    pub fn new() -> Self {
        Self::default()
    }

    /// 每帧更新（定点积分）。
    /// 规格: /21-fade-and-scroll-animation — fade_in
    /// 返回15位滚动偏移量。
    pub fn tick(&mut self) -> u16 {
        // acc += velocity
        let product = Self::mul32(self.accumulator, self.velocity);
        self.accumulator = product.wrapping_add(self.constant);
        // 返回高16位的低15位
        ((self.accumulator >> 16) & 0x7FFF) as u16
    }

    /// 设置速度。
    pub fn set_velocity(&mut self, vel: u32) {
        self.velocity = vel;
    }

    /// 重置累加器。
    pub fn reset(&mut self) {
        self.accumulator = 0;
    }

    /// 32位乘法（对应原版 main_graphics_scroll）。
    /// 规格: /21-fade-and-scroll-animation — 32-bit multiplication
    fn mul32(a: u32, b: u32) -> u32 {
        a.wrapping_mul(b)
    }
}

/// 线性插值（lerp）。
/// 规格: /21-fade-and-scroll-animation — interp_fade
/// 公式: start + t * (end - start) / 100
/// 其中 t = (value * 100) / 0xB0
pub fn lerp_fade(value: u16, start: i32, end: i32) -> i32 {
    let t = (value as i32 * 100) / 0xB0;
    start + t * (end - start) / 100
}

/// 淡化百分比计算。
/// 规格: /21-fade-and-scroll-animation — calc_fade
pub fn calc_fade_percentage(current: i32, target: i32, boundary: i32) -> u16 {
    if current == target {
        0
    } else if boundary == target {
        100
    } else {
        let raw = ((current as i64 - target as i64).abs() * 100 /
                  (boundary as i64 - target as i64).abs()) as u16;
        raw.min(100)
    }
}

/// 淡化缩放计算。
/// 规格: fade_scaled — 用于资源计算的缩放因子
pub fn fade_scaled(value: u16) -> u16 {
    (value as u32 * 100 / 0xB0) as u16
}

/// 循环查找表。
/// 规格: /21-fade-and-scroll-animation — circular_lookup
pub fn circular_lookup(index: u8, table: &[i16]) -> i16 {
    let len = table.len();
    if len == 0 { return 0; }
    let idx = ((index as usize + 18) % (len * 2)) % len;
    table[idx]
}

/// 滚动淡化序列（4步复合动画）。
/// 规格: /21-fade-and-scroll-animation — scroll_fade_sequence
pub fn scroll_fade_sequence(
    params: &[i32; 4],
    table: &[i16],
) -> ([i32; 4], [i32; 4]) {
    let mut primary = [0i32; 4];
    let mut secondary = [0i32; 4];

    for i in 0..4 {
        // 阶段A: 正向
        let lookup_val = circular_lookup(i as u8, table) as i32;
        let product = FadeAnimator::mul32(lookup_val as u32, params[0] as u32);
        primary[i] = (product as i32 / 10000) + params[2] as i32;

        // 阶段B: 反向
        let stagger_idx = (i * 18) % (table.len() * 2);
        let table_val = table[stagger_idx % table.len()] as i32;
        let product2 = FadeAnimator::mul32(table_val as u32, params[1] as u32);
        secondary[i] = (product2 as i32 / 10000) + params[3] as i32;
    }

    (primary, secondary)
}

/// 帧计时器。
pub struct FrameTimer {
    last_tick: u64,
    frame_ms: u64,
}

impl FrameTimer {
    /// 创建帧计时器（指定每帧毫秒数）。
    pub fn new(fps: u32) -> Self {
        Self {
            last_tick: 0,
            frame_ms: 1000 / fps as u64,
        }
    }

    /// 检查是否到达下一帧。
    pub fn is_ready(&self, current_tick: u64) -> bool {
        current_tick >= self.last_tick + self.frame_ms
    }

    /// 标记帧已处理。
    pub fn mark(&mut self, current_tick: u64) {
        self.last_tick = current_tick;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fade_animator_tick() {
        let mut anim = FadeAnimator::new();
        let val1 = anim.tick();
        let val2 = anim.tick();
        assert_ne!(val1, val2);
    }

    #[test]
    fn test_lerp_fade() {
        // lerp_fade(value, start, end) = start + (value*100/0xB0) * (end-start) / 100
        // lerp_fade(50, 0, 100) = 0 + (5000/176) * 100 / 100 = 28
        let result = lerp_fade(50, 0, 100);
        assert_eq!(result, 28);
    }

    #[test]
    fn test_calc_fade_percentage() {
        assert_eq!(calc_fade_percentage(50, 50, 100), 0);
        assert_eq!(calc_fade_percentage(100, 50, 100), 100);
    }

    #[test]
    fn test_fade_scaled() {
        let val = fade_scaled(100);
        assert_eq!(val, (100 * 100 / 0xB0) as u16);
    }
}
