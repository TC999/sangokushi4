//! City management.
//!
//! 规格文档引用:
//!   /17-resource-accumulation-and-depletion
//!   /15-bitfield-based-stat-accessors

pub use crate::data::city::*;

use crate::data::{GameState, officer::Officer};

/// 资源积累管道。
/// 规格: /17-resource-accumulation-and-depletion — city_accumulate_all_stats
pub fn accumulate_all_stats(game: &mut GameState) {
    // Phase 1: 初始化缓冲区
    // Phase 2: 每城市统计聚合
    for city_id in 0..crate::data::city::CITY_PLAYABLE as u8 {
        let city = game.cities.get(city_id);
        if !city.is_valid() { continue; }
        
        // 累积资源（简化版）
        let _gold = city.off_0f;
        let _food = city.off_11;
        let _troops = city.off_13;
    }
    
    // Phase 3: 武将分配计数
    for officer in &game.officers.officers {
        if !officer.is_valid() || officer.is_disabled() { continue; }
        if Officer::bit_ratio_is_7(officer.off7, officer.off9) { continue; }
        if Officer::bit_ratio_is_8(officer.off7, officer.off9) { continue; }
        if officer.loyalty >= 21 { continue; }
        // 计入所在城市的有效武将数
    }
    
    // Phase 4: 结果回写
}

/// 资源消耗循环。
/// 规格: /17-resource-accumulation-and-depletion — resource_deplete_loop
pub fn resource_deplete_loop(game: &mut GameState, mode: u8) {
    let deficit_base = 60u16;
    
    for city_id in 0..super::data::city::CITY_PLAYABLE as u8 {
        let city = game.cities.get(city_id);
        if !city.is_valid() { continue; }
        
        let condition = city.off_1d as u16;
        let deficit = deficit_base.saturating_sub(condition);
        
        if mode == 0 {
            // 计算模式：基于赤字驱动消耗
            let rate_1d = (deficit / 5) as u8;
            let rate_1e = (deficit / 4) as u8;
            
            game.cities.get_mut(city_id).stat_dec(0x1D, rate_1d);
            game.cities.get_mut(city_id).stat_dec(0x1E, rate_1e);
        } else {
            // 读取模式：直接读取城市值
            let rate_1d = city.stat_read(0x1D);
            let rate_1e = city.stat_read(0x1E);
            game.cities.get_mut(city_id).stat_dec(0x1D, rate_1d);
            game.cities.get_mut(city_id).stat_dec(0x1E, rate_1e);
        }
    }
}
