//! City data structures and resource accessors.
//!
//! 规格文档引用:
//!   /17-resource-accumulation-and-depletion — 资源积累与消耗
//!   /15-bitfield-based-stat-accessors — 城市位域访问器

/// 城市数量上限。
pub const CITY_COUNT: usize = 42;
/// 实际可游戏城市数。
pub const CITY_PLAYABLE: usize = 21;

/// 城市记录 — 87字节(0x57)。
///
/// 规格: /17-resource-accumulation-and-depletion
/// 所有偏移通过 `city_ptr + offset` 访问。
#[derive(Debug, Clone)]
pub struct City {
    pub id: u8,
    /// 偏移+0x00: 所有权 (0=无, 5=过渡)
    pub ownership: u8,
    /// 偏移+0x01: 武将列表首指针
    pub officer_list: u16,
    /// 偏移+0x0B: 任命槽位1（太守）
    pub appoint_1: u16,
    /// 偏移+0x0D: 任命槽位2（军师）
    pub appoint_2: u16,
    /// 偏移+0x0F: 主要资源（金）
    pub off_0f: u16,
    /// 偏移+0x11: 次要资源（粮）
    pub off_11: u16,
    /// 偏移+0x13: 第三资源（兵）
    pub off_13: u16,
    /// 偏移+0x15: 治理属性
    pub off_15: u16,
    /// 偏移+0x17: 武将字节
    pub off_17: u8,
    /// 偏移+0x18: 发展参数A
    pub off_18: u8,
    /// 偏移+0x19: 发展参数B
    pub off_19: u8,
    /// 偏移+0x1A: 条件属性
    pub off_1a: u8,
    /// 偏移+0x1C: 乘数属性
    pub off_1c: u8,
    /// 偏移+0x1D: 可消耗资源A
    pub off_1d: u8,
    /// 偏移+0x1E: 可消耗资源B
    pub off_1e: u8,
    /// 偏移+0x1F: 加权属性输入
    pub off_1f: u8,
    /// 偏移+0x20: 资源属性20
    pub off_20: u16,
    /// 偏移+0x22: 资源属性22
    pub off_22: u16,
    /// 偏移+0x24: 资源属性24
    pub off_24: u16,
    /// 偏移+0x26: 资源属性26
    pub off_26: u16,
    /// 偏移+0x28-0x2E: 扩展可消耗属性
    pub ext_deplete: [u8; 7],
    /// 偏移+0x37: 武将名册（每行4字节=2武将ID）
    pub roster: Vec<u16>,
    /// 位域标志
    pub bitflags: u16,
}

impl Default for City {
    fn default() -> Self {
        Self {
            id: 0,
            ownership: 0,
            officer_list: 0xFFFF,
            appoint_1: 0xFFFF,
            appoint_2: 0xFFFF,
            off_0f: 0,
            off_11: 0,
            off_13: 0,
            off_15: 0,
            off_17: 0,
            off_18: 0,
            off_19: 0,
            off_1a: 0,
            off_1c: 0,
            off_1d: 0,
            off_1e: 0,
            off_1f: 0,
            off_20: 0,
            off_22: 0,
            off_24: 0,
            off_26: 0,
            ext_deplete: [0; 7],
            roster: vec![],
            bitflags: 0,
        }
    }
}

impl City {
    /// 城市是否有效（有所有权）。
    pub fn is_valid(&self) -> bool {
        self.ownership != 0
    }

    /// 获取指定偏移的word值（off_11, off_13）。
    pub fn get_word_off(&self, offset: u8) -> u16 {
        match offset {
            0x0F => self.off_0f,
            0x11 => self.off_11,
            0x13 => self.off_13,
            0x15 => self.off_15,
            0x20 => self.off_20,
            0x22 => self.off_22,
            0x24 => self.off_24,
            0x26 => self.off_26,
            _ => 0,
        }
    }

    /// 设置指定偏移的word值。
    pub fn set_word_off(&mut self, offset: u8, val: u16) {
        match offset {
            0x0F => self.off_0f = val,
            0x11 => self.off_11 = val,
            0x13 => self.off_13 = val,
            0x15 => self.off_15 = val,
            0x20 => self.off_20 = val,
            0x22 => self.off_22 = val,
            0x24 => self.off_24 = val,
            0x26 => self.off_26 = val,
            _ => {}
        }
    }

    /// 递减指定偏移的byte值（消费资源）。
    /// 规格: /17-resource-accumulation-and-depletion — stat_dec系列
    pub fn stat_dec(&mut self, offset: u8, amount: u8) {
        match offset {
            0x1D => self.off_1d = self.off_1d.saturating_sub(amount),
            0x1E => self.off_1e = self.off_1e.saturating_sub(amount),
            0x18 => self.off_18 = self.off_18.saturating_sub(amount),
            0x19 => self.off_19 = self.off_19.saturating_sub(amount),
            _ => {}
        }
    }

    /// 读取指定偏移的byte值。
    pub fn stat_read(&self, offset: u8) -> u8 {
        match offset {
            0x18 => self.off_18,
            0x19 => self.off_19,
            0x1A => self.off_1a,
            0x1C => self.off_1c,
            0x1D => self.off_1d,
            0x1E => self.off_1e,
            0x1F => self.off_1f,
            _ => 0,
        }
    }

    /// 重置城市资源数据（城市被征服后）。
    /// 规格: /17-resource-accumulation-and-depletion — city_data_reset
    pub fn reset(&mut self) {
        self.ownership = 0;
        self.officer_list = 0xFFFF;
        self.appoint_1 = 0;
        self.appoint_2 = 0;
        self.off_17 = 0;
        self.bitflags = 0;
        // ... 其他字段清零
    }

    /// 计算发展产出公式。
    /// 规格: /17-resource-accumulation-and-depletion — compute_stat_formula
    /// 公式: ((off18 >> 1) * (off1c + 1) + off18) * governance_factor
    pub fn compute_development_output(&self, governance: u16) -> u32 {
        let base = self.off_18 as u32;
        let multiplier = (self.off_1c as u32) + 1;
        let result = (base / 2) * multiplier + base;
        result * governance as u32
    }

    /// 计算饱和官员产出。公式: officer_value * 125。
    /// 规格: compute_stat_mul125
    pub fn compute_officer_production(&self, officer_value: u32) -> u32 {
        let raw = officer_value * 125;
        raw.min(u32::MAX) // 饱和
    }
}

/// 城市存储。
/// 全局无效城市静态实例。
static INVALID_CITY: std::sync::OnceLock<City> = std::sync::OnceLock::new();

pub struct CityStore {
    pub cities: Vec<City>,
}

impl CityStore {
    pub fn new() -> Self {
        Self {
            cities: vec![City::default(); CITY_COUNT],
        }
    }

    pub fn get(&self, id: u8) -> &City {
        if (id as usize) < CITY_COUNT {
            &self.cities[id as usize]
        } else {
            INVALID_CITY.get_or_init(|| City::default())
        }
    }

    pub fn get_mut(&mut self, id: u8) -> &mut City {
        &mut self.cities[id as usize]
    }
}
