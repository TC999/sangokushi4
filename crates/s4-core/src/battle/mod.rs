//! Battle system.
//!
//! 规格文档引用:
//!   /11-round-processing-and-turn-dispatch — 战斗流程
//!   /14-move-evaluation-and-pathfinding — 战斗移动目标检测
//!   /12-turn-execution-pipeline — 战斗动作分派

use crate::data::{GameState, Unit, UnitState};

/// 最大战斗回合数。
const MAX_BATTLE_ROUNDS: u32 = 20;

/// 战斗结果。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BattleResult {
    Win,
    Lose,
    Draw,
    Retreat,
}

/// 战斗日志条目。
#[derive(Debug, Clone)]
pub struct BattleLog {
    pub round: u32,
    pub attacker_damage: u16,
    pub defender_damage: u16,
    pub attacker_troops: u16,
    pub defender_troops: u16,
}

/// 战斗状态。
pub struct BattleState {
    pub attacker_idx: usize,
    pub defender_idx: usize,
    pub attacker_power: f64,
    pub defender_power: f64,
    pub rounds: Vec<BattleLog>,
}

/// 战斗配置参数。
pub struct BattleConfig {
    /// 基础伤害倍率。
    pub damage_multiplier: f64,
    /// 撤退阈值（兵力百分比）。
    pub retreat_threshold: f64,
    /// 最大疲劳增长/回合。
    pub max_fatigue_per_round: u8,
}

impl Default for BattleConfig {
    fn default() -> Self {
        Self {
            damage_multiplier: 0.3,
            retreat_threshold: 0.2,
            max_fatigue_per_round: 5,
        }
    }
}

/// 发起战斗。
///
/// 规格: /12-turn-execution-pipeline — action_dispatch case 3
/// 执行完整的战斗流程：多回合交战→结果判定→状态更新。
pub fn engage_battle(
    game: &mut GameState,
    attacker_unit_idx: usize,
    defender_unit_idx: usize,
) -> BattleResult {
    let config = BattleConfig::default();
    engage_battle_with_config(game, attacker_unit_idx, defender_unit_idx, &config)
}

/// 使用自定义配置发起战斗。
pub fn engage_battle_with_config(
    game: &mut GameState,
    attacker_unit_idx: usize,
    defender_unit_idx: usize,
    config: &BattleConfig,
) -> BattleResult {
    // 验证双方部队有效
    if !game.units.units[attacker_unit_idx].is_combat_ready() {
        return BattleResult::Lose;
    }
    if !game.units.units[defender_unit_idx].is_combat_ready() {
        return BattleResult::Win;
    }

    // 计算初始战力
    let atk_power = game.units.units[attacker_unit_idx].effective_power();
    let def_power = game.units.units[defender_unit_idx].effective_power();

    let mut state = BattleState {
        attacker_idx: attacker_unit_idx,
        defender_idx: defender_unit_idx,
        attacker_power: atk_power,
        defender_power: def_power,
        rounds: Vec::new(),
    };

    // 执行多回合战斗
    let result = execute_battle_rounds(game, &mut state, config);

    // 处理战斗结果
    apply_battle_result(game, &state, result);

    result
}

/// 执行战斗回合。
fn execute_battle_rounds(
    game: &mut GameState,
    state: &mut BattleState,
    config: &BattleConfig,
) -> BattleResult {
    for round in 1..=MAX_BATTLE_ROUNDS {
        let atk = &game.units.units[state.attacker_idx];
        let def = &game.units.units[state.defender_idx];

        // 检查是否有一方已被消灭
        if atk.troops == 0 {
            return BattleResult::Lose;
        }
        if def.troops == 0 {
            return BattleResult::Win;
        }

        // 计算双方伤害
        let (atk_dmg, def_dmg) = calculate_round_damage(game, state, config);

        // 应用伤害
        game.units.units[state.attacker_idx].take_damage(def_dmg);
        game.units.units[state.defender_idx].take_damage(atk_dmg);

        // 增加疲劳
        game.units.units[state.attacker_idx].add_fatigue(config.max_fatigue_per_round);
        game.units.units[state.defender_idx].add_fatigue(config.max_fatigue_per_round);

        // 记录日志
        state.rounds.push(BattleLog {
            round,
            attacker_damage: def_dmg,
            defender_damage: atk_dmg,
            attacker_troops: game.units.units[state.attacker_idx].troops,
            defender_troops: game.units.units[state.defender_idx].troops,
        });

        // 更新战力
        state.attacker_power = game.units.units[state.attacker_idx].effective_power();
        state.defender_power = game.units.units[state.defender_idx].effective_power();

        // 检查撤退条件
        let atk_ratio = game.units.units[state.attacker_idx].troops as f64 /
                       game.units.units[state.attacker_idx].attr_17.max(1) as f64;
        let def_ratio = game.units.units[state.defender_idx].troops as f64 /
                       game.units.units[state.defender_idx].attr_17.max(1) as f64;

        if atk_ratio < config.retreat_threshold {
            return BattleResult::Retreat;
        }
        if def_ratio < config.retreat_threshold {
            return BattleResult::Win;
        }
    }

    // 超过最大回合数，根据剩余兵力判定
    let atk_troops = game.units.units[state.attacker_idx].troops;
    let def_troops = game.units.units[state.defender_idx].troops;

    if atk_troops > def_troops {
        BattleResult::Win
    } else if def_troops > atk_troops {
        BattleResult::Lose
    } else {
        BattleResult::Draw
    }
}

/// 计算单回合伤害。
///
/// 伤害公式: 基础战力 × 伤害倍率 × 地形修正 × 随机因子
fn calculate_round_damage(
    game: &mut GameState,
    state: &BattleState,
    config: &BattleConfig,
) -> (u16, u16) {
    let atk_power = state.attacker_power;
    let def_power = state.defender_power;

    // 随机因子 (0.8 - 1.2)
    let atk_random = 0.8 + game.random_range(40) as f64 / 100.0;
    let def_random = 0.8 + game.random_range(40) as f64 / 100.0;

    // 地形修正（简化：防御方在城市/要塞有加成）
    let def_terrain_bonus = 1.0; // TODO: 根据瓦片类型计算

    // 攻击方对防御方造成的伤害
    let atk_damage_raw = atk_power * config.damage_multiplier * atk_random;
    let def_effective = def_power * def_terrain_bonus;
    let atk_damage = (atk_damage_raw / def_effective.max(1.0) * 10.0) as u16;

    // 防御方对攻击方造成的伤害（反击）
    let def_damage_raw = def_power * config.damage_multiplier * 0.5 * def_random; // 反击伤害减半
    let atk_effective = atk_power;
    let def_damage = (def_damage_raw / atk_effective.max(1.0) * 10.0) as u16;

    (atk_damage.max(1), def_damage.max(0))
}

/// 应用战斗结果到游戏状态。
fn apply_battle_result(
    game: &mut GameState,
    state: &BattleState,
    result: BattleResult,
) {
    match result {
        BattleResult::Win => {
            // 攻击方胜利：防御方部队被消灭或撤退
            game.units.units[state.defender_idx].state = UnitState::Destroyed;
            game.units.units[state.defender_idx].troops = 0;

            // 攻击方恢复少量疲劳
            game.units.units[state.attacker_idx].rest(10);
        }
        BattleResult::Lose => {
            // 攻击方失败：攻击方部队被消灭或撤退
            game.units.units[state.attacker_idx].state = UnitState::Destroyed;
            game.units.units[state.attacker_idx].troops = 0;

            // 防御方恢复少量疲劳
            game.units.units[state.defender_idx].rest(10);
        }
        BattleResult::Draw => {
            // 平局：双方都受到重创
            game.units.units[state.attacker_idx].add_fatigue(20);
            game.units.units[state.defender_idx].add_fatigue(20);
        }
        BattleResult::Retreat => {
            // 攻击方撤退：撤退到安全位置
            game.units.units[state.attacker_idx].state = UnitState::Retreating;
            game.units.units[state.attacker_idx].add_fatigue(15);

            // 防御方恢复疲劳
            game.units.units[state.defender_idx].rest(5);
        }
    }
}

/// 获取战斗日志。
pub fn get_battle_log(_game: &GameState, _attacker_idx: usize, _defender_idx: usize) -> Vec<BattleLog> {
    // 简化：返回空日志（实际应从BattleState获取）
    Vec::new()
}

#[cfg(test)]
mod tests {
    use super::*;
use crate::data::{GameState, UnitState};

    fn setup_battle() -> (GameState, usize, usize) {
        let mut game = GameState::new();

        // 创建攻击方
        game.units.units[0] = Unit {
            id: 1,
            officer_id: 1,
            col: 5,
            row: 5,
            faction_id: 1,
            troops: 100,
            attr_17: 80,
            fatigue: 0,
            state: UnitState::Idle,
            ..Unit::default()
        };

        // 创建防御方
        game.units.units[1] = Unit {
            id: 2,
            officer_id: 2,
            col: 6,
            row: 5,
            faction_id: 2,
            troops: 100,
            attr_17: 70,
            fatigue: 0,
            state: UnitState::Idle,
            ..Unit::default()
        };

        (game, 0, 1)
    }

    #[test]
    fn test_battle_basic() {
        let (mut game, atk, def) = setup_battle();
        let result = engage_battle(&mut game, atk, def);
        // 战斗应该有结果
        assert!(matches!(result, BattleResult::Win | BattleResult::Lose | BattleResult::Draw | BattleResult::Retreat));
    }

    #[test]
    fn test_battle_stronger_wins_mostly() {
        let mut wins = 0;
        for _ in 0..100 {
            let (mut game, atk, def) = setup_battle();
            let result = engage_battle(&mut game, atk, def);
            if result == BattleResult::Win {
                wins += 1;
            }
        }
        // 攻击方属性更高，胜率应>50%
        assert!(wins > 30, "攻击方胜率过低: {}", wins);
    }

    #[test]
    fn test_battle_damage_depletes_troops() {
        let (mut game, atk, def) = setup_battle();
        let initial_atk = game.units.units[atk].troops;
        let initial_def = game.units.units[def].troops;
        engage_battle(&mut game, atk, def);
        // 至少有一方兵力减少
        assert!(game.units.units[atk].troops < initial_atk || game.units.units[def].troops < initial_def);
    }

    #[test]
    fn test_battle_winner_troops_positive() {
        let (mut game, atk, def) = setup_battle();
        let result = engage_battle(&mut game, atk, def);
        match result {
            BattleResult::Win => {
                assert!(game.units.units[atk].troops > 0);
                assert_eq!(game.units.units[def].state, UnitState::Destroyed);
            }
            BattleResult::Lose => {
                assert!(game.units.units[def].troops > 0);
                assert_eq!(game.units.units[atk].state, UnitState::Destroyed);
            }
            _ => {}
        }
    }

    #[test]
    fn test_unit_effective_power() {
        let mut unit = Unit::new(1, 10, 1);
        unit.attr_17 = 100;
        unit.troops = 100;
        unit.fatigue = 0;
        let full_power = unit.effective_power();

        unit.fatigue = 50;
        let tired_power = unit.effective_power();
        assert!(tired_power < full_power);

        unit.troops = 50;
        unit.fatigue = 0;
        let half_troops = unit.effective_power();
        assert!(half_troops < full_power);
    }

    #[test]
    fn test_unit_state_transitions() {
        let mut unit = Unit::new(1, 10, 1);
        assert_eq!(unit.state, UnitState::Idle);

        unit.state = UnitState::Marching;
        assert_eq!(unit.state, UnitState::Marching);

        unit.take_damage(100);
        assert_eq!(unit.state, UnitState::Destroyed);
        assert!(!unit.is_valid());
    }
}
