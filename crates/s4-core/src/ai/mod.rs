//! AI decision system.
//!
//! 规格文档引用:
//!   /12-turn-execution-pipeline — AI决策管道
//!   /13-officer-scoring-and-selection — 武将评分与选择
//!   /14-move-evaluation-and-pathfinding — 移动评估与寻路

mod scoring;
mod movement;
mod decision;

pub use scoring::*;
pub use movement::*;
pub use decision::*;
