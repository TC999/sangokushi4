//! UI framework.
//!
//! 规格文档引用:
//!   /22-entity-vtable-and-event-dispatch — 实体vtable系统
//!   /23-widget-and-dialog-system — 控件与对话框

mod entity;
mod widget;
mod dialog;

pub use entity::*;
pub use widget::*;
pub use dialog::*;
