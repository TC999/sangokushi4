//! Battle system.
//!
//! 规格文档引用:
//!   /11-round-processing-and-turn-dispatch — 战斗流程
//!   /24-combat-ui-layout-and-input — 战斗UI

use crate::data::GameState;

/// 战斗结果。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BattleResult {
    Win,
    Lose,
    Draw,
    Retreat,
}

/// 发起战斗。
pub fn engage_battle(game: &mut GameState, attacker_unit: usize, defender_unit: usize) -> BattleResult {
    let attacker = &game.units.units[attacker_unit];
    let defender = &game.units.units[defender_unit];
    
    // 简化战斗计算
    let atk_power = attacker.attr_17 as i32;
    let def_power = defender.attr_17 as i32;
    let random_factor = game.random_range(20) as i32;
    
    let result = atk_power + random_factor - def_power;
    
    if result > 0 {
        BattleResult::Win
    } else if result < -5 {
        BattleResult::Lose
    } else {
        BattleResult::Draw
    }
}
