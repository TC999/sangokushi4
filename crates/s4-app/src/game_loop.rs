//! Main game loop.
//!
//! 规格文档引用:
//!   /9-main-game-loop — 主游戏循环
//!   /11-round-processing-and-turn-dispatch — 回合处理

use s4_platform::Platform;
use s4_core::data::GameState;
use s4_core::scene::{SceneManager, SceneId};
use crate::input::{InputHandler, InputCommand, show_main_menu};
use crate::renderer;
use crate::init;

/// 游戏循环。
pub struct GameLoop {
    pub game: GameState,
    pub scene_manager: SceneManager,
    pub input: InputHandler,
    pub running: bool,
    pub current_faction: u8,
}

impl GameLoop {
    pub fn new() -> Self {
        Self {
            game: GameState::new(),
            scene_manager: SceneManager::new(),
            input: InputHandler::new(),
            running: true,
            current_faction: 1,
        }
    }

    /// 初始化游戏。
    pub fn init(&mut self, scenario_id: u8) {
        init::init_new_scenario(&mut self.game, scenario_id);
        self.current_faction = 1;
        self.scene_manager.push(SceneId::CityView);
    }

    /// 主循环。
    pub fn run(&mut self, platform: &mut dyn Platform) {
        while self.running {
            // 渲染当前状态
            self.render();

            // 处理输入
            let cmd = self.input.wait_input(platform, &self.game);

            // 处理命令
            match cmd {
                InputCommand::Quit => {
                    self.running = false;
                }
                InputCommand::DebugStatus => {
                    init::print_game_summary(&self.game);
                    init::print_factions(&self.game);
                    init::print_cities(&self.game);
                    init::print_officers(&self.game);
                }
                InputCommand::DebugNextTurn => {
                    self.process_turn(platform);
                }
                InputCommand::Domestic => {
                    self.handle_domestic(platform);
                }
                InputCommand::Military => {
                    self.handle_military(platform);
                }
                InputCommand::Diplomacy => {
                    self.handle_diplomacy(platform);
                }
                InputCommand::Personnel => {
                    self.handle_personnel(platform);
                }
                InputCommand::Info => {
                    self.handle_info(platform);
                }
                InputCommand::EndTurn => {
                    self.process_turn(platform);
                }
                InputCommand::Confirm => {
                    // 在城市视图中确认 = 显示城市详情
                    if self.scene_manager.current() == Some(SceneId::CityView) {
                        self.handle_city_selection(platform);
                    }
                }
                _ => {}
            }
        }
    }

    /// 渲染当前状态。
    fn render(&self) {
        match self.scene_manager.current() {
            Some(SceneId::CityView) | Some(SceneId::MainMenu) => {
                show_main_menu(&self.game);
                renderer::render_map_console(&self.game, &s4_render::viewport::Viewport::new());
            }
            Some(SceneId::BattleView) => {
                println!("=== 战斗视图 ===");
                renderer::render_map_console(&self.game, &s4_render::viewport::Viewport::new());
                renderer::render_units(&self.game);
            }
            _ => {}
        }
    }

    /// 处理回合。
    fn process_turn(&mut self, _platform: &mut dyn Platform) {
        println!("\n>>> 处理第{}回合...", self.game.turn);

        // 执行回合处理管道（包含时间推进）
        let report = s4_core::round::round_process(&mut self.game);

        // 打印回合报告
        s4_core::round::print_round_report(&report);

        println!(">>> 回合{}完成\n", report.round);
    }

    /// 处理内政命令。
    fn handle_domestic(&mut self, _platform: &mut dyn Platform) {
        crate::input::show_domestic_menu();
        // 简化：执行开发
        println!("执行开发...");
        let city_id = self.get_current_city();
        let city = self.game.cities.get(city_id);
        let development = city.off_18 as u32 + 5;
        self.game.cities.get_mut(city_id).off_18 = development.min(100) as u8;
        println!("城市{}开发值提升至{}", city_id, self.game.cities.get(city_id).off_18);
    }

    /// 处理军事命令。
    fn handle_military(&mut self, _platform: &mut dyn Platform) {
        crate::input::show_military_menu();
        // 简化：执行征兵
        println!("执行征兵...");
        let city_id = self.get_current_city();
        let city = self.game.cities.get(city_id);
        let recruits = (city.off_0f / 100).min(100) as u16;
        self.game.cities.get_mut(city_id).off_13 += recruits;
        self.game.cities.get_mut(city_id).off_0f -= recruits as u16 * 10;
        println!("城市{}征兵{}人", city_id, recruits);
    }

    /// 处理外交命令。
    fn handle_diplomacy(&mut self, _platform: &mut dyn Platform) {
        crate::input::show_diplomacy_menu();
        println!("外交功能待实现");
    }

    /// 处理人事命令。
    fn handle_personnel(&mut self, _platform: &mut dyn Platform) {
        crate::input::show_personnel_menu();
        println!("人事功能待实现");
    }

    /// 处理信息查询。
    fn handle_info(&self, _platform: &mut dyn Platform) {
        init::print_game_summary(&self.game);
    }

    /// 处理城市选择。
    fn handle_city_selection(&self, _platform: &mut dyn Platform) {
        let city_id = self.get_current_city();
        renderer::render_city_detail(&self.game, city_id);
    }

    /// 获取当前城市ID。
    fn get_current_city(&self) -> u8 {
        // 简化：返回势力的第一个城市
        let faction_id = self.current_faction;
        for city in &self.game.cities.cities {
            if city.is_valid() && city.ownership == faction_id {
                return city.id;
            }
        }
        1
    }
}

impl Default for GameLoop {
    fn default() -> Self {
        Self::new()
    }
}
