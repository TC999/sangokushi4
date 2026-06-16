//! Round processing pipeline.
//!
//! 规格文档引用:
//!   /11-round-processing-and-turn-dispatch — 回合处理
//!   /17-resource-accumulation-and-depletion — 资源积累与消耗

use crate::data::GameState;

/// 回合处理结果报告。
#[derive(Debug, Clone)]
pub struct RoundReport {
    pub round: u32,
    pub year: u16,
    pub month: u8,
    pub day: u8,
    pub gold_changes: Vec<(u8, i32)>,
    pub food_changes: Vec<(u8, i32)>,
    pub troop_changes: Vec<(u8, i32)>,
    pub loyalty_changes: Vec<(u16, i8)>,
    pub events: Vec<String>,
}

impl Default for RoundReport {
    fn default() -> Self {
        Self {
            round: 0, year: 0, month: 0, day: 0,
            gold_changes: Vec::new(), food_changes: Vec::new(),
            troop_changes: Vec::new(), loyalty_changes: Vec::new(),
            events: Vec::new(),
        }
    }
}

/// 回合处理主入口。
/// 规格: /11-round-processing-and-turn-dispatch — round_process
pub fn round_process(game: &mut GameState) -> RoundReport {
    let mut report = RoundReport {
        round: game.turn as u32,
        year: game.year,
        month: game.month,
        day: game.day,
        ..RoundReport::default()
    };

    phase1_ai_movement(game);
    phase2_resource_accumulation(game, &mut report);
    phase3_resource_depletion(game, &mut report);
    phase4_loyalty_processing(game, &mut report);
    phase5_time_advance(game, &mut report);
    phase6_cleanup(game);

    report
}

/// Phase 1: AI移动与回合执行。
fn phase1_ai_movement(game: &mut GameState) {
    if game.active_unit_idx.is_some() {
        crate::ai::execute_full_pass(game);
    }
}

/// Phase 2: 城市资源积累。
fn phase2_resource_accumulation(game: &mut GameState, report: &mut RoundReport) {
    // 收集所有城市的数据，避免借用冲突
    let city_data: Vec<(u8, u16, u16, u16, u16, u8, u8)> = (0..crate::data::city::CITY_PLAYABLE as u8)
        .filter_map(|city_id| {
            let city = game.cities.get(city_id);
            if !city.is_valid() || city.ownership == 0 { return None; }
            Some((city_id, city.off_0f, city.off_11, city.off_13, city.off_15, city.off_18, city.off_19))
        })
        .collect();

    for (city_id, base_gold, base_food, _troops, governance, dev_a, dev_b) in city_data {
        let gold_growth = calc_growth(game, base_gold, governance);
        let food_growth = calc_growth(game, base_food, governance);
        let troop_growth = calc_troop_growth(game, base_food, base_gold, governance);

        // 更新城市数据
        let city = game.cities.get_mut(city_id);
        let old_gold = city.off_0f;
        let old_food = city.off_11;
        city.off_0f = old_gold.saturating_add(gold_growth);
        city.off_11 = old_food.saturating_add(food_growth);
        if troop_growth > 0 {
            city.off_13 = city.off_13.saturating_add(troop_growth);
            report.troop_changes.push((city_id, troop_growth as i32));
        }
        if dev_a < 100 { city.off_18 = dev_a + 1; }
        if dev_b < 100 { city.off_19 = dev_b + 1; }

        report.gold_changes.push((city_id, gold_growth as i32));
        report.food_changes.push((city_id, food_growth as i32));
    }
}

/// 计算资源增长量。
fn calc_growth(game: &mut GameState, base: u16, governance: u16) -> u16 {
    if base == 0 { return 0; }
    let gov = (governance as f64 / 100.0).max(0.1);
    let rand = 0.8 + game.random_range(40) as f64 / 100.0;
    ((base as f64 * gov * rand * 0.1) as u16).max(1)
}

/// 计算兵力增长。
fn calc_troop_growth(game: &mut GameState, food: u16, gold: u16, governance: u16) -> u16 {
    if food < 100 || gold < 50 { return 0; }
    let base = (governance / 10).max(1);
    base + game.random_range(5) as u16
}

/// Phase 3: 城市资源消耗。
fn phase3_resource_depletion(game: &mut GameState, report: &mut RoundReport) {
    let city_data: Vec<(u8, u16, u16, u16, u8)> = (0..crate::data::city::CITY_PLAYABLE as u8)
        .filter_map(|city_id| {
            let city = game.cities.get(city_id);
            if !city.is_valid() { return None; }
            Some((city_id, city.off_0f, city.off_11, city.off_13, city.off_1d))
        })
        .collect();

    for (city_id, gold, food, troops, condition) in city_data {
        let gold_upkeep = (troops / 400).max(1) as u16;
        let food_upkeep = (troops / 50).max(1) as u16;

        let city = game.cities.get_mut(city_id);
        city.off_0f = gold.saturating_sub(gold_upkeep);
        city.off_11 = food.saturating_sub(food_upkeep);

        // 消耗可消耗资源
        let deficit = 60u16.saturating_sub(condition as u16);
        city.stat_dec(0x1D, (deficit / 5) as u8);
        city.stat_dec(0x1E, (deficit / 4) as u8);

        // 更新报告
        if let Some(e) = report.gold_changes.iter_mut().find(|(id, _)| *id == city_id) {
            e.1 -= gold_upkeep as i32;
        }
        if let Some(e) = report.food_changes.iter_mut().find(|(id, _)| *id == city_id) {
            e.1 -= food_upkeep as i32;
        }
    }
}

/// Phase 4: 武将忠诚度处理。
fn phase4_loyalty_processing(game: &mut GameState, report: &mut RoundReport) {
    let current_faction = game.current_faction_id();

    for officer in game.officers.officers.iter_mut() {
        if !officer.is_valid() || officer.is_disabled() { continue; }
        if officer.loyalty == 0 || officer.loyalty == 255 { continue; }

        let old_loyalty = officer.loyalty;
        let is_own = officer.off3 >> 4 == current_faction;
        let decay = if is_own { 1 } else { 2 };
        officer.loyalty = officer.loyalty.saturating_sub(decay).max(1);

        let change = officer.loyalty as i8 - old_loyalty as i8;
        if change != 0 {
            report.loyalty_changes.push((officer.id, change));
        }
    }
}

/// Phase 5: 时间推进。
fn phase5_time_advance(game: &mut GameState, report: &mut RoundReport) {
    game.day += 1;
    if game.day > 30 {
        game.day = 1;
        game.month += 1;
        if game.month > 12 {
            game.month = 1;
            game.year += 1;
            report.events.push(format!("{}年到来！", game.year));
        }
        report.events.push(format!("{}月开始", game.month));
    }
    game.turn += 1;
}

/// Phase 6: 清理与重置。
fn phase6_cleanup(game: &mut GameState) {
    game.dialog_slot_attr15 = 0;
    game.dialog_slot_attr1e = 0;

    for unit in game.units.units.iter_mut() {
        if unit.is_valid() {
            unit.rest(5);
        }
    }
}

/// 打印回合报告。
pub fn print_round_report(report: &RoundReport) {
    println!("========================================");
    println!("  第{}回合  {}年{}月{}日", report.round, report.year, report.month, report.day);
    println!("========================================");

    if !report.gold_changes.is_empty() {
        println!("  金: {}", report.gold_changes.iter()
            .map(|(id, c)| format!("城{}({})", id, if *c >= 0 { format!("+{}", c) } else { c.to_string() }))
            .collect::<Vec<_>>().join(" "));
    }
    if !report.food_changes.is_empty() {
        println!("  粮: {}", report.food_changes.iter()
            .map(|(id, c)| format!("城{}({})", id, if *c >= 0 { format!("+{}", c) } else { c.to_string() }))
            .collect::<Vec<_>>().join(" "));
    }
    if !report.troop_changes.is_empty() {
        println!("  兵: {}", report.troop_changes.iter()
            .map(|(id, c)| format!("城{}(+{})", id, c))
            .collect::<Vec<_>>().join(" "));
    }
    if !report.loyalty_changes.is_empty() {
        println!("  忠诚: {}", report.loyalty_changes.iter()
            .map(|(id, c)| format!("将{}({})", id, if *c >= 0 { format!("+{}", c) } else { c.to_string() }))
            .collect::<Vec<_>>().join(" "));
    }
    for event in &report.events {
        println!("  ⚡ {}", event);
    }
    println!("========================================");
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::{Unit, UnitState};

    fn setup_game() -> GameState {
        let mut game = GameState::new();
        game.turn = 1;
        game.year = 189;
        game.month = 1;
        game.day = 1;
        game.rng_state = 0x12345678;

        game.cities.cities[0].id = 1;
        game.cities.cities[0].ownership = 1;
        game.cities.cities[0].off_0f = 1000;
        game.cities.cities[0].off_11 = 1500;
        game.cities.cities[0].off_13 = 500;
        game.cities.cities[0].off_15 = 50;

        game.officers.officers[1].id = 1;
        game.officers.officers[1].off3 = 0x10;
        game.officers.officers[1].loyalty = 200;

        game.units.units[0] = Unit {
            id: 1, officer_id: 1, fatigue: 50,
            state: UnitState::Idle, ..Unit::default()
        };

        game
    }

    #[test]
    fn test_round_process_basic() {
        let mut game = setup_game();
        let report = round_process(&mut game);
        assert_eq!(report.round, 1);
        assert_eq!(game.turn, 2);
    }

    #[test]
    fn test_time_advance() {
        let mut game = setup_game();
        game.day = 30;
        game.month = 12;
        let report = round_process(&mut game);
        assert_eq!(game.day, 1);
        assert_eq!(game.month, 1);
        assert_eq!(game.year, 190);
        assert!(report.events.iter().any(|e| e.contains("190")));
    }

    #[test]
    fn test_resource_changes() {
        let mut game = setup_game();
        let report = round_process(&mut game);
        assert!(!report.gold_changes.is_empty());
        assert!(!report.food_changes.is_empty());
    }

    #[test]
    fn test_loyalty_decay() {
        let mut game = setup_game();
        let report = round_process(&mut game);
        assert!(game.officers.officers[1].loyalty <= 200);
    }

    #[test]
    fn test_unit_fatigue_recovery() {
        let mut game = setup_game();
        round_process(&mut game);
        assert!(game.units.units[0].fatigue < 50);
    }

    #[test]
    fn test_report_print() {
        let mut game = setup_game();
        let report = round_process(&mut game);
        print_round_report(&report);
    }
}
