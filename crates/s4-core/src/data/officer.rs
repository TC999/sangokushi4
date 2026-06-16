//! Officer data structures and bitfield accessors.
//!
//! 规格文档引用:
//!   /15-bitfield-based-stat-accessors — 三层访问器架构
//!   /16-officer-slot-management — 槽位管理

/// 武将ID，范围1-400，0表示无效。
pub const OFFICER_ID_MIN: u16 = 1;
pub const OFFICER_ID_MAX: u16 = 400;
pub const OFFICER_COUNT: usize = 401;

/// 武将无效哨兵值。
pub const OFFICER_INVALID: u16 = 0xFFFF;

/// 武将分类类别（get_category返回值范围）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum OfficerCategory {
    Loyal = 0,
    Rebellious = 1,
    Captured = 2,
    Independent = 3,
    Remnant = 4,
    Special1 = 5,
    Special2 = 6,
    Reserved7 = 7,
    Reserved8 = 8,
    Reserved9 = 9,
    Special3 = 10,
}

/// 武将记录 — 布局A（20字节原始布局）。
///
/// 规格: /15-bitfield-based-stat-accessors
/// 偏移表:
///   +0x03: 分类字节（高4位+低4位=两个子分类）
///   +0x04: INT属性
///   +0x05: POL属性
///   +0x06: CHR属性
///   +0x07: 标志字A
///   +0x09: 标志字B（bit15=主禁用标志=死亡/无能力）
///   +0x0D: 忠诚度 (0xFF=无效哨兵)
///   +0x0E: 标志字
///   +0x10: 标志字
///   +0x12: 标志字
#[derive(Debug, Clone)]
pub struct Officer {
    pub id: u16,
    pub off3: u8,
    pub off4: u8,    // INT
    pub off5: u8,    // POL
    pub off6: u8,    // CHR
    pub off7: u16,   // 标志字A
    pub off9: u16,   // 标志字B（bit15=主禁用）
    pub loyalty: u8,
    pub flag_e: u16,
    pub flag_10: u16,
    pub flag_12: u16,
}

impl Default for Officer {
    fn default() -> Self {
        Self {
            id: 0,
            off3: 0,
            off4: 0,
            off5: 0,
            off6: 0,
            off7: 0,
            off9: 0,
            loyalty: 0xFF,
            flag_e: 0,
            flag_10: 0,
            flag_12: 0,
        }
    }
}

impl Officer {
    /// 创建无效武将（哨兵值）。
    pub fn invalid() -> Self {
        Self::default()
    }

    /// 武将是否有效。
    pub fn is_valid(&self) -> bool {
        self.id != 0 && self.id <= OFFICER_ID_MAX
    }

    /// 武将是否已死亡/无能力（off9 bit15）。
    /// 规格: /16-officer-slot-management — get_off7_cond
    pub fn is_disabled(&self) -> bool {
        self.off9 & 0x8000 != 0
    }

    /// 获取off7条件值。如果off9 bit15置位，返回0。
    pub fn get_off7_cond(&self) -> u16 {
        if self.is_disabled() { 0 } else { self.off7 }
    }

    // === 位域操作（规格: /15-bitfield-based-stat-accessors）===

    /// 测试flag_e的指定位。
    pub fn test_flag_e(&self, mask: u16) -> bool {
        self.flag_e & mask != 0
    }

    /// 测试flag_10的指定位。
    pub fn test_flag_10(&self, mask: u16) -> bool {
        self.flag_10 & mask != 0
    }

    /// 设置flag_e的指定位（OR操作）。
    pub fn set_flag_e(&mut self, mask: u16) {
        self.flag_e |= mask;
    }

    /// 清除flag_e的指定位（AND-NOT操作）。
    pub fn clear_flag_e(&mut self, mask: u16) {
        self.flag_e &= !mask;
    }

    /// 测试flag_12的指定位。
    pub fn test_flag_12(&self, mask: u16) -> bool {
        self.flag_12 & mask != 0
    }

    /// 设置flag_12的指定位。
    pub fn set_flag_12(&mut self, mask: u16) {
        self.flag_12 |= mask;
    }

    /// 清除flag_12的指定位。
    pub fn clear_flag_12(&mut self, mask: u16) {
        self.flag_12 &= !mask;
    }

    /// 在flag_e和flag_10上测试任意一个字的位（双字测试）。
    /// 规格: flags_test_any
    pub fn test_flags_any(&self, mask_e: u16, mask_10: u16) -> bool {
        self.flag_e & mask_e != 0 || self.flag_10 & mask_10 != 0
    }

    /// 跨两个标志字设置位（双字OR）。
    pub fn set_flags_both(&mut self, mask_e: u16, mask_10: u16) {
        self.flag_e |= mask_e;
        self.flag_10 |= mask_10;
    }

    // === 位比率计算（规格: /15-bitfield-based-stat-accessors）===

    /// 计算位比率。公式: (DX & AX) / (DX & -DX)，其中DX=掩码，AX=数据。
    pub fn bit_ratio(data: u16, mask: u16) -> u16 {
        if mask == 0 {
            return data & 0xFF00;
        }
        let lowest_bit = mask & mask.wrapping_neg(); // x & -x 取最低位
        (mask & data) / lowest_bit
    }

    /// 位比率是否等于7。
    pub fn bit_ratio_is_7(data: u16, mask: u16) -> bool {
        Self::bit_ratio(data, mask) == 7
    }

    /// 位比率是否等于8。
    pub fn bit_ratio_is_8(data: u16, mask: u16) -> bool {
        Self::bit_ratio(data, mask) == 8
    }

    /// 从位比率推导属性值。公式: (0x14 - ratio) × 5。
    /// 比率0→100(最大), 比率20→0(最小)。
    pub fn calc_stat_from_ratio(ratio: u16) -> u16 {
        if ratio >= 20 { 0 } else { (20 - ratio) * 5 }
    }

    /// 从off3半字节提取上半分类（高4位）。
    pub fn nibble_hi_get(off3: u8) -> u8 {
        let val = off3 >> 4;
        if val == 0xF { 0xFF } else { val }
    }

    /// 从off3半字节提取下下半分类（低4位 + 4偏移）。
    pub fn nibble_lo_get(off3: u8) -> u8 {
        let val = off3 & 0x0F;
        if val == 0xF { 0xFF } else { val + 4 }
    }

    /// 获取off_1a值（布局C中的条件属性）。
    pub fn off_1a(&self) -> u8 {
        (self.off3 >> 4) | ((self.off3 & 0x0F) << 4) // 简化映射
    }

    /// 综合分类判断（get_category的简化版本）。
    pub fn get_category(&self) -> OfficerCategory {
        let r7 = Self::bit_ratio_is_7(self.off7, self.off9);
        let r8 = Self::bit_ratio_is_8(self.off7, self.off9);
        if r7 || r8 { return OfficerCategory::Loyal; }
        if self.test_flag_12(0x1000) { return OfficerCategory::Rebellious; }
        if self.off9 & 0x8000 != 0 { return OfficerCategory::Captured; }
        // ... 更多条件
        OfficerCategory::Independent
    }
}

/// 武将扩展记录 — 布局B（30字节）。
#[derive(Debug, Clone, Default)]
pub struct OfficerExtB {
    pub field_14: u16,
    pub field_16: u16,
    pub field_18: u8,
}

/// 武将替代记录 — 布局C（32字节）。
///
/// 规格: /15-bitfield-based-stat-accessors — 偏移3处半字节打包
#[derive(Debug, Clone)]
pub struct OfficerExtC {
    pub alt_off0: u8,
    pub war: u8,         // 军事属性
    pub alt_off4: u8,
    pub nibble_byte: u8,  // 偏移3处: 高4位+低4位=两个子分类
    pub flag_d: u16,
}

impl Default for OfficerExtC {
    fn default() -> Self {
        Self { alt_off0: 0, war: 0, alt_off4: 0, nibble_byte: 0, flag_d: 0 }
    }
}

/// 全局无效武将静态实例。
static INVALID_OFFICER: std::sync::OnceLock<Officer> = std::sync::OnceLock::new();

/// 武将存储。
pub struct OfficerStore {
    pub officers: Vec<Officer>,
    pub ext_b: Vec<OfficerExtB>,
    pub ext_c: Vec<OfficerExtC>,
}

impl OfficerStore {
    pub fn new() -> Self {
        Self {
            officers: vec![Officer::default(); OFFICER_COUNT],
            ext_b: vec![OfficerExtB::default(); OFFICER_COUNT],
            ext_c: vec![OfficerExtC::default(); OFFICER_COUNT],
        }
    }

    /// 通过ID获取武将引用（1-based索引，0返回哨兵）。
    pub fn get(&self, id: u16) -> &Officer {
        if id == 0 || id as usize > OFFICER_COUNT {
            INVALID_OFFICER.get_or_init(|| Officer::invalid())
        } else {
            &self.officers[id as usize]
        }
    }

    /// 通过ID获取武将可变引用。
    pub fn get_mut(&mut self, id: u16) -> &mut Officer {
        if id == 0 || id as usize > OFFICER_COUNT {
            panic!("Invalid officer ID: {}", id);
        }
        &mut self.officers[id as usize]
    }
}
