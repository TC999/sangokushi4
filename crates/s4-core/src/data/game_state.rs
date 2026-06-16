//! Global game state container.
//!
//! 规格文档引用:
//!   架构设计文档 §五.2 — GameState统一状态容器

use super::officer::OfficerStore;
use super::city::CityStore;
use super::map::GameMap;
use super::faction::FactionStore;
use super::unit::{Unit, UnitStore};

/// 游戏全局状态。
///
/// 替代原版所有固定地址全局变量（0x39EC, 0x5686_xxxx等）。
pub struct GameState {
    // === 核心数据 ===
    pub officers: OfficerStore,
    pub cities: CityStore,
    pub factions: FactionStore,
    pub map: GameMap,
    pub units: UnitStore,

    // === 时间系统 ===
    /// 当前回合号。
    pub turn: u16,
    /// 年份。
    pub year: u16,
    /// 月份。
    pub month: u8,
    /// 日期。
    pub day: u8,
    /// 游戏时间刻度 (hours×60+min)×75+sec-150。
    /// 规格: /7-overlay-loading-and-dispatch — date_calc
    pub time_ticks: u32,

    // === 场景状态 ===
    /// 当前游戏模式 (0=战略, 3=战斗A, 4=战斗B)。
    pub game_mode: u8,
    /// 活跃部队索引。
    pub active_unit_idx: Option<usize>,
    /// 循环退出标志（对应0x39EC）。
    pub exit_flag: bool,
    /// 常规状态标志（对应game_check_flag）。
    pub normal_state: bool,
    /// 替代处理模式（对应0x8564）。
    pub alternate_mode: bool,

    // === 随机数 ===
    pub rng_state: u32,

    // === 武将槽位表（全局32项） ===
    /// 规格: /16-officer-slot-management — 全局武将槽位表
    pub global_slots: [GlobalSlot; 32],

    // === 对话框槽位属性 ===
    pub dialog_slot_attr15: u16,
    pub dialog_slot_attr1e: u16,
}

/// 全局槽位条目。
#[derive(Debug, Clone, Default)]
pub struct GlobalSlot {
    pub officer_id: u16,
    pub vtable_index: u8,
    pub city_id: u8,
}

impl Default for GameState {
    fn default() -> Self {
        Self {
            officers: OfficerStore::new(),
            cities: CityStore::new(),
            factions: FactionStore::new(),
            map: GameMap::default(),
            units: UnitStore::new(),

            turn: 0,
            year: 0,
            month: 1,
            day: 1,
            time_ticks: 0,

            game_mode: 0,
            active_unit_idx: None,
            exit_flag: false,
            normal_state: true,
            alternate_mode: false,

            rng_state: 0x12345678,

            global_slots: std::array::from_fn(|_| GlobalSlot::default()),

            dialog_slot_attr15: 0,
            dialog_slot_attr1e: 0,
        }
    }
}

impl GameState {
    /// 创建新游戏状态。
    pub fn new() -> Self {
        Self::default()
    }

    /// 生成伪随机u32（LCG算法）。
    /// 规格: /12-turn-execution-pipeline — 随机化
    pub fn random_u32(&mut self) -> u32 {
        self.rng_state = self.rng_state.wrapping_mul(1664525).wrapping_add(1013904223);
        self.rng_state
    }

    /// 生成指定范围的随机数 [0, bound)。
    pub fn random_range(&mut self, bound: u32) -> u32 {
        if bound == 0 { return 0; }
        self.random_u32() % bound
    }

    /// 随机判定（百分比概率）。
    pub fn random_chance(&mut self, percent: u32) -> bool {
        self.random_range(100) < percent
    }

    /// 获取活跃部队引用。
    pub fn active_unit(&self) -> Option<&Unit> {
        self.active_unit_idx.and_then(|i| self.units.units.get(i))
    }

    /// 获取活跃部队可变引用。
    pub fn active_unit_mut(&mut self) -> Option<&mut Unit> {
        self.active_unit_idx.and_then(|i| self.units.units.get_mut(i))
    }

    /// 获取当前势力ID。
    pub fn current_faction_id(&self) -> u8 {
        // 从当前回合推导
        ((self.turn - 1) % 7 + 1) as u8
    }

    /// 游戏检查标志（对应main_game_check_flag）。
    /// 规格: /11-round-processing-and-turn-dispatch
    pub fn check_normal_state(&self) -> bool {
        self.normal_state
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_game_state_creation() {
        let state = GameState::new();
        assert_eq!(state.turn, 0);
        assert_eq!(state.map.width, 20);
        assert_eq!(state.map.height, 11);
    }

    #[test]
    fn test_random_range() {
        let mut state = GameState::new();
        for _ in 0..100 {
            let val = state.random_range(10);
            assert!(val < 10);
        }
    }
}
