//! AI scoring functions.
//!
//! 规格文档引用:
//!   /13-officer-scoring-and-selection — 武将评分函数

use crate::data::{Officer, GameState};

/// 评分上下文 — 对应原版0x937E, 0x9380, 0x9382。
pub struct ScoreContext {
    pub action_param: u16,
    pub threshold: u16,
    pub position_flag: u8,
}

/// 武将评分分派 — 策略模式。
/// 规格: /13-officer-scoring-and-selection — officer_score_dispatch
pub fn officer_score_dispatch<F>(
    officers: &[Officer],
    scorer: F,
) -> Option<(usize, u16)>
where
    F: Fn(&Officer) -> u16,
{
    let mut best_id: i32 = -1;
    let mut best_score: u16 = 0;
    for (i, officer) in officers.iter().enumerate() {
        if !officer.is_valid() || officer.is_disabled() {
            continue;
        }
        let score = scorer(officer);
        if score > best_score {
            best_score = score;
            best_id = i as i32;
        }
    }
    if best_id >= 0 {
        Some((best_id as usize, best_score))
    } else {
        None
    }
}

/// 军事评分。
/// 规格: /13-officer-scoring-and-selection — officer_score_military
/// 公式: div_or_default_50(& 0xFF) × 1.5 if bitratio_d
/// 最低分0x32(50)以下返回0。
pub fn score_military(officer: &Officer, _game: &GameState) -> u16 {
    let base = officer.off5 as u16 + officer.off4 as u16;
    let result = if base == 0 { 50 } else { base / 2 };
    if result < 50 { 0 } else { result }
}

/// 发展评分。
/// 规格: /13-officer-scoring-and-selection — officer_score_develop
/// 公式: city_off13 / (scenario_val / 3 + 1)，上限30。
pub fn score_develop(officer: &Officer, game: &GameState) -> u16 {
    let city_idx = officer.off3 % 21;
    let city = game.cities.get(city_idx);
    let base_val = 100u32;
    let scenario_base = base_val / 3 + 1;
    let result = city.off_13 as u32 / scenario_base;
    result.min(30) as u16
}

/// 攻击评分（含财富门槛）。
/// 规格: /13-officer-scoring-and-selection — officer_score_attack
/// 3999/4500/6500金门槛分级。
pub fn score_attack(officer: &Officer, game: &mut GameState) -> u16 {
    let stat_sum = officer.off4 as u16 + officer.off5 as u16 + officer.off6 as u16;
    let random_mod = game.random_range(100) as u16;
    let avg = stat_sum / 3;
    let max_stat = officer.off4.max(officer.off5).max(officer.off6) as u16;
    let raw = (max_stat + avg + random_mod) * 10;
    if raw > 3999 { raw } else { 0 }
}

/// 防御评分。
/// 规格: /13-officer-scoring-and-selection — officer_score_defense
/// (attr17 >> 3) + 100 或 (attr17 >> 6) + 100。
pub fn score_defense(officer: &Officer, _game: &GameState) -> u16 {
    let attr17 = officer.off7;
    if attr17 > 1 {
        (attr17 >> 6) + 100
    } else {
        (attr17 >> 3) + 100
    }
}

/// 招募评分。
/// 规格: officer_score_recruit — 直接返回attr17。
pub fn score_recruit(officer: &Officer, _game: &GameState) -> u16 {
    officer.off7
}

/// 建设评分。
/// 规格: officer_score_build — attr17 + 10。
pub fn score_build(officer: &Officer, _game: &GameState) -> u16 {
    officer.off7 + 10
}

/// 特殊评分（含反雪球机制）。
/// 规格: officer_score_special
/// 公式: -(attr_1a + stat_max) / 20 + 20(或30) + attr17/1024。
pub fn score_special(officer: &Officer, game: &GameState) -> u16 {
    let max_stat = officer.off4.max(officer.off5).max(officer.off6) as i32;
    let penalty = (officer.off_1a() as i32 + max_stat) / 20;
    let base_bonus = 20i32;
    let attr_bonus = officer.off7 as i32 / 1024;
    let result = base_bonus - penalty + attr_bonus;
    result.max(0) as u16
}

/// 复合评分。
/// 规格: officer_score_composite — 位域重叠计数 + 属性总和。
pub fn score_composite(officer: &Officer, _game: &GameState) -> u16 {
    let stat_sum = officer.off4 as u16 + officer.off5 as u16 + officer.off6 as u16;
    let loyalty_bonus = if officer.loyalty < 50 { 10 } else { 0 };
    stat_sum + loyalty_bonus
}

/// 策略选择最优（epsilon-greedy）。
/// 规格: /13-officer-scoring-and-selection — strategy_select_best
pub fn strategy_select_best<F>(
    candidates: &[(usize, u16)],  // (索引, 分数)
    evaluator: F,
    game: &mut GameState,
) -> Option<usize>
where
    F: Fn(usize) -> u16,
{
    if candidates.is_empty() {
        return None;
    }

    // 收集最高分候选
    let max_score = candidates.iter().map(|(_, s)| *s).max().unwrap_or(0);
    let top: Vec<usize> = candidates
        .iter()
        .filter(|(_, s)| *s == max_score)
        .map(|(i, _)| *i)
        .collect();

    if top.is_empty() {
        return None;
    }

    // 从顶级候选中随机选择
    let idx = game.random_range(top.len() as u32) as usize;
    Some(top[idx])
}
