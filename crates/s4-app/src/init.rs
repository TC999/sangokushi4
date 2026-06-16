//! Game initialization.
//!
//! 规格文档引用:
//!   /3-dos-loader-and-boot-sequence — 启动序列
//!   /17-resource-accumulation-and-depletion — 初始资源

use s4_core::data::*;

/// 初始化新剧本。
///
/// 规格: /17-resource-accumulation-and-depletion
/// 从剧本数据设置初始武将、城市、势力状态。
pub fn init_new_scenario(game: &mut GameState, scenario_id: u8) {
    game.turn = 1;
    game.game_mode = 0;
    game.normal_state = true;
    game.alternate_mode = false;

    match scenario_id {
        1 => init_scenario_1(game),
        2 => init_scenario_2(game),
        _ => init_scenario_1(game),
    }

    // 初始化地图
    init_map(game);

    println!("剧本{}初始化完成", scenario_id);
    println!("  年份: {}年{}月", game.year, game.month);
    println!("  城市: {}个有效", count_valid_cities(game));
    println!("  武将: {}个有效", count_valid_officers(game));
}

/// 剧本1：189年，群雄割据。
fn init_scenario_1(game: &mut GameState) {
    game.year = 189;
    game.month = 1;
    game.day = 1;

    // 初始化势力
    init_factions(game);

    // 初始化城市
    init_cities_s1(game);

    // 初始化武将
    init_officers_s1(game);
}

/// 剧本2：200年，官渡之战。
fn init_scenario_2(game: &mut GameState) {
    game.year = 200;
    game.month = 1;
    game.day = 1;

    init_factions(game);
    init_cities_s2(game);
    init_officers_s2(game);
}

/// 初始化势力。
fn init_factions(game: &mut GameState) {
    let faction_data = [
        (1, "曹操", 4),  // 红色
        (2, "刘备", 2),  // 绿色
        (3, "孙策", 3),  // 青色
        (4, "袁绍", 5),  // 品红
        (5, "袁术", 6),  // 棕色
        (6, "马腾", 1),  // 蓝色
        (7, "刘表", 7),  // 亮灰
    ];

    for (i, (id, _name, color)) in faction_data.iter().enumerate() {
        let faction = &mut game.factions.factions[i];
        faction.id = *id;
        faction.color = *color;
        faction.gold = 1000;
        faction.food = 2000;
    }
}

/// 剧本1城市初始化。
fn init_cities_s1(game: &mut GameState) {
    let cities = [
        (1, 1, 1000, 1500, 500),  // 洛阳 - 曹操
        (2, 1, 800, 1200, 400),   // 许昌 - 曹操
        (3, 2, 600, 1000, 300),   // 徐州 - 刘备
        (4, 3, 700, 1100, 350),   // 建业 - 孙策
        (5, 3, 500, 800, 250),    // 吴 - 孙策
        (6, 4, 900, 1300, 450),   // 邺 - 袁绍
        (7, 5, 400, 600, 200),    // 寿春 - 袁术
        (8, 6, 500, 700, 250),    // 天水 - 马腾
        (9, 7, 600, 900, 300),    // 襄阳 - 刘表
        (10, 1, 700, 1000, 350),  // 陈留 - 曹操
    ];

    for &(id, owner, gold, food, troops) in &cities {
        let city = game.cities.get_mut(id);
        city.id = id;
        city.ownership = owner;
        city.off_0f = gold;
        city.off_11 = food;
        city.off_13 = troops;
        city.off_15 = 50; // 治理属性
        city.off_18 = 10; // 发展参数A
        city.off_19 = 10; // 发展参数B
        city.off_1d = 50; // 可消耗资源A
        city.off_1e = 50; // 可消耗资源B
    }
}

/// 剧本2城市初始化。
fn init_cities_s2(game: &mut GameState) {
    // 官渡之战时期的城市分布
    init_cities_s1(game); // 简化：复用剧本1
}

/// 剧本1武将初始化。
fn init_officers_s1(game: &mut GameState) {
    let officers = [
        // (id, faction, int, pol, chr, loyalty)
        (1, 1, 90u8, 95u8, 70u8, 200u8),   // 曹操
        (2, 1, 85, 80, 75, 180),   // 夏侯惇
        (3, 1, 80, 75, 70, 170),   // 夏侯渊
        (4, 1, 88, 82, 68, 190),   // 曹仁
        (5, 1, 75, 70, 65, 160),   // 曹洪
        (6, 2, 92, 95, 80, 200),   // 刘备
        (7, 2, 95, 98, 85, 220),   // 关羽
        (8, 2, 93, 96, 90, 220),   // 张飞
        (9, 2, 90, 88, 75, 190),   // 赵云
        (10, 3, 88, 90, 78, 200),  // 孙策
        (11, 3, 85, 82, 72, 180),  // 周瑜
        (12, 3, 80, 78, 68, 170),  // 黄盖
        (13, 4, 82, 85, 70, 180),  // 袁绍
        (14, 4, 78, 75, 65, 160),  // 颜良
        (15, 4, 76, 73, 63, 155),  // 文丑
        (16, 5, 75, 70, 60, 150),  // 袁术
        (17, 6, 80, 82, 68, 170),  // 马腾
        (18, 6, 85, 80, 72, 180),  // 马超
        (19, 7, 78, 80, 65, 165),  // 刘表
        (20, 1, 90, 88, 72, 185),  // 荀彧
    ];

    for &(id, faction, int, pol, chr, loyalty) in &officers {
        let officer = game.officers.get_mut(id);
        officer.id = id;
        officer.off4 = int;
        officer.off5 = pol;
        officer.off6 = chr;
        officer.loyalty = loyalty as u8;
        officer.off7 = (int as u16 + pol as u16 + chr as u16) / 3; // 综合属性

        // 根据势力设置分类
        officer.off3 = ((faction as u8) << 4) | (id % 10) as u8;

        // 设置部队
        if id <= 10 {
            let unit_idx = (id - 1) as usize;
            if unit_idx < game.units.units.len() {
                game.units.units[unit_idx].id = id;
                game.units.units[unit_idx].officer_id = id;
                game.units.units[unit_idx].faction_id = faction as u8;
                game.units.units[unit_idx].attr_1b = 3; // 移动力
                game.units.units[unit_idx].attr_17 = officer.off7;
                game.units.units[unit_idx].col = ((id - 1) % 10) as u8;
                game.units.units[unit_idx].row = ((id - 1) / 10) as u8;
            }
        }
    }
}

/// 剧本2武将初始化。
fn init_officers_s2(game: &mut GameState) {
    init_officers_s1(game); // 简化：复用剧本1
}

/// 初始化地图。
fn init_map(game: &mut GameState) {
    // 设置基本地形
    for row in 0..11u8 {
        for col in 0..20u8 {
            let tile_type = if col < 2 || col > 17 || row < 1 || row > 9 {
                0x0A // 边界屏障
            } else if (row + col) % 5 == 0 {
                0x03 // 森林
            } else if (row * 3 + col) % 7 == 0 {
                0x04 // 山地
            } else if (row + col * 2) % 6 == 0 {
                0x05 // 河流
            } else {
                0x02 // 平原
            };
            game.map.set_tile(col, row, tile_type as u16);
        }
    }

    // 在城市位置设置城市瓦片
    for city in &game.cities.cities {
        if city.is_valid() {
            let col = (city.id as u8 - 1) % 10 + 5;
            let row = (city.id as u8 - 1) / 10 + 3;
            if col < 20 && row < 11 {
                game.map.set_tile(col, row, 0x01); // 城市瓦片
            }
        }
    }

    // 在单位位置设置单位标记
    for unit in &game.units.units {
        if unit.is_valid() {
            let current = game.map.get_tile(unit.col, unit.row);
            game.map.set_tile(unit.col, unit.row, current | 0x0100);
        }
    }
}

/// 统计有效城市数。
fn count_valid_cities(game: &GameState) -> usize {
    game.cities.cities.iter().filter(|c| c.is_valid()).count()
}

/// 统计有效武将数。
fn count_valid_officers(game: &GameState) -> usize {
    game.officers.officers.iter().filter(|o| o.is_valid()).count()
}

/// 打印游戏状态摘要。
pub fn print_game_summary(game: &GameState) {
    println!("========== 游戏状态 ==========");
    println!("时间: {}年{}月{}日  回合: {}", game.year, game.month, game.day, game.turn);
    println!("模式: {}", match game.game_mode {
        0 => "战略",
        3 => "战斗A",
        4 => "战斗B",
        _ => "未知",
    });
    println!("势力数: {}", game.factions.factions.iter().filter(|f| f.is_valid()).count());
    println!("有效城市: {}", count_valid_cities(game));
    println!("有效武将: {}", count_valid_officers(game));
    println!("活跃部队: {}", game.units.active_count());
    println!("==============================");
}

/// 打印势力信息。
pub fn print_factions(game: &GameState) {
    println!("--- 势力列表 ---");
    for faction in &game.factions.factions {
        if faction.is_valid() {
            println!("  势力{}: 金={} 粮={} 城市={}",
                faction.id, faction.gold, faction.food, faction.cities.len());
        }
    }
}

/// 打印城市信息。
pub fn print_cities(game: &GameState) {
    println!("--- 城市列表 ---");
    for city in &game.cities.cities {
        if city.is_valid() {
            println!("  城市{}: 所有者={} 金={} 粮={} 兵={}",
                city.id, city.ownership, city.off_0f, city.off_11, city.off_13);
        }
    }
}

/// 打印武将信息。
pub fn print_officers(game: &GameState) {
    println!("--- 武将列表（前10个） ---");
    let mut count = 0;
    for officer in &game.officers.officers {
        if officer.is_valid() {
            println!("  武将{}: INT={} POL={} CHR={} 忠诚={}",
                officer.id, officer.off4, officer.off5, officer.off6, officer.loyalty);
            count += 1;
            if count >= 10 { break; }
        }
    }
}
