//! Console-based renderer.
//!
//! 规格文档引用:
//!   /20-viewport-and-map-rendering — 地图渲染
//!   /19-tile-and-sprite-blitting-engine — 瓦片绘制

use s4_core::data::{GameState, MAP_COLS, MAP_ROWS};
use s4_core::data::map::TileType;
use s4_render::viewport::Viewport;

/// 瓦片字符表示。
fn tile_char(tile_type: TileType, has_unit: bool) -> &'static str {
    if has_unit {
        return "◆";
    }
    match tile_type {
        TileType::Empty => "·",
        TileType::City => "城",
        TileType::Plain => "·",
        TileType::Forest => "林",
        TileType::Mountain => "山",
        TileType::River => "～",
        TileType::Bridge => "≡",
        TileType::Coast => "～",
        TileType::Sea => "～",
        TileType::Fortress => "堡",
        TileType::Barrier => "■",
        TileType::Special1 => "？",
        TileType::Barrier2 => "■",
        TileType::Barrier3 => "■",
        TileType::Developed => "田",
        TileType::Road => "─",
    }
}

/// 渲染地图到控制台。
///
/// 规格: /20-viewport-and-map-rendering — 地图渲染管道
pub fn render_map_console(game: &GameState, viewport: &Viewport) {
    let start_col = (viewport.scroll_x / 32).max(0) as u8;
    let start_row = (viewport.scroll_y / 32).max(0) as u8;
    let visible_cols = (viewport.width / 32).min(MAP_COLS as u16) as u8;
    let visible_rows = (viewport.height / 32).min(MAP_ROWS as u16) as u8;

    // 渲染地图
    for row in start_row..(start_row + visible_rows).min(MAP_ROWS as u8) {
        let mut line = String::new();

        // 奇数行缩进
        if (row & 1) != 0 {
            line.push_str("  ");
        }

        for col in start_col..(start_col + visible_cols).min(MAP_COLS as u8) {
            let tile_type = game.map.get_tile_type(col, row);
            let has_unit = game.map.get_tile(col, row) & 0xFF00 != 0;
            line.push_str(tile_char(tile_type, has_unit));
            line.push(' ');
        }
        println!("{}", line);
    }
}

/// 渲染单位信息。
pub fn render_units(game: &GameState) {
    println!("--- 活跃部队 ---");
    for unit in &game.units.units {
        if unit.is_valid() {
            let officer = game.officers.get(unit.officer_id);
            println!("  部队{}: 武将{} ({},{}) 移动力={} 属性={}",
                unit.id, unit.officer_id,
                unit.col, unit.row,
                unit.attr_1b, unit.attr_17);
        }
    }
}

/// 渲染城市详情。
pub fn render_city_detail(game: &GameState, city_id: u8) {
    let city = game.cities.get(city_id);
    if !city.is_valid() {
        println!("城市{}不存在", city_id);
        return;
    }

    println!("========== 城市{} ==========", city_id);
    println!("  所有者: 势力{}", city.ownership);
    println!("  金: {}", city.off_0f);
    println!("  粮: {}", city.off_11);
    println!("  兵: {}", city.off_13);
    println!("  治理: {}", city.off_15);
    println!("  发展A: {}  发展B: {}", city.off_18, city.off_19);
    println!("  条件: {}  乘数: {}", city.off_1a, city.off_1c);
    println!("  消耗A: {}  消耗B: {}", city.off_1d, city.off_1e);
    println!("============================");
}

/// 渲染武将详情。
pub fn render_officer_detail(game: &GameState, officer_id: u16) {
    let officer = game.officers.get(officer_id);
    if !officer.is_valid() {
        println!("武将{}不存在", officer_id);
        return;
    }

    println!("========== 武将{} ==========", officer_id);
    println!("  INT: {}  POL: {}  CHR: {}", officer.off4, officer.off5, officer.off6);
    println!("  忠诚: {}", officer.loyalty);
    println!("  分类: 0x{:02X}", officer.off3);
    println!("  标志A: 0x{:04X}", officer.off7);
    println!("  标志B: 0x{:04X}", officer.off9);
    println!("  属性E: 0x{:04X}", officer.flag_e);
    println!("  属性10: 0x{:04X}", officer.flag_10);
    println!("  属性12: 0x{:04X}", officer.flag_12);
    if officer.is_disabled() {
        println!("  ** 已死亡/无能力 **");
    }
    println!("============================");
}

/// 渲染回合报告。
pub fn render_round_report(game: &GameState, round: u32) {
    println!("========================================");
    println!("         第{}回合报告", round);
    println!("========================================");

    // 各势力资源
    for faction in &game.factions.factions {
        if faction.is_valid() {
            println!("  势力{}: 金={} 粮={}", faction.id, faction.gold, faction.food);
        }
    }

    println!("----------------------------------------");

    // 城市统计
    let total_gold: u32 = game.cities.cities.iter()
        .filter(|c| c.is_valid())
        .map(|c| c.off_0f as u32)
        .sum();
    let total_food: u32 = game.cities.cities.iter()
        .filter(|c| c.is_valid())
        .map(|c| c.off_11 as u32)
        .sum();
    let total_troops: u32 = game.cities.cities.iter()
        .filter(|c| c.is_valid())
        .map(|c| c.off_13 as u32)
        .sum();

    println!("  总金: {}  总粮: {}  总兵: {}", total_gold, total_food, total_troops);
    println!("========================================");
}
