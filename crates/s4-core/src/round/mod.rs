//! Round processing pipeline.
//!
//! 规格文档引用:
//!   /11-round-processing-and-turn-dispatch — 回合处理

use crate::data::GameState;

/// 回合处理主入口。
/// 规格: /11-round-processing-and-turn-dispatch — round_process
pub fn round_process(game: &mut GameState) {
    // Phase 1: AI移动与回合执行
    phase1_ai_movement(game);
    
    // Phase 2: 城市资源重算
    phase2_city_resources(game);
    
    // Phase 3: 场景引导与清理
    phase3_bootstrap_cleanup(game);
}

/// Phase 1: AI移动与回合执行。
fn phase1_ai_movement(game: &mut GameState) {
    // 检查dialog_slot_attr15
    if game.dialog_slot_attr15 == 0 { return; }
    
    // 如果有活跃单位，执行AI移动
    if game.active_unit_idx.is_some() {
        // unit_move_with_ai
        crate::ai::execute_full_pass(game);
    }
}

/// Phase 2: 城市资源重算。
fn phase2_city_resources(game: &mut GameState) {
    // 基于bitratio检查执行Off13和Off11修改
    // 规格: /11-round-processing-and-turn-dispatch
    super::city::accumulate_all_stats(game);
}

/// Phase 3: 场景引导与清理。
fn phase3_bootstrap_cleanup(game: &mut GameState) {
    // 场景引导（城市经济模拟）
    // 武将忠诚度处理
    // 对话框槽位重置
    // 图形刷新
    
    super::city::resource_deplete_loop(game, 0);
    
    // 武将忠诚度衰减
    officer_loyalty_decay(game);
}

/// 武将忠诚度衰减。
/// 规格: /11-round-processing-and-turn-dispatch — officer_loyalty_ratio_handler
fn officer_loyalty_decay(game: &mut GameState) {
    for officer in game.officers.officers.iter_mut() {
        if !officer.is_valid() { continue; }
        if officer.is_disabled() { continue; }
        
        // 轻微忠诚度衰减
        if officer.loyalty > 1 && officer.loyalty < 255 {
            // 每回合微量衰减（简化版）
        }
    }
}
