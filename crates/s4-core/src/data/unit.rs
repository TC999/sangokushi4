//! Unit (部队) data structures.
//!
//! 规格文档引用:
//!   /14-move-evaluation-and-pathfinding — 部队移动
//!   /12-turn-execution-pipeline — 部队回合执行
//!   /11-round-processing-and-turn-dispatch — 战斗流程

/// 最大活跃部队数。
pub const MAX_UNITS: usize = 64;

/// 部队标志位。
pub const UNIT_FLAG5: u8 = 0x20;
pub const UNIT_FLAG6: u8 = 0x40;

/// 部队状态。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnitState {
    /// 空闲/驻扎。
    Idle,
    /// 行军中。
    Marching,
    /// 战斗中。
    Fighting,
    /// 撤退中。
    Retreating,
    /// 被消灭。
    Destroyed,
}

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
    /// 兵力数量。
    pub troops: u16,
    /// 疲劳度 (0-100)。
    pub fatigue: u8,
    /// 部队状态。
    pub state: UnitState,
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
            troops: 0,
            fatigue: 0,
            state: UnitState::Idle,
        }
    }
}

impl Unit {
    /// 创建新部队。
    pub fn new(id: u16, officer_id: u16, faction_id: u8) -> Self {
        Self {
            id,
            officer_id,
            faction_id,
            troops: 100,
            state: UnitState::Idle,
            ..Self::default()
        }
    }

    pub fn is_valid(&self) -> bool {
        self.officer_id != 0xFFFF && self.state != UnitState::Destroyed
    }

    /// 部队是否还有战斗力。
    pub fn is_combat_ready(&self) -> bool {
        self.is_valid() && self.troops > 0 && self.state != UnitState::Destroyed
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

    /// 计算有效战斗力（考虑疲劳度）。
    pub fn effective_power(&self) -> f64 {
        let base = self.attr_17 as f64;
        let fatigue_factor = 1.0 - (self.fatigue as f64 / 200.0);
        let troop_factor = (self.troops as f64 / 100.0).min(1.0);
        base * fatigue_factor * troop_factor
    }

    /// 受到伤害后减少兵力。
    pub fn take_damage(&mut self, damage: u16) {
        self.troops = self.troops.saturating_sub(damage);
        if self.troops == 0 {
            self.state = UnitState::Destroyed;
        }
    }

    /// 增加疲劳度。
    pub fn add_fatigue(&mut self, amount: u8) {
        self.fatigue = self.fatigue.saturating_add(amount).min(100);
    }

    /// 恢复疲劳度。
    pub fn rest(&mut self, amount: u8) {
        self.fatigue = self.fatigue.saturating_sub(amount);
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

    /// 查找指定势力的部队。
    pub fn find_by_faction(&self, faction_id: u8) -> Vec<&Unit> {
        self.units.iter()
            .filter(|u| u.is_valid() && u.faction_id == faction_id)
            .collect()
    }

    /// 活跃部队数量。
    pub fn active_count(&self) -> usize {
        self.units.iter().filter(|u| u.is_valid()).count()
    }

    /// 查找相邻的敌方单位。
    pub fn find_adjacent_enemy(&self, unit: &Unit) -> Option<&Unit> {
        self.units.iter().find(|u| {
            u.is_valid()
            && u.faction_id != unit.faction_id
            && (u.col as i16 - unit.col as i16).abs() <= 1
            && (u.row as i16 - unit.row as i16).abs() <= 1
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_unit_creation() {
        let unit = Unit::new(1, 10, 1);
        assert!(unit.is_valid());
        assert_eq!(unit.troops, 100);
        assert_eq!(unit.state, UnitState::Idle);
    }

    #[test]
    fn test_unit_damage() {
        let mut unit = Unit::new(1, 10, 1);
        unit.take_damage(50);
        assert_eq!(unit.troops, 50);
        assert!(unit.is_valid());

        unit.take_damage(60);
        assert_eq!(unit.troops, 0);
        assert_eq!(unit.state, UnitState::Destroyed);
        assert!(!unit.is_valid());
    }

    #[test]
    fn test_unit_fatigue() {
        let mut unit = Unit::new(1, 10, 1);
        unit.add_fatigue(30);
        assert_eq!(unit.fatigue, 30);
        unit.add_fatigue(80);
        assert_eq!(unit.fatigue, 100); // capped at 100
        unit.rest(50);
        assert_eq!(unit.fatigue, 50);
    }

    #[test]
    fn test_effective_power() {
        let mut unit = Unit::new(1, 10, 1);
        unit.attr_17 = 100;
        unit.troops = 100;
        unit.fatigue = 0;
        let power = unit.effective_power();
        assert!((power - 100.0).abs() < 1.0);

        unit.fatigue = 50;
        let fatigued_power = unit.effective_power();
        assert!(fatigued_power < power);
    }
}
