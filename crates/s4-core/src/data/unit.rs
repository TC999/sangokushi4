//! Unit (部队) data structures.
//!
//! 规格文档引用:
//!   /14-move-evaluation-and-pathfinding — 部队移动
//!   /12-turn-execution-pipeline — 部队回合执行

/// 最大活跃部队数。
pub const MAX_UNITS: usize = 64;

/// 部队标志位。
pub const UNIT_FLAG5: u8 = 0x20;
pub const UNIT_FLAG6: u8 = 0x40;

/// 部队记录。
///
/// 规格: /14-move-evaluation-and-pathfinding
/// 存储在地图网格中的活跃作战单位。
#[derive(Debug, Clone)]
pub struct Unit {
    pub id: u16,
    pub officer_id: u16,
    pub col: u8,
    pub row: u8,
    pub flags: u8,
    pub attr_1b: u8,      // 移动力
    pub attr_17: u16,     // 综合属性
    pub faction_id: u8,
}

impl Default for Unit {
    fn default() -> Self {
        Self {
            id: 0,
            officer_id: 0xFFFF,
            col: 0,
            row: 0,
            flags: 0,
            attr_1b: 0,
            attr_17: 0,
            faction_id: 0,
        }
    }
}

impl Unit {
    pub fn is_valid(&self) -> bool {
        self.officer_id != 0xFFFF
    }

    /// 是否有flag5或flag6（战斗/部署状态）。
    /// 规格: /14-move-evaluation-and-pathfinding — eval_two_phase
    pub fn has_combat_flags(&self) -> bool {
        self.flags & (UNIT_FLAG5 | UNIT_FLAG6) != 0
    }

    /// 是否设置了flag5。
    pub fn has_flag5(&self) -> bool {
        self.flags & UNIT_FLAG5 != 0
    }

    /// 是否设置了flag6（扩展移动能力）。
    pub fn has_flag6(&self) -> bool {
        self.flags & UNIT_FLAG6 != 0
    }
}

/// 活跃部队存储。
pub struct UnitStore {
    pub units: Vec<Unit>,
}

impl UnitStore {
    pub fn new() -> Self {
        Self {
            units: vec![Unit::default(); MAX_UNITS],
        }
    }

    /// 查找指定位置的部队。
    pub fn find_at(&self, col: u8, row: u8) -> Option<&Unit> {
        self.units.iter().find(|u| u.is_valid() && u.col == col && u.row == row)
    }

    /// 查找指定武将的部队。
    pub fn find_by_officer(&self, officer_id: u16) -> Option<&Unit> {
        self.units.iter().find(|u| u.is_valid() && u.officer_id == officer_id)
    }

    /// 活跃部队数量。
    pub fn active_count(&self) -> usize {
        self.units.iter().filter(|u| u.is_valid()).count()
    }
}
