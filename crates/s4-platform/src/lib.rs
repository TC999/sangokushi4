//! Platform abstraction layer for Sangokushi IV rewrite.
//!
//! Provides traits that encapsulate DOS-era system calls (INT 21h, INT 10h, etc.)
//! allowing the core game logic to remain platform-independent.
//!
//! 规格文档引用: 架构设计文档 §三.7

mod traits;

pub use traits::*;
