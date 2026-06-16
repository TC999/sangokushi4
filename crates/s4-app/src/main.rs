//! Application entry point and main game loop.
//!
//! 规格文档引用:
//!   /3-dos-loader-and-boot-sequence
//!   /9-main-game-loop

use s4_core::data::GameState;
use s4_core::scene::SceneManager;

/// 主游戏循环。
/// 规格: /9-main-game-loop — main_game_mainloop
fn main_game_loop() {
    let mut game = GameState::new();
    let mut scene_manager = SceneManager::new();

    println!("三国志4 — Rust重写版");
    println!("基于净室规格文档，功能等价重写");

    // 初始化
    game.turn = 1;
    game.year = 189;
    game.month = 1;
    game.day = 1;

    // 主循环
    loop {
        // 场景分派
        scene_manager.dispatch(&mut game);

        // 检查退出
        if game.exit_flag {
            break;
        }

        // 回合推进
        if scene_manager.current() == Some(s4_core::scene::SceneId::CityView) {
            s4_core::round::round_process(&mut game);
            game.turn += 1;
        }
    }

    println!("游戏结束。总回合数: {}", game.turn);
}

fn main() {
    main_game_loop();
}
