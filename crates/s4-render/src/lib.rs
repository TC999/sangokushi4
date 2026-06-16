//! Rendering pipeline.
//!
//! 规格文档引用:
//!   /18-vga-mode-and-display-setup — VGA模式与显示设置
//!   /19-tile-and-sprite-blitting-engine — 瓦片与精灵绘制
//!   /20-viewport-and-map-rendering — 视口与地图渲染
//!   /21-fade-and-scroll-animation — 淡入淡出与滚动动画

pub mod viewport;
pub mod tile;
pub mod fade;
pub mod font;
pub mod palette;

pub use viewport::*;
pub use tile::*;
pub use fade::*;
pub use font::*;
pub use palette::*;
