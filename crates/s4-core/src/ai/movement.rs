//! AI movement evaluation.
//!
//! 规格文档引用:
//!   /14-move-evaluation-and-pathfinding — 移动评估与寻路

use crate::data::{GameMap, TileType, Unit, GameState};
use crate::data::map::{MAP_COLS, MAP_ROWS};

/// 地形成本表（对应原版0x87aa处的表）。
/// 规格: /14-move-evaluation-and-pathfinding — lookup_cost_table
const COST_TABLE: [u8; 16] = [
    0xFF, // 0x00 Empty - 不可通过
    1,    // 0x01 City
    1,    // 0x02 Plain
    2,    // 0x03 Forest
    3,    // 0x04 Mountain
    0xFF, // 0x05 River - 特殊
    1,    // 0x06 Bridge
    1,    // 0x07 Coast
    0xFF, // 0x08 Sea - 不可通过
    1,    // 0x09 Fortress
    0xFF, // 0x0A Barrier - 不可通过
    2,    // 0x0B Special1
    0xFF, // 0x0C Barrier2 - 不可通过
    0xFF, // 0x0D Barrier3 - 不可通过
    1,    // 0x0E Developed
    1,    // 0x0F Road
];

/// 获取地形成本。
/// 规格: /14-move-evaluation-and-pathfinding — tile_lookup_cost_table
pub fn lookup_cost_table(col: u8, row: u8, map: &GameMap) -> u8 {
    if col >= map.width as u8 || row >= map.height as u8 {
        return 0xFF;
    }
    let tile_type = map.get_tile_type(col, row) as u8;
    if (tile_type as usize) < COST_TABLE.len() {
        COST_TABLE[tile_type as usize]
    } else {
        0xFF
    }
}

/// 预测是否是屏障地形。
/// 规格: /14-move-evaluation-and-pathfinding — pred_is_barrier
pub fn pred_is_barrier(col: u8, row: u8, map: &GameMap) -> bool {
    let tt = map.get_tile_type(col, row);
    matches!(tt, TileType::Empty | TileType::Barrier | TileType::Barrier2 | TileType::Barrier3)
}

/// 距离启发式计算（交错网格优化）。
/// 规格: /14-move-evaluation-and-pathfinding — calc_distance_heuristic
/// 切比雪夫距离变体。
pub fn calc_distance_heuristic(
    from_col: u8, from_row: u8,
    to_col: u8, to_row: u8,
) -> u16 {
    let dx = (from_col as i16 - to_col as i16).unsigned_abs() as u16;
    let dy = (from_row as i16 - to_row as i16).unsigned_abs() as u16;
    
    let adjusted = if (from_row & 1) != 0 {
        dx / 2
    } else {
        (dx + 1) / 2
    };
    
    adjusted.max(dy) + adjusted
}

/// 方向偏移表（6个方向）。
/// 规格: /14-move-evaluation-and-pathfinding — 方向移动
const DIR_COLS: [i8; 6] = [0, 1, 1, 0, -1, -1];
const DIR_ROWS: [i8; 6] = [-1, -1, 0, 1, 1, 0];

/// 两阶段评估管道。
/// 规格: /14-move-evaluation-and-pathfinding — eval_two_phase
pub fn eval_two_phase(unit: &Unit, game: &GameState) -> i32 {
    if !unit.has_combat_flags() {
        return evaluate_move_cost(unit, game);
    }
    
    let phase1 = probe_and_eval(unit, game);
    if phase1 == -1 { return -1; }
    
    let phase2 = evaluate_move_cost(unit, game);
    if phase2 == -1 { return -1; }
    
    saturating_add_wrap(phase1, phase2)
}

/// 完整移动成本评估。
/// 规格: /14-move-evaluation-and-pathfinding — evaluate_move_cost
pub fn evaluate_move_cost(unit: &Unit, game: &GameState) -> i32 {
    let cost = lookup_cost_table(unit.col, unit.row, &game.map);
    if cost == 0xFF { return -1; }
    if cost > unit.attr_1b { return -1; }
    
    // 检查单位占用
    if game.map.get_tile_bitmask(unit.col, unit.row) != 0 {
        return -1;
    }
    
    cost as i32
}

/// 探测并评估。
fn probe_and_eval(unit: &Unit, game: &GameState) -> i32 {
    evaluate_move_cost(unit, game)
}

/// 饱和加法（避免16位溢出）。
fn saturating_add_wrap(a: i32, b: i32) -> i32 {
    a.wrapping_add(b)
}

/// 最近目标选择。
/// 规格: /14-move-evaluation-and-pathfinding — find_best_move_target
pub fn find_best_move_target(unit: &Unit, game: &GameState) -> Option<(u8, u8)> {
    let mut best_col = unit.col;
    let mut best_row = unit.row;
    let mut best_dist = u16::MAX;
    
    for row in 0..MAP_ROWS as u8 {
        for col in 0..MAP_COLS as u8 {
            if row == 10 && (col & 1) != 0 { continue; }
            
            let cost = lookup_cost_table(col, row, &game.map);
            if cost == 0xFF { continue; }
            if game.map.get_tile_bitmask(col, row) != 0 { continue; }
            
            let dist = calc_distance_heuristic(unit.col, unit.row, col, row);
            if dist < best_dist {
                best_dist = dist;
                best_col = col;
                best_row = row;
            }
        }
    }
    
    if best_dist < u16::MAX {
        Some((best_col, best_row))
    } else {
        None
    }
}

/// 加权单元格选择。
/// 规格: /14-move-evaluation-and-pathfinding — find_best_cell_weighted
pub fn find_best_cell_weighted(unit: &Unit, game: &GameState) -> Option<(u8, u8)> {
    let mut best_col = unit.col;
    let mut best_row = unit.row;
    let mut best_score: i32 = i32::MIN;
    
    for row in 0..MAP_ROWS as u8 {
        for col in 0..MAP_COLS as u8 {
            if row == 10 && (col & 1) != 0 { continue; }
            
            let cost = lookup_cost_table(col, row, &game.map);
            if cost == 0xFF { continue; }
            if game.map.get_tile_bitmask(col, row) != 0 { continue; }
            
            let dist = calc_distance_heuristic(unit.col, unit.row, col, row);
            let score = -(dist as i32); // 最小化距离
            
            if score > best_score {
                best_score = score;
                best_col = col;
                best_row = row;
            }
        }
    }
    
    if best_score > i32::MIN {
        Some((best_col, best_row))
    } else {
        None
    }
}

/// 贪心方向选择。
/// 规格: /14-move-evaluation-and-pathfinding — officer_pick_best_direction
pub fn pick_best_direction(unit: &Unit, game: &GameState) -> Option<(i8, i8)> {
    let mut best_dir: Option<(i8, i8)> = None;
    let mut best_cost: u8 = u8::MAX;
    
    let dir_order = shuffle_directions(game);
    
    for &dir_idx in &dir_order {
        let dc = DIR_COLS[dir_idx as usize];
        let dr = DIR_ROWS[dir_idx as usize];
        let new_col = unit.col as i16 + dc as i16;
        let new_row = unit.row as i16 + dr as i16;
        
        if new_col < 0 || new_col >= MAP_COLS as i16 { continue; }
        if new_row < 0 || new_row >= MAP_ROWS as i16 { continue; }
        
        let cost = lookup_cost_table(new_col as u8, new_row as u8, &game.map);
        if cost == 0xFF { continue; }
        if cost > unit.attr_1b { continue; }
        if game.map.get_tile_bitmask(new_col as u8, new_row as u8) != 0 { continue; }
        
        if cost < best_cost {
            best_cost = cost;
            best_dir = Some((dc, dr));
        }
    }
    
    best_dir
}

/// 随机化方向顺序。
fn shuffle_directions(game: &GameState) -> [u8; 6] {
    let mut dirs = [0u8, 1, 2, 3, 4, 5];
    // Fisher-Yates shuffle
    let mut state = game.rng_state;
    for i in (1..6).rev() {
        state = state.wrapping_mul(1664525).wrapping_add(1013904223);
        let j = (state as usize) % (i + 1);
        dirs.swap(i, j);
    }
    dirs
}
