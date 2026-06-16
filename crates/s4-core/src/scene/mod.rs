//! Scene engine.
//!
//! 规格文档引用:
//!   /10-scene-manager-and-state-transitions
//!   /9-main-game-loop

use crate::data::GameState;

/// 场景ID。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SceneId {
    MainMenu,
    CityView,
    BattleView,
    MapView,
    Dialog,
    Event,
}

/// 场景管理器。
pub struct SceneManager {
    current: Option<SceneId>,
    stack: Vec<SceneId>,
}

impl Default for SceneManager {
    fn default() -> Self {
        Self {
            current: Some(SceneId::MainMenu),
            stack: Vec::new(),
        }
    }
}

impl SceneManager {
    pub fn new() -> Self { Self::default() }
    
    /// 推送场景到栈。
    pub fn push(&mut self, scene: SceneId) {
        if let Some(current) = self.current {
            self.stack.push(current);
        }
        self.current = Some(scene);
    }
    
    /// 弹出场景。
    pub fn pop(&mut self) {
        self.current = self.stack.pop();
    }
    
    /// 获取当前场景。
    pub fn current(&self) -> Option<SceneId> {
        self.current
    }
    
    /// 主分派循环。
    /// 规格: /10-scene-manager-and-state-transitions — dispatch
    pub fn dispatch(&mut self, game: &mut GameState) {
        match self.current {
            Some(SceneId::CityView) => {
                // 城市场景处理
                super::round::round_process(game);
            }
            Some(SceneId::BattleView) => {
                // 战斗场景处理
            }
            Some(SceneId::MainMenu) => {
                // 主菜单
            }
            _ => {}
        }
    }
}
