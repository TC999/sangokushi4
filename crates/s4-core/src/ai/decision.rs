//! AI decision dispatch.
//!
//! 规格文档引用:
//!   /12-turn-execution-pipeline — AI决策管道
//!   /11-round-processing-and-turn-dispatch — 回合处理

use crate::data::GameState;
use super::scoring::*;
use super::movement;

/// AI完整回合扫描 — 11×20网格。
/// 规格: /11-round-processing-and-turn-dispatch — execute_full_pass
pub fn execute_full_pass(game: &mut GameState) {
    for row in 0..11u8 {
        for col in 0..20u8 {
            // 第10行跳过奇数列
            if row == 10 && (col & 1) != 0 {
                continue;
            }
            execute_turn(game, col, row);
        }
    }
}

/// 单个AI回合执行。
/// 规格: /12-turn-execution-pipeline — execute_turn
pub fn execute_turn(game: &mut GameState, col: u8, row: u8) {
    // 1. 查找此位置的单位
    let unit_idx = match game.units.units.iter().position(|u| u.col == col && u.row == row) {
        Some(i) => i,
        None => return,
    };
    
    // 2. 验证瓦片和标志
    let tile_type = game.map.get_tile_type(col, row);
    if tile_type as u8 == 0 { return; }
    if !game.check_normal_state() { return; }
    
    let has_flags = game.units.units[unit_idx].has_combat_flags();
    if !has_flags { return; }
    
    // 3. 执行最多3次移动
    let mut current_col = game.units.units[unit_idx].col;
    let mut current_row = game.units.units[unit_idx].row;
    
    for _ in 0..3 {
        // 获取当前单位的快照
        let unit_snapshot = game.units.units[unit_idx].clone();
        if let Some((dc, dr)) = movement::pick_best_direction(&unit_snapshot, game) {
            let new_col = current_col as i16 + dc as i16;
            let new_row = current_row as i16 + dr as i16;
            if game.map.in_bounds(new_col as i32, new_row as i32) {
                game.units.units[unit_idx].col = new_col as u8;
                game.units.units[unit_idx].row = new_row as u8;
                current_col = new_col as u8;
                current_row = new_row as u8;
            }
        }
    }
    
    // 4. 如果flag6设置，额外移动一步
    let has_flag6 = game.units.units[unit_idx].has_flag6();
    if has_flag6 {
        let unit_snapshot = game.units.units[unit_idx].clone();
        if let Some((dc, dr)) = movement::pick_best_direction(&unit_snapshot, game) {
            let new_col = current_col as i16 + dc as i16;
            let new_row = current_row as i16 + dr as i16;
            if game.map.in_bounds(new_col as i32, new_row as i32) {
                game.units.units[unit_idx].col = new_col as u8;
                game.units.units[unit_idx].row = new_row as u8;
            }
        }
    }
    
    // 5. 更新地图瓦片值
    game.map.set_tile(col, row, 0);
    let final_col = game.units.units[unit_idx].col;
    let final_row = game.units.units[unit_idx].row;
    let officer_id = game.units.units[unit_idx].officer_id;
    game.map.set_tile(final_col, final_row, 0x0100 | officer_id);
}

/// 武将方向移动决策。
/// 规格: /12-turn-execution-pipeline — officer_direction_move
pub fn officer_direction_decision(game: &mut GameState, unit_idx: usize) {
    let unit = &game.units.units[unit_idx];
    
    // 选择最佳移动目标
    if let Some(target) = movement::find_best_cell_weighted(unit, game) {
        let unit_mut = &mut game.units.units[unit_idx];
        unit_mut.col = target.0;
        unit_mut.row = target.1;
    }
}

/// 开发决策检查。
/// 规格: /12-turn-execution-pipeline — develop_decision_check
pub fn develop_decision_check(game: &mut GameState, city_id: u8) -> bool {
    let city = game.cities.get(city_id);
    if city.off_0f < 2048 {
        return false;
    }
    // 调用develop决策
    let officers: Vec<_> = game.officers.officers.iter()
        .filter(|o| o.is_valid() && !o.is_disabled())
        .cloned()
        .collect();
    
    if let Some((idx, _score)) = officer_score_dispatch(&officers, |o| score_develop(o, game)) {
        true
    } else {
        false
    }
}

/// 招募决策。
pub fn recruit_decision(game: &mut GameState, city_id: u8) -> bool {
    let city = game.cities.get(city_id);
    let pop_threshold = if city.off_0f < 151 { 50u16 } else { 40u16 };
    
    let officers: Vec<_> = game.officers.officers.iter()
        .filter(|o| o.is_valid() && !o.is_disabled())
        .cloned()
        .collect();
    
    if let Some((_idx, score)) = officer_score_dispatch(&officers, |o| score_recruit(o, game)) {
        score >= pop_threshold
    } else {
        false
    }
}

/// 动作分派（终端执行）。
/// 规格: /12-turn-execution-pipeline — officer_action_dispatch
pub fn action_dispatch(game: &mut GameState, action_id: u8, city_id: u8) -> i32 {
    match action_id {
        3 => {
            // 战斗UI设置
            // 规格: combat UI layout
            0
        }
        4 => {
            // 招募
            if recruit_decision(game, city_id) { 0 } else { -1 }
        }
        7 => {
            // 城市开发
            let city = game.cities.get(city_id);
            let stat_sum = city.off_18 as u32 + city.off_19 as u32;
            let sqrt_val = integer_sqrt(stat_sum);
            if sqrt_val == 0 { return 0; }
            
            let base = (city.off_18 as u32 / 2) * (city.off_1c as u32 + 1) + city.off_18 as u32;
            let result = base / sqrt_val + 1;
            let capped = if result > 16 {
                game.random_range(10) as u32 + 14
            } else {
                result
            };
            
            game.cities.get_mut(city_id).off_1e = capped.min(255) as u8;
            0
        }
        _ => -1,
    }
}

/// 整数平方根。
fn integer_sqrt(val: u32) -> u32 {
    if val == 0 { return 0; }
    let mut x = val;
    let mut y = (x + 1) / 2;
    while y < x {
        x = y;
        y = (x + val / x) / 2;
    }
    x
}
