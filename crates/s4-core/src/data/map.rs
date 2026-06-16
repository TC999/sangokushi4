//! Map data structures.
//!
//! 规格文档引用:
//!   /20-viewport-and-map-rendering — 地图数据结构、交错等距坐标

/// 地图列数。
pub const MAP_COLS: usize = 20;
/// 地图行数。
pub const MAP_ROWS: usize = 11;

/// 地图瓦片类型（低4位）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum TileType {
    Empty = 0x00,
    City = 0x01,
    Plain = 0x02,
    Forest = 0x03,
    Mountain = 0x04,
    River = 0x05,
    Bridge = 0x06,
    Coast = 0x07,
    Sea = 0x08,
    Fortress = 0x09,
    Barrier = 0x0A,
    Special1 = 0x0B,
    Barrier2 = 0x0C,
    Barrier3 = 0x0D,
    Developed = 0x0E,
    Road = 0x0F,
}

impl TileType {
    /// 从原始u8值解析。
    pub fn from_raw(val: u8) -> Self {
        match val & 0x0F {
            0x00 => TileType::Empty,
            0x01 => TileType::City,
            0x02 => TileType::Plain,
            0x03 => TileType::Forest,
            0x04 => TileType::Mountain,
            0x05 => TileType::River,
            0x06 => TileType::Bridge,
            0x07 => TileType::Coast,
            0x08 => TileType::Sea,
            0x09 => TileType::Fortress,
            0x0A => TileType::Barrier,
            0x0B => TileType::Special1,
            0x0C => TileType::Barrier2,
            0x0D => TileType::Barrier3,
            0x0E => TileType::Developed,
            0x0F => TileType::Road,
            _ => TileType::Empty,
        }
    }

    /// 是否是不可通过的障碍地形。
    /// 规格: /14-move-evaluation-and-pathfinding — pred_is_barrier
    pub fn is_barrier(&self) -> bool {
        matches!(self, TileType::Empty | TileType::Barrier | TileType::Barrier2 | TileType::Barrier3)
    }
}

/// 游戏地图 — 20列×11行。
///
/// 规格: /20-viewport-and-map-rendering
/// 每个条目16位：低4位=地形类型，高位=标志和元数据。
#[derive(Debug, Clone)]
pub struct GameMap {
    pub width: u16,
    pub height: u16,
    pub tiles: Vec<u16>,
    pub scroll_x: i16,
    pub scroll_y: i16,
    pub pixel_width: u16,
    pub pixel_height: u16,
}

impl Default for GameMap {
    fn default() -> Self {
        Self {
            width: MAP_COLS as u16,
            height: MAP_ROWS as u16,
            tiles: vec![0; MAP_COLS * MAP_ROWS],
            scroll_x: 0,
            scroll_y: 0,
            pixel_width: (MAP_COLS as u16) << 4,
            pixel_height: (MAP_ROWS as u16) << 4,
        }
    }
}

impl GameMap {
    /// 验证坐标是否在地图范围内。
    pub fn in_bounds(&self, col: i32, row: i32) -> bool {
        col >= 0 && col < self.width as i32 && row >= 0 && row < self.height as i32
    }

    /// 获取瓦片索引。
    fn tile_index(&self, col: u8, row: u8) -> usize {
        row as usize * MAP_COLS + col as usize
    }

    /// 获取瓦片原始值。
    /// 规格: /20-viewport-and-map-rendering — map_get_tile
    pub fn get_tile(&self, col: u8, row: u8) -> u16 {
        if col >= self.width as u8 || row >= self.height as u8 {
            return 0;
        }
        self.tiles[self.tile_index(col, row)]
    }

    /// 设置瓦片值。
    pub fn set_tile(&mut self, col: u8, row: u8, val: u16) {
        if col < self.width as u8 && row < self.height as u8 {
            let idx = self.tile_index(col, row);
            self.tiles[idx] = val;
        }
    }

    /// 获取瓦片类型（低4位）。
    /// 规格: map_get_tile_type
    pub fn get_tile_type(&self, col: u8, row: u8) -> TileType {
        TileType::from_raw(self.get_tile(col, row) as u8)
    }

    /// 检查瓦片是否匹配指定类型。
    pub fn check_tile_type(&self, col: u8, row: u8, target: TileType) -> bool {
        self.get_tile_type(col, row) == target
    }

    /// 获取瓦片位掩码（高位标志）。
    pub fn get_tile_bitmask(&self, col: u8, row: u8) -> u8 {
        (self.get_tile(col, row) >> 8) as u8
    }

    /// 查找第一个被占用的瓦片位置。
    /// 规格: map_find_first_occupied
    pub fn find_first_occupied(&self) -> Option<(u8, u8)> {
        for row in 0..self.height as u8 {
            for col in 0..self.width as u8 {
                if self.get_tile_bitmask(col, row) != 0 {
                    return Some((col, row));
                }
            }
        }
        None
    }
}

/// 交错等距坐标变换。
///
/// 规格: /20-viewport-and-map-rendering — grid_to_screen
/// 奇数行偏移半瓦片宽度。
pub fn grid_to_screen(col: u8, row: u8) -> (i32, i32) {
    let is_odd = (row & 1) != 0;
    let screen_x = if is_odd {
        (col as i32 + 2) * 32
    } else {
        col as i32 * 32 + 48
    };
    let screen_y = row as i32 * 32;
    (screen_x, screen_y)
}

/// 交错变换（网格坐标→双重坐标）。
pub fn stagger_transform(col: u8, row: u8) -> (u16, u16) {
    let stagger_x = if (row & 1) != 0 {
        col as u16 * 2 + 1
    } else {
        col as u16 * 2
    };
    let stagger_y = row as u16 * 2;
    (stagger_x, stagger_y)
}
