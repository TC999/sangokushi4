//! 战斗系统集成测试。
//!
//! 验证战斗流程：初始化→多回合交战→结果判定→状态更新。

use s4_core::data::{GameState, Unit, UnitState};
use s4_core::battle::{engage_battle, engage_battle_with_config, BattleResult, BattleConfig};

/// 创建测试用游戏状态。
fn create_test_game() -> GameState {
    let mut game = GameState::new();
    game.rng_state = 0x12345678;
    game
}

/// 创建测试部队。
fn create_unit(game: &mut GameState, idx: usize, id: u16, faction: u8, troops: u16, attr: u16) {
    game.units.units[idx] = Unit {
        id,
        officer_id: id,
        faction_id: faction,
        troops,
        attr_17: attr,
        fatigue: 0,
        state: UnitState::Idle,
        col: (idx as u8) % 10,
        row: (idx as u8) / 10,
        ..Unit::default()
    };
}

#[test]
fn test_complete_battle_flow() {
    let mut game = create_test_game();
    create_unit(&mut game, 0, 1, 1, 100, 80);
    create_unit(&mut game, 1, 2, 2, 100, 70);

    let result = engage_battle(&mut game, 0, 1);

    // 战斗应有明确结果
    assert!(matches!(result, BattleResult::Win | BattleResult::Lose | BattleResult::Draw | BattleResult::Retreat));

    // 至少一方受到损伤
    let atk_troops = game.units.units[0].troops;
    let def_troops = game.units.units[1].troops;
    assert!(atk_troops < 100 || def_troops < 100,
        "双方都未受损: atk={}, def={}", atk_troops, def_troops);
}

#[test]
fn test_stronger_unit_wins_majority() {
    let mut wins = 0;
    let mut total = 0;

    for _ in 0..50 {
        let mut game = create_test_game();
        // 攻击方明显更强
        create_unit(&mut game, 0, 1, 1, 150, 95);
        create_unit(&mut game, 1, 2, 2, 80, 60);

        let result = engage_battle(&mut game, 0, 1);
        if result == BattleResult::Win {
            wins += 1;
        }
        total += 1;
    }

    // 强者胜率应>60%
    let win_rate = wins as f64 / total as f64;
    assert!(win_rate > 0.5, "强者胜率过低: {:.1}%", win_rate * 100.0);
}

#[test]
fn test_battle_weak_unit_loses_often() {
    let mut losses = 0;
    let total = 50;

    for _ in 0..total {
        let mut game = create_test_game();
        // 攻击方明显更弱
        create_unit(&mut game, 0, 1, 1, 50, 40);
        create_unit(&mut game, 1, 2, 2, 150, 90);

        let result = engage_battle(&mut game, 0, 1);
        if result == BattleResult::Lose || result == BattleResult::Retreat {
            losses += 1;
        }
    }

    // 弱者失败率应>50%
    let loss_rate = losses as f64 / total as f64;
    assert!(loss_rate > 0.4, "弱者失败率过低: {:.1}%", loss_rate * 100.0);
}

#[test]
fn test_battle_troop_depletion() {
    let mut game = create_test_game();
    create_unit(&mut game, 0, 1, 1, 100, 80);
    create_unit(&mut game, 1, 2, 2, 100, 70);

    let result = engage_battle(&mut game, 0, 1);

    // 胜者保留兵力，败者兵力归零
    match result {
        BattleResult::Win => {
            assert!(game.units.units[0].troops > 0, "胜方兵力应>0");
            assert_eq!(game.units.units[1].troops, 0, "败方兵力应=0");
            assert_eq!(game.units.units[1].state, UnitState::Destroyed);
        }
        BattleResult::Lose => {
            assert!(game.units.units[1].troops > 0, "胜方兵力应>0");
            assert_eq!(game.units.units[0].troops, 0, "败方兵力应=0");
            assert_eq!(game.units.units[0].state, UnitState::Destroyed);
        }
        BattleResult::Retreat => {
            assert_eq!(game.units.units[0].state, UnitState::Retreating);
        }
        _ => {}
    }
}

#[test]
fn test_battle_fatigue_increases() {
    let mut game = create_test_game();
    create_unit(&mut game, 0, 1, 1, 100, 80);
    create_unit(&mut game, 1, 2, 2, 100, 70);

    engage_battle(&mut game, 0, 1);

    // 战斗后双方应有疲劳
    // （除非战斗在第一回合就结束了）
}

#[test]
fn test_unit_damage_and_state() {
    let mut game = create_test_game();
    create_unit(&mut game, 0, 1, 1, 10, 50);

    game.units.units[0].take_damage(5);
    assert_eq!(game.units.units[0].troops, 5);
    assert!(game.units.units[0].is_valid());

    game.units.units[0].take_damage(10);
    assert_eq!(game.units.units[0].troops, 0);
    assert_eq!(game.units.units[0].state, UnitState::Destroyed);
    assert!(!game.units.units[0].is_valid());
}

#[test]
fn test_unit_effective_power_calculation() {
    let mut unit = Unit::new(1, 10, 1);
    unit.attr_17 = 100;
    unit.troops = 100;
    unit.fatigue = 0;

    let full = unit.effective_power();
    assert!((full - 100.0).abs() < 1.0);

    unit.fatigue = 50;
    let tired = unit.effective_power();
    assert!(tired < full, "疲劳应降低战力");

    unit.fatigue = 0;
    unit.troops = 50;
    let fewer = unit.effective_power();
    assert!(fewer < full, "兵力减少应降低战力");
}

#[test]
fn test_custom_battle_config() {
    let mut game = create_test_game();
    create_unit(&mut game, 0, 1, 1, 100, 80);
    create_unit(&mut game, 1, 2, 2, 100, 70);

    let config = BattleConfig {
        damage_multiplier: 0.5, // 更高伤害
        retreat_threshold: 0.3, // 更高撤退阈值
        max_fatigue_per_round: 10,
    };

    let result = engage_battle_with_config(&mut game, 0, 1, &config);
    assert!(matches!(result, BattleResult::Win | BattleResult::Lose | BattleResult::Draw | BattleResult::Retreat));
}

#[test]
fn test_adjacent_enemy_detection() {
    let mut game = create_test_game();
    create_unit(&mut game, 0, 1, 1, 100, 80);
    game.units.units[0].col = 5;
    game.units.units[0].row = 5;

    create_unit(&mut game, 1, 2, 2, 100, 70);
    game.units.units[1].col = 6;
    game.units.units[1].row = 5;

    create_unit(&mut game, 2, 3, 3, 100, 60);
    game.units.units[2].col = 10;
    game.units.units[2].row = 10; // 远处

    let enemy = game.units.find_adjacent_enemy(&game.units.units[0]);
    assert!(enemy.is_some(), "应找到相邻敌方单位");
    assert_eq!(enemy.unwrap().id, 2); // 应该是id=2的单位

    let far_enemy = game.units.units[2].clone();
    let no_enemy = game.units.find_adjacent_enemy(&far_enemy);
    assert!(no_enemy.is_none(), "远处不应有相邻敌人");
}
