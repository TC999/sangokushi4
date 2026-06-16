//! Game data models.
//!
//! 规格文档引用:
//!   - /15-bitfield-based-stat-accessors  — 武将位域访问器
//!   - /16-officer-slot-management       — 武将槽位管理
//!   - /17-resource-accumulation-and-depletion — 资源积累与消耗
//!   - /20-viewport-and-map-rendering    — 地图数据结构

pub mod officer;
pub mod city;
pub mod map;
pub mod faction;
pub mod unit;
mod game_state;

pub use officer::*;
pub use city::*;
pub use map::*;
pub use faction::*;
pub use unit::*;
pub use game_state::*;
