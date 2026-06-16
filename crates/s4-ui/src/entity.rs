//! Entity vtable system.
//!
//! 规格文档引用:
//!   /22-entity-vtable-and-event-dispatch — 实体vtable与事件分派

/// 实体矩形。
#[derive(Debug, Clone, Copy, Default)]
pub struct Rect {
    pub x: i16,
    pub y: i16,
    pub w: u16,
    pub h: u16,
}

impl Rect {
    pub fn new(x: i16, y: i16, w: u16, h: u16) -> Self {
        Self { x, y, w, h }
    }

    /// 点是否在矩形内。
    pub fn contains(&self, px: i16, py: i16) -> bool {
        px >= self.x && px < self.x + self.w as i16 &&
        py >= self.y && py < self.y + self.h as i16
    }

    /// 与另一个矩形是否有交集。
    pub fn intersects(&self, other: &Rect) -> bool {
        self.x < other.x + other.w as i16 &&
        self.x + self.w as i16 > other.x &&
        self.y < other.y + other.h as i16 &&
        self.y + self.h as i16 > other.y
    }

    /// 设置为有序矩形（确保w,h非负）。
    pub fn set_ordered(&mut self, x1: i16, y1: i16, x2: i16, y2: i16) {
        self.x = x1.min(x2);
        self.y = y1.min(y2);
        self.w = (x2 - x1).unsigned_abs();
        self.h = (y2 - y1).unsigned_abs();
    }
}

/// 实体标志位。
pub const ENTITY_FLAG_VISIBLE: u8 = 0x01;
pub const ENTITY_FLAG_DISPATCH: u8 = 0x02;
pub const ENTITY_FLAG_DTOR_ELIGIBLE: u8 = 0x08;
pub const ENTITY_FLAG_MODAL: u8 = 0x80;
pub const ENTITY_FLAG_POSITION_MODE: u8 = 0x40;

/// 实体扩展标志位（flag10）。
pub const FLAG10_HIT_TEST: u16 = 0x0002;

/// vtable类型标识。
pub type VtableId = u16;

/// 实体结构。
///
/// 规格: /22-entity-vtable-and-event-dispatch
/// 每个实体开头是vtable指针（offset 0），用于多态调度。
#[derive(Debug, Clone)]
pub struct Entity {
    /// vtable类型标识（决定行为的类型标签）。
    pub vtable_id: VtableId,
    /// 包围矩形。
    pub rect: Rect,
    /// 标志字（低8位=flag_byte, 高8位=flag10）。
    pub flags: u16,
    /// 标志字节。
    pub flag_byte: u8,
    /// 扩展位置/数据。
    pub field_11: u16,
    /// 视口相对原点。
    pub viewport_origin: (i16, i16),
    /// 工作矩形。
    pub work_rect: Rect,
    /// 子实体ID列表。
    pub children: Vec<usize>,
    /// 父实体ID。
    pub parent: Option<usize>,
    /// 全局列表中的索引。
    pub list_index: Option<usize>,
    /// 引用计数。
    pub refcount: u16,
}

impl Default for Entity {
    fn default() -> Self {
        Self {
            vtable_id: 0,
            rect: Rect::default(),
            flags: 0,
            flag_byte: 0,
            field_11: 0,
            viewport_origin: (0, 0),
            work_rect: Rect::default(),
            children: Vec::new(),
            parent: None,
            list_index: None,
            refcount: 0,
        }
    }
}

impl Entity {
    /// 创建实体（清零字段、设置vtable标签）。
    pub fn new(vtable_id: VtableId) -> Self {
        Self {
            vtable_id,
            ..Self::default()
        }
    }

    /// 是否可见。
    pub fn is_visible(&self) -> bool {
        self.flag_byte & ENTITY_FLAG_VISIBLE != 0
    }

    /// 设置可见标志。
    pub fn set_visible(&mut self, visible: bool) {
        if visible {
            self.flag_byte |= ENTITY_FLAG_VISIBLE;
        } else {
            self.flag_byte &= !ENTITY_FLAG_VISIBLE;
        }
    }

    /// 是否是模态实体（阻塞事件传递）。
    pub fn is_modal(&self) -> bool {
        self.flag_byte & ENTITY_FLAG_MODAL != 0
    }

    /// 设置模态标志。
    pub fn set_modal(&mut self, modal: bool) {
        if modal {
            self.flag_byte |= ENTITY_FLAG_MODAL;
        } else {
            self.flag_byte &= !ENTITY_FLAG_MODAL;
        }
    }

    /// 是否应该调用析构函数。
    pub fn is_dtor_eligible(&self) -> bool {
        self.flag_byte & ENTITY_FLAG_DTOR_ELIGIBLE != 0
    }

    /// 测试flag10位。
    pub fn test_flag10(&self, mask: u16) -> bool {
        self.flags & mask != 0
    }

    /// 设置flag10位。
    pub fn set_flag10(&mut self, mask: u16, set: bool) {
        if set {
            self.flags |= mask;
        } else {
            self.flags &= !mask;
        }
    }

    /// 获取vtable方法偏移处的函数指针（简化版：返回vtable_id）。
    pub fn vtable_method(&self, offset: u8) -> u16 {
        // 在实际实现中，这里会从vtable表中读取函数指针
        // 简化版：返回vtable_id + offset作为标识
        self.vtable_id.wrapping_add(offset as u16)
    }
}

/// 碰撞测试（点是否在实体内）。
/// 规格: /22-entity-vtable-and-event-dispatch — hit_test_deep
pub fn entity_hit_test(entity: &Entity, px: i16, py: i16) -> bool {
    if !entity.is_visible() {
        return false;
    }
    entity.rect.contains(px, py)
}

/// 实体类型构造函数标签。
pub mod vtable_tags {
    /// 根实体。
    pub const ROOT: u16 = 0x9CE2;
    /// 基础矩形实体。
    pub const RECT_BASE: u16 = 0x9D26;
    /// 交互矩形实体。
    pub const RECT_INTERACTIVE: u16 = 0x9D6A;
    /// 视口实体。
    pub const VIEWPORT: u16 = 0x9D82;
    /// 窗口控件。
    pub const WINDOW: u16 = 0xA426;
    /// 按钮控件。
    pub const BUTTON: u16 = 0x9F0A;
    /// 滚动条。
    pub const SCROLLBAR: u16 = 0xA31E;
    /// 文本控件。
    pub const TEXT: u16 = 0x9F0E;
}

/// 实体存储管理器。
pub struct EntityManager {
    entities: Vec<Entity>,
    root_id: Option<usize>,
    free_list: Vec<usize>,
}

impl Default for EntityManager {
    fn default() -> Self {
        Self {
            entities: Vec::new(),
            root_id: None,
            free_list: Vec::new(),
        }
    }
}

impl EntityManager {
    pub fn new() -> Self {
        Self::default()
    }

    /// 创建实体。
    pub fn create(&mut self, vtable_id: VtableId) -> usize {
        let id = if let Some(free_id) = self.free_list.pop() {
            self.entities[free_id] = Entity::new(vtable_id);
            free_id
        } else {
            let id = self.entities.len();
            self.entities.push(Entity::new(vtable_id));
            id
        };
        id
    }

    /// 获取实体引用。
    pub fn get(&self, id: usize) -> Option<&Entity> {
        self.entities.get(id)
    }

    /// 获取实体可变引用。
    pub fn get_mut(&mut self, id: usize) -> Option<&mut Entity> {
        self.entities.get_mut(id)
    }

    /// 添加子实体。
    pub fn add_child(&mut self, parent_id: usize, child_id: usize) {
        if let Some(parent) = self.entities.get_mut(parent_id) {
            parent.children.push(child_id);
        }
        if let Some(child) = self.entities.get_mut(child_id) {
            child.parent = Some(parent_id);
        }
    }

    /// 销毁实体及其子实体。
    pub fn destroy(&mut self, id: usize) {
        // 先递归销毁子实体
        let children: Vec<usize> = self.entities[id].children.clone();
        for child_id in children {
            self.destroy(child_id);
        }
        // 标记为空闲
        self.entities[id] = Entity::default();
        self.free_list.push(id);
    }

    /// 深度碰撞测试（递归到最深子实体）。
    pub fn hit_test_deep(&self, root_id: usize, px: i16, py: i16) -> Option<usize> {
        let entity = &self.entities[root_id];
        if !entity.is_visible() || !entity.test_flag10(FLAG10_HIT_TEST) {
            return None;
        }

        // 递归到最深子实体
        for &child_id in &entity.children {
            if let Some(hit_id) = self.hit_test_deep(child_id, px, py) {
                return Some(hit_id);
            }
        }

        // 测试自身
        if entity.rect.contains(px, py) {
            Some(root_id)
        } else {
            None
        }
    }

    /// 将实体移到最前（z-order管理）。
    pub fn bring_to_front(&mut self, id: usize) {
        if let Some(parent_id) = self.entities[id].parent {
            let parent = &mut self.entities[parent_id];
            if let Some(pos) = parent.children.iter().position(|&x| x == id) {
                let child = parent.children.remove(pos);
                parent.children.push(child);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rect_contains() {
        let r = Rect::new(10, 20, 100, 50);
        assert!(r.contains(50, 30));
        assert!(!r.contains(5, 30));
    }

    #[test]
    fn test_entity_creation() {
        let mut mgr = EntityManager::new();
        let id = mgr.create(vtable_tags::ROOT);
        assert_eq!(mgr.get(id).unwrap().vtable_id, vtable_tags::ROOT);
    }

    #[test]
    fn test_entity_parent_child() {
        let mut mgr = EntityManager::new();
        let parent = mgr.create(vtable_tags::ROOT);
        let child = mgr.create(vtable_tags::BUTTON);
        mgr.add_child(parent, child);
        assert_eq!(mgr.get(child).unwrap().parent, Some(parent));
        assert!(mgr.get(parent).unwrap().children.contains(&child));
    }

    #[test]
    fn test_hit_test() {
        let mut entity = Entity::new(vtable_tags::RECT_BASE);
        entity.rect = Rect::new(0, 0, 100, 50);
        entity.set_visible(true);
        assert!(entity_hit_test(&entity, 50, 25));
        assert!(!entity_hit_test(&entity, 150, 25));
    }
}
