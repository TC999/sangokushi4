//! Console-based platform implementation.
//!
//! 提供基本的文件I/O和随机数，用于无GUI模式测试。

use s4_platform::*;
use std::collections::VecDeque;
use std::fs;
use std::io;

/// 控制台平台实现。
pub struct ConsolePlatform {
    rng_state: u32,
    key_queue: VecDeque<KeyEvent>,
    start_time: std::time::Instant,
}

impl Default for ConsolePlatform {
    fn default() -> Self {
        Self {
            rng_state: 0x12345678,
            key_queue: VecDeque::new(),
            start_time: std::time::Instant::now(),
        }
    }
}

impl ConsolePlatform {
    pub fn new() -> Self {
        Self::default()
    }

    /// 推送按键到队列（用于测试和脚本输入）。
    pub fn push_key(&mut self, key: KeyEvent) {
        self.key_queue.push_back(key);
    }
}

impl Platform for ConsolePlatform {
    // === 文件I/O ===

    fn open_file(&mut self, path: &str) -> io::Result<FileHandle> {
        match fs::File::open(path) {
            Ok(_) => Ok(FileHandle(1)), // 简化：固定句柄
            Err(e) => Err(e),
        }
    }

    fn read_file(&mut self, _handle: FileHandle, buf: &mut [u8]) -> io::Result<usize> {
        // 简化实现
        Ok(0)
    }

    fn close_file(&mut self, _handle: FileHandle) -> io::Result<()> {
        Ok(())
    }

    fn disk_reset(&mut self) -> io::Result<()> {
        Ok(())
    }

    // === 显示（控制台输出） ===

    fn set_video_mode(&mut self, _mode: VideoMode) -> Result<(), DisplayError> {
        Ok(())
    }

    fn draw_pixels(&mut self, _x: u16, _y: u16, _w: u16, _h: u16, _data: &[u8]) -> Result<(), DisplayError> {
        Ok(())
    }

    fn present(&mut self) -> Result<(), DisplayError> {
        Ok(())
    }

    fn wait_vsync(&mut self) {
        // 控制台模式：简单延迟
        std::thread::sleep(std::time::Duration::from_millis(16));
    }

    fn set_palette_color(&mut self, _index: u8, _r: u8, _g: u8, _b: u8) -> Result<(), DisplayError> {
        Ok(())
    }

    // === 输入 ===

    fn poll_key(&mut self) -> Option<KeyEvent> {
        self.key_queue.pop_front()
    }

    fn poll_mouse(&mut self) -> Option<MouseEvent> {
        None
    }

    fn wait_any_key(&mut self) -> KeyEvent {
        if let Some(key) = self.key_queue.pop_front() {
            return key;
        }
        // 简化：返回Enter
        KeyEvent::Enter
    }

    fn wait_mouse_idle(&mut self) {
        // 无操作
    }

    // === 音频 ===

    fn audio_command(&mut self, _cmd: AudioCommand) -> Result<(), AudioError> {
        Ok(())
    }

    // === 时间 ===

    fn get_ticks_ms(&self) -> u64 {
        self.start_time.elapsed().as_millis() as u64
    }

    fn delay_ms(&mut self, ms: u32) {
        std::thread::sleep(std::time::Duration::from_millis(ms as u64));
    }

    // === 随机数 ===

    fn random_u32(&mut self) -> u32 {
        self.rng_state = self.rng_state.wrapping_mul(1664525).wrapping_add(1013904223);
        self.rng_state
    }
}
