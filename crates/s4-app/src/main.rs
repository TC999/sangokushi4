//! Application entry point.
//!
//! 规格文档引用:
//!   /3-dos-loader-and-boot-sequence — 启动序列
//!   /9-main-game-loop — 主游戏循环

mod platform;
mod init;
mod input;
mod renderer;
mod game_loop;

use platform::ConsolePlatform;
use game_loop::GameLoop;

fn main() {
    println!("========================================");
    println!("    三国志IV — Rust重写版 v0.1.0");
    println!("  基于净室规格文档，功能等价重写");
    println!("========================================");
    println!();

    // 选择剧本
    println!("请选择剧本:");
    println!("  [1] 189年 群雄割据");
    println!("  [2] 200年 官渡之战");
    print!("请选择 (1-2): ");

    let scenario = read_number(1, 2);

    println!();
    println!("初始化游戏...");

    // 创建平台
    let mut platform = ConsolePlatform::new();

    // 创建游戏循环
    let mut game_loop = GameLoop::new();

    // 初始化游戏
    game_loop.init(scenario as u8);

    println!();
    println!("游戏初始化完成！");
    println!("操作说明:");
    println!("  方向键/WASD - 移动");
    println!("  Enter/空格  - 确认");
    println!("  Esc/Q       - 返回/退出");
    println!("  1-7         - 功能菜单");
    println!("  0/E         - 结束回合");
    println!("  N           - 下一回合（调试）");
    println!("  @           - 显示状态（调试）");
    println!("  X           - 退出游戏");
    println!();

    // 运行游戏
    game_loop.run(&mut platform);

    println!();
    println!("感谢游玩！");
    println!("游戏结束。共{}回合。", game_loop.game.turn);
}

/// 读取范围内的数字。
fn read_number(min: u32, max: u32) -> u32 {
    loop {
        let mut input = String::new();
        std::io::stdin().read_line(&mut input).unwrap();
        match input.trim().parse::<u32>() {
            Ok(n) if n >= min && n <= max => return n,
            _ => print!("请输入{}-{}之间的数字: ", min, max),
        }
    }
}
