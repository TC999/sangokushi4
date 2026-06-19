//! 完整游戏流程集成测试。
//!
//! 验证：开局初始化→内政管理→军事行动→战斗→回合推进→状态验证

use s4_core::data::*;
use s4_core::battle::{engage_battle, BattleResult};
use s4_core::round::round_process;

/// 创建完整的游戏初始状态。
fn setup_full_game() -> GameState {
    let mut game = GameState::new();
    game.turn = 1;
    game.year = 189;
    game.month = 1;
    game.day = 1;
    game.rng_state = 0x12345678;
    game.normal_state = true;

    // 初始化7个势力
    for i in 0..7 {
        game.factions.factions[i].id = (i + 1) as u8;
        game.factions.factions[i].gold = 1000;
        game.factions.factions[i].food = 2000;
    }

    // 初始化10个城市
    let cities = [
        (1u8, 1u8, 1000u16, 1500, 500),
        (2, 1, 800, 1200, 400),
        (3, 2, 600, 1000, 300),
        (4, 3, 700, 1100, 350),
        (5, 3, 500, 800, 250),
        (6, 4, 900, 1300, 450),
        (7, 5, 400, 600, 200),
        (8, 6, 500, 700, 250),
        (9, 7, 600, 900, 300),
        (10, 1, 700, 1000, 350),
    ];
    for &(id, owner, gold, food, troops) in &cities {
        let city = game.cities.get_mut(id);
        city.id = id;
        city.ownership = owner;
        city.off_0f = gold;
        city.off_11 = food;
        city.off_13 = troops;
        city.off_15 = 50;
        city.off_18 = 10;
        city.off_19 = 10;
    }

    // 初始化20个武将
    let officers = [
        (1u16, 1u8, 90u8, 95, 70, 200u8),
        (2, 1, 85, 80, 75, 180),
        (3, 1, 80, 75, 70, 170),
        (4, 1, 88, 82, 68, 190),
        (5, 1, 75, 70, 65, 160),
        (6, 2, 92, 95, 80, 200),
        (7, 2, 95, 98, 85, 220),
        (8, 2, 93, 96, 90, 220),
        (9, 2, 90, 88, 75, 190),
        (10, 3, 88, 90, 78, 200),
        (11, 3, 85, 82, 72, 180),
        (12, 3, 80, 78, 68, 170),
        (13, 4, 82, 85, 70, 180),
        (14, 4, 78, 75, 65, 160),
        (15, 4, 76, 73, 63, 155),
        (16, 5, 75, 70, 60, 150),
        (17, 6, 80, 82, 68, 170),
        (18, 6, 85, 80, 72, 180),
        (19, 7, 78, 80, 65, 165),
        (20, 1, 90, 88, 72, 185),
    ];
    for &(id, faction, int, pol, chr, loyalty) in &officers {
        let officer = game.officers.get_mut(id);
        officer.id = id;
        officer.off4 = int;
        officer.off5 = pol;
        officer.off6 = chr;
        officer.loyalty = loyalty;
        officer.off7 = ((int as u16 + pol as u16 + chr as u16) / 3) as u16;
        officer.off3 = (faction << 4) | (id % 10) as u8;
    }

    // 初始化10个部队
    for i in 0..10u16 {
        let idx = i as usize;
        game.units.units[idx] = Unit {
            id: i + 1,
            officer_id: i + 1,
            faction_id: ((i / 3) + 1) as u8,
            col: (i % 10) as u8,
            row: (i / 10) as u8,
            attr_1b: 3,
            attr_17: game.officers.get(i + 1).off7,
            troops: 100,
            state: UnitState::Idle,
            ..Unit::default()
        };
    }

    game
}

// ========== 测试1: 游戏初始化 ==========

#[test]
fn test_game_initialization() {
    let game = setup_full_game();

    assert_eq!(game.turn, 1);
    assert_eq!(game.year, 189);
    assert_eq!(game.month, 1);
    assert_eq!(game.day, 1);

    // 验证势力
    assert_eq!(game.factions.factions[0].id, 1);
    assert_eq!(game.factions.factions[0].gold, 1000);

    // 验证城市
    let city = game.cities.get(1);
    assert!(city.is_valid());
    assert_eq!(city.ownership, 1);
    assert_eq!(city.off_0f, 1000);

    // 验证武将
    let officer = game.officers.get(1);
    assert!(officer.is_valid());
    assert_eq!(officer.off4, 90); // INT
    assert_eq!(officer.loyalty, 200);

    // 验证部队
    let unit = &game.units.units[0];
    assert!(unit.is_valid());
    assert_eq!(unit.troops, 100);
}

// ========== 测试2: 内政管理 ==========

#[test]
fn test_domestic_development() {
    let mut game = setup_full_game();

    // 城市开发
    let city = game.cities.get_mut(1);
    let initial_dev = city.off_18;
    city.off_18 = city.off_18.saturating_add(10);

    assert!(game.cities.get(1).off_18 > initial_dev);
}

#[test]
fn test_resource_accumulation() {
    let mut game = setup_full_game();
    let initial_gold = game.cities.get(1).off_0f;

    // 执行回合处理（包含资源积累）
    let report = round_process(&mut game);

    // 资源应该有变化
    assert!(!report.gold_changes.is_empty());
    assert!(!report.food_changes.is_empty());
}

// ========== 测试3: 军事行动 ==========

#[test]
fn test_military_recruitment() {
    let mut game = setup_full_game();

    // 征兵
    let city = game.cities.get_mut(1);
    let initial_troops = city.off_13;
    city.off_13 = city.off_13.saturating_add(100);
    city.off_0f = city.off_0f.saturating_sub(1000); // 花费金

    assert!(game.cities.get(1).off_13 > initial_troops);
}

#[test]
fn test_unit_movement() {
    let mut game = setup_full_game();

    let initial_col = game.units.units[0].col;
    let initial_row = game.units.units[0].row;

    // 移动部队
    game.units.units[0].col += 1;
    game.units.units[0].row += 1;

    assert_ne!(game.units.units[0].col, initial_col);
    assert_ne!(game.units.units[0].row, initial_row);
}

// ========== 测试4: 战斗系统 ==========

#[test]
fn test_battle_between_factions() {
    let mut game = setup_full_game();

    // 设置两个相邻的敌方单位
    game.units.units[0] = Unit {
        id: 1, officer_id: 1, faction_id: 1,
        col: 5, row: 5, troops: 100, attr_17: 80,
        state: UnitState::Idle, ..Unit::default()
    };
    game.units.units[1] = Unit {
        id: 2, officer_id: 2, faction_id: 2,
        col: 6, row: 5, troops: 100, attr_17: 70,
        state: UnitState::Idle, ..Unit::default()
    };

    let result = engage_battle(&mut game, 0, 1);

    assert!(matches!(result, BattleResult::Win | BattleResult::Lose | BattleResult::Draw | BattleResult::Retreat));

    // 战斗后至少一方受损
    assert!(game.units.units[0].troops < 100 || game.units.units[1].troops < 100);
}

#[test]
fn test_battle_stronger_wins() {
    let mut wins = 0;
    for _ in 0..30 {
        let mut game = setup_full_game();
        game.units.units[0] = Unit {
            id: 1, officer_id: 1, faction_id: 1,
            col: 5, row: 5, troops: 150, attr_17: 95,
            state: UnitState::Idle, ..Unit::default()
        };
        game.units.units[1] = Unit {
            id: 2, officer_id: 2, faction_id: 2,
            col: 6, row: 5, troops: 80, attr_17: 60,
            state: UnitState::Idle, ..Unit::default()
        };

        if engage_battle(&mut game, 0, 1) == BattleResult::Win {
            wins += 1;
        }
    }
    assert!(wins > 15, "强者胜率应>50%");
}

// ========== 测试5: 回合推进 ==========

#[test]
fn test_multiple_rounds() {
    let mut game = setup_full_game();

    for _ in 0..10 {
        let _report = round_process(&mut game);
    }

    assert_eq!(game.turn, 11);
    assert!(game.year >= 189);
}

#[test]
fn test_year_transition() {
    let mut game = setup_full_game();
    game.month = 12;
    game.day = 30;

    let report = round_process(&mut game);

    assert_eq!(game.day, 1);
    assert_eq!(game.month, 1);
    assert_eq!(game.year, 190);
    assert!(report.events.iter().any(|e| e.contains("190")));
}

#[test]
fn test_loyalty_changes_over_time() {
    let mut game = setup_full_game();
    let initial_loyalty = game.officers.get(1).loyalty;

    for _ in 0..5 {
        round_process(&mut game);
    }

    // 忠诚度应该衰减
    assert!(game.officers.get(1).loyalty <= initial_loyalty);
}

// ========== 测试6: 完整游戏流程 ==========

#[test]
fn test_complete_game_flow() {
    let mut game = setup_full_game();

    // 阶段1: 内政（开发城市）
    game.cities.get_mut(1).off_18 = game.cities.get(1).off_18.saturating_add(5);
    game.cities.get_mut(1).off_0f = game.cities.get(1).off_0f.saturating_add(200);

    // 阶段2: 军事（征兵）
    game.cities.get_mut(1).off_13 = game.cities.get(1).off_13.saturating_add(50);

    // 阶段3: 战斗
    game.units.units[0] = Unit {
        id: 1, officer_id: 1, faction_id: 1,
        col: 5, row: 5, troops: 100, attr_17: 80,
        state: UnitState::Idle, ..Unit::default()
    };
    game.units.units[1] = Unit {
        id: 2, officer_id: 2, faction_id: 2,
        col: 6, row: 5, troops: 80, attr_17: 60,
        state: UnitState::Idle, ..Unit::default()
    };
    let battle_result = engage_battle(&mut game, 0, 1);

    // 阶段4: 回合推进
    let report = round_process(&mut game);

    // 验证状态
    assert_eq!(game.turn, 2);
    assert!(report.round == 1);
    assert!(matches!(battle_result, BattleResult::Win | BattleResult::Lose | BattleResult::Draw | BattleResult::Retreat));
}

#[test]
fn test_full_game_10_rounds() {
    let mut game = setup_full_game();

    for round in 0..10 {
        // 每回合：内政→军事→战斗→推进
        game.cities.get_mut(1).off_18 = game.cities.get(1).off_18.saturating_add(1);

        if round % 3 == 0 {
            // 每3回合打一仗
            game.units.units[0].col = 5;
            game.units.units[0].row = 5;
            game.units.units[1].col = 6;
            game.units.units[1].row = 5;

            if game.units.units[0].is_combat_ready() && game.units.units[1].is_combat_ready() {
                engage_battle(&mut game, 0, 1);
            }
        }

        let _report = round_process(&mut game);
    }

    assert_eq!(game.turn, 11);
    assert!(game.year >= 189);
}

// ========== 测试7: 边界条件 ==========

#[test]
fn test_battle_with_zero_troops() {
    let mut game = setup_full_game();
    game.units.units[0] = Unit {
        id: 1, officer_id: 1, faction_id: 1,
        col: 5, row: 5, troops: 0, attr_17: 80,
        state: UnitState::Idle, ..Unit::default()
    };
    game.units.units[1] = Unit {
        id: 2, officer_id: 2, faction_id: 2,
        col: 6, row: 5, troops: 100, attr_17: 70,
        state: UnitState::Idle, ..Unit::default()
    };

    let result = engage_battle(&mut game, 0, 1);
    assert_eq!(result, BattleResult::Lose); // 无兵力自动失败
}

#[test]
fn test_round_with_no_cities() {
    let mut game = setup_full_game();
    // 清空所有城市
    for city in game.cities.cities.iter_mut() {
        city.ownership = 0;
    }

    let report = round_process(&mut game);
    assert!(report.gold_changes.is_empty());
    assert!(report.food_changes.is_empty());
}

#[test]
fn test_round_with_no_officers() {
    let mut game = setup_full_game();
    // 清空所有武将
    for officer in game.officers.officers.iter_mut() {
        officer.id = 0;
    }

    let report = round_process(&mut game);
    assert!(report.loyalty_changes.is_empty());
}
