//! Platform trait definitions.
//!
//! 规格文档引用: /overview, /3-dos-loader-and-boot-sequence
//!
//! These traits abstract the DOS interrupt services (INT 21h, INT 10h, INT 16h, etc.)
//! used by the original game, allowing the core logic to be platform-independent.

use std::io;

/// A file handle returned by the platform layer.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct FileHandle(pub u32);

/// Video mode for display initialization.
/// 规格: /18-vga-mode-and-display-setup — VGA Mode 12h (640×480, 16色) / Mode X (320×200, 256色)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VideoMode {
    Text80x25,
    Vga12h,      // 640×480, 16 colors
    ModeX,       // 320×200, 256 colors, 4-plane
}

/// Key event from input polling.
/// 规格: /9-main-game-loop — 输入轮询循环
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyEvent {
    Char(u8),
    Special(u16),  // 功能键、方向键等
    Esc,
    Enter,
    Backspace,
    F(u8),        // F1-F10
}

/// Mouse event.
/// 规格: /9-main-game-loop — 鼠标输入检测
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MouseEvent {
    pub x: i16,
    pub y: i16,
    pub buttons: u8,
    pub dx: i16,
    pub dy: i16,
}

/// Audio command types.
/// 规格: /3-dos-loader-and-boot-sequence — 音频初始化
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AudioCommand {
    PlayMidi(u8),    // MIDI轨道
    PlaySound(u8),   // 音效编号
    StopAll,
}

/// Platform abstraction trait.
///
/// 替代原版DOS中断调用（INT 21h文件I/O、INT 10h视频、INT 16h键盘等）。
/// 每个方法对应一个或多个原始DOS中断服务。
pub trait Platform {
    // === 文件I/O (对应INT 21h AH=3Dh/3Fh/3Eh) ===

    /// 打开文件，返回文件句柄。
    fn open_file(&mut self, path: &str) -> io::Result<FileHandle>;

    /// 从文件读取数据到缓冲区，返回实际读取的字节数。
    fn read_file(&mut self, handle: FileHandle, buf: &mut [u8]) -> io::Result<usize>;

    /// 关闭文件。
    fn close_file(&mut self, handle: FileHandle) -> io::Result<()>;

    /// 重置磁盘子系统（对应INT 21h AH=0Dh）。
    fn disk_reset(&mut self) -> io::Result<()>;

    // === 显示 (对应INT 10h) ===

    /// 设置视频模式。
    fn set_video_mode(&mut self, mode: VideoMode) -> Result<(), DisplayError>;

    /// 写入像素数据到指定区域。
    fn draw_pixels(&mut self, x: u16, y: u16, w: u16, h: u16, data: &[u8]) -> Result<(), DisplayError>;

    /// 呈现当前帧到屏幕。
    fn present(&mut self) -> Result<(), DisplayError>;

    /// 等待垂直同步信号。
    fn wait_vsync(&mut self);

    /// 设置调色板颜色（对应VGA DAC寄存器编程）。
    fn set_palette_color(&mut self, index: u8, r: u8, g: u8, b: u8) -> Result<(), DisplayError>;

    // === 输入 (对应INT 16h/INT 33h) ===

    /// 轮询键盘输入，返回按键事件（非阻塞）。
    fn poll_key(&mut self) -> Option<KeyEvent>;

    /// 轮询鼠标输入。
    fn poll_mouse(&mut self) -> Option<MouseEvent>;

    /// 等待任意按键（阻塞）。
    fn wait_any_key(&mut self) -> KeyEvent;

    /// 等待鼠标空闲（消除抖动）。
    fn wait_mouse_idle(&mut self);

    // === 音频 ===

    /// 发送音频命令。
    fn audio_command(&mut self, cmd: AudioCommand) -> Result<(), AudioError>;

    // === 时间 ===

    /// 获取当前毫秒数（用于计时）。
    fn get_ticks_ms(&self) -> u64;

    /// 延迟指定毫秒数。
    fn delay_ms(&mut self, ms: u32);

    // === 随机数 ===

    /// 生成伪随机u32。
    fn random_u32(&mut self) -> u32;

    /// 生成指定范围内的随机数 [0, bound)。
    fn random_range(&mut self, bound: u32) -> u32 {
        self.random_u32() % bound
    }
}

/// Display error type.
#[derive(Debug, Clone)]
pub enum DisplayError {
    InitFailed(String),
    DrawFailed(String),
    ModeNotSupported,
}

impl std::fmt::Display for DisplayError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DisplayError::InitFailed(s) => write!(f, "Display init failed: {}", s),
            DisplayError::DrawFailed(s) => write!(f, "Draw failed: {}", s),
            DisplayError::ModeNotSupported => write!(f, "Video mode not supported"),
        }
    }
}

impl std::error::Error for DisplayError {}

/// Audio error type.
#[derive(Debug, Clone)]
pub enum AudioError {
    InitFailed(String),
    PlaybackFailed(String),
}

impl std::fmt::Display for AudioError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AudioError::InitFailed(s) => write!(f, "Audio init failed: {}", s),
            AudioError::PlaybackFailed(s) => write!(f, "Audio playback failed: {}", s),
        }
    }
}

impl std::error::Error for AudioError {}

/// Simple PRNG state (LCG) for deterministic random number generation.
/// 替代原版DOS随机数生成。
pub struct SimpleRng {
    state: u32,
}

impl SimpleRng {
    pub fn new(seed: u32) -> Self {
        SimpleRng { state: seed }
    }

    pub fn next_u32(&mut self) -> u32 {
        // LCG参数 from Numerical Recipes
        self.state = self.state.wrapping_mul(1664525).wrapping_add(1013904223);
        self.state
    }

    pub fn range(&mut self, bound: u32) -> u32 {
        if bound == 0 { return 0; }
        self.next_u32() % bound
    }
}
