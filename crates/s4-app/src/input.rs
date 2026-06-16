//! Input handling.
//!
//! 规格文档引用:
//!   /9-main-game-loop — 输入轮询循环

use s4_platform::{KeyEvent, Platform};
use s4_core::data::GameState;

/// 输入命令。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputCommand {
    /// 无操作。
    None,
    /// 向上移动。
    MoveUp,
    /// 向下移动。
    MoveDown,
    /// 向左移动。
    MoveLeft,
    /// 向右移动。
    MoveRight,
    /// 确认。
    Confirm,
    /// 取消。
    Cancel,
    /// 内政。
    Domestic,
    /// 军事。
    Military,
    /// 外交。
    Diplomacy,
    /// 人事。
    Personnel,
    /// 存档。
    Save,
    /// 读档。
    Load,
    /// 结束回合。
    EndTurn,
    /// 显示信息。
    Info,
    /// 退出游戏。
    Quit,
    /// 调试：打印状态。
    DebugStatus,
    /// 调试：下一回合。
    DebugNextTurn,
}

/// 输入处理器。
pub struct InputHandler {
    /// 当前等待模式。
    pub waiting: bool,
    /// 命令缓冲。
    pub commands: Vec<InputCommand>,
}

impl Default for InputHandler {
    fn default() -> Self {
        Self {
            waiting: false,
            commands: Vec::new(),
        }
    }
}

impl InputHandler {
    pub fn new() -> Self {
        Self::default()
    }

    /// 轮询输入并转换为游戏命令。
    pub fn poll_input(&mut self, platform: &mut dyn Platform, game: &GameState) -> Option<InputCommand> {
        if let Some(key) = platform.poll_key() {
            return Some(self.translate_key(key, game));
        }
        None
    }

    /// 等待输入并返回命令。
    pub fn wait_input(&mut self, platform: &mut dyn Platform, game: &GameState) -> InputCommand {
        let key = platform.wait_any_key();
        self.translate_key(key, game)
    }

    /// 翻译按键为游戏命令。
    fn translate_key(&self, key: KeyEvent, game: &GameState) -> InputCommand {
        match key {
            KeyEvent::Char(b'w') | KeyEvent::Char(b'W') | KeyEvent::Special(0x48) => InputCommand::MoveUp,
            KeyEvent::Char(b's') | KeyEvent::Char(b'S') | KeyEvent::Special(0x50) => InputCommand::MoveDown,
            KeyEvent::Char(b'a') | KeyEvent::Char(b'A') | KeyEvent::Special(0x4B) => InputCommand::MoveLeft,
            KeyEvent::Char(b'd') | KeyEvent::Char(b'D') | KeyEvent::Special(0x4D) => InputCommand::MoveRight,
            KeyEvent::Enter | KeyEvent::Char(b' ') => InputCommand::Confirm,
            KeyEvent::Esc | KeyEvent::Char(b'q') | KeyEvent::Char(b'Q') => {
                if game.exit_flag {
                    InputCommand::Quit
                } else {
                    InputCommand::Cancel
                }
            }
            KeyEvent::Char(b'1') => InputCommand::Domestic,
            KeyEvent::Char(b'2') => InputCommand::Military,
            KeyEvent::Char(b'3') => InputCommand::Diplomacy,
            KeyEvent::Char(b'4') => InputCommand::Personnel,
            KeyEvent::Char(b'5') => InputCommand::Info,
            KeyEvent::Char(b'6') => InputCommand::Save,
            KeyEvent::Char(b'7') => InputCommand::Load,
            KeyEvent::Char(b'0') | KeyEvent::Char(b'e') | KeyEvent::Char(b'E') => InputCommand::EndTurn,
            KeyEvent::Char(b'x') | KeyEvent::Char(b'X') => InputCommand::Quit,
            KeyEvent::Char(b'@') => InputCommand::DebugStatus,
            KeyEvent::Char(b'n') | KeyEvent::Char(b'N') => InputCommand::DebugNextTurn,
            _ => InputCommand::None,
        }
    }
}

/// 显示游戏菜单。
pub fn show_main_menu(game: &GameState) {
    println!("========================================");
    println!("           三 国 志 IV");
    println!("         Rust重写版 v0.1.0");
    println!("========================================");
    println!("  {}年{}月{}日  回合{}", game.year, game.month, game.day, game.turn);
    println!("----------------------------------------");
    println!("  [1] 内政    [2] 军事    [3] 外交");
    println!("  [4] 人事    [5] 信息    [6] 存档");
    println!("  [7] 读档    [0] 结束回合");
    println!("  [n] 下一回合（调试）");
    println!("  [x] 退出游戏");
    println!("========================================");
    print!("请选择: ");
}

/// 显示内政菜单。
pub fn show_domestic_menu() {
    println!("--- 内政 ---");
    println!("  [1] 开发");
    println!("  [2] 商业");
    println!("  [3] 农业");
    println!("  [0] 返回");
}

/// 显示军事菜单。
pub fn show_military_menu() {
    println!("--- 军事 ---");
    println!("  [1] 征兵");
    println!("  [2] 编队");
    println!("  [3] 移动");
    println!("  [0] 返回");
}

/// 显示外交菜单。
pub fn show_diplomacy_menu() {
    println!("--- 外交 ---");
    println!("  [1] 使者");
    println!("  [2] 同盟");
    println!("  [3] 进贡");
    println!("  [0] 返回");
}

/// 显示人事菜单。
pub fn show_personnel_menu() {
    println!("--- 人事 ---");
    println!("  [1] 任命");
    println!("  [2] 登用");
    println!("  [3] 处罚");
    println!("  [0] 返回");
}
