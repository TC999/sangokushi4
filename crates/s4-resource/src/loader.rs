//! Resource loading pipeline.
//!
//! 规格文档引用:
//!   /25-resource-loading-and-decompression

use std::collections::HashMap;

/// 资源模式（磁盘/内存）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResourceMode {
    Disk,
    Memory,
}

/// 资源句柄。
pub struct ResourceHandle {
    pub file_path: Option<String>,
    pub data: Option<Vec<u8>>,
    pub mode: ResourceMode,
    pub decomp_param: u16,
}

/// 资源管理器。
pub struct ResourceManager {
    resources: HashMap<u16, ResourceHandle>,
}

impl Default for ResourceManager {
    fn default() -> Self {
        Self {
            resources: HashMap::new(),
        }
    }
}

impl ResourceManager {
    pub fn new() -> Self {
        Self::default()
    }

    /// 加载资源。
    pub fn load(&mut self, id: u16, path: &str) -> bool {
        // 尝试从磁盘读取
        match std::fs::read(path) {
            Ok(data) => {
                self.resources.insert(id, ResourceHandle {
                    file_path: Some(path.to_string()),
                    data: Some(data),
                    mode: ResourceMode::Disk,
                    decomp_param: 0,
                });
                true
            }
            Err(_) => false,
        }
    }

    /// 获取资源数据。
    pub fn get(&self, id: u16) -> Option<&ResourceHandle> {
        self.resources.get(&id)
    }

    /// 释放资源。
    pub fn release(&mut self, id: u16) {
        self.resources.remove(&id);
    }
}
