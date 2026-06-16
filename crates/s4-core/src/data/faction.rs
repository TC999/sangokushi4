//! Faction data structures.
//!
//! 规格文档引用:
//!   /17-resource-accumulation-and-depletion — 势力资源

/// 最大势力数。
pub const FACTION_COUNT: usize = 7;

/// 势力记录。
#[derive(Debug, Clone)]
pub struct Faction {
    pub id: u8,
    pub color: u8,
    pub cities: Vec<u8>,
    pub gold: u16,
    pub food: u16,
}

impl Default for Faction {
    fn default() -> Self {
        Self {
            id: 0,
            color: 0,
            cities: Vec::new(),
            gold: 0,
            food: 0,
        }
    }
}

impl Faction {
    pub fn is_valid(&self) -> bool {
        self.id != 0
    }

    pub fn city_count(&self) -> usize {
        self.cities.len()
    }
}

/// 全局无效势力静态实例。
static INVALID_FACTION: std::sync::OnceLock<Faction> = std::sync::OnceLock::new();

pub struct FactionStore {
    pub factions: Vec<Faction>,
}

impl FactionStore {
    pub fn new() -> Self {
        Self {
            factions: vec![Faction::default(); FACTION_COUNT],
        }
    }

    pub fn get(&self, id: u8) -> &Faction {
        if (id as usize) < FACTION_COUNT {
            &self.factions[id as usize]
        } else {
            INVALID_FACTION.get_or_init(|| Faction::default())
        }
    }
}
