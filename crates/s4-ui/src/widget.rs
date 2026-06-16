//! Widget system.
//!
//! 规格文档引用:
//!   /23-widget-and-dialog-system — 控件基类与子控件设置
//!   /22-entity-vtable-and-event-dispatch — 实体dispatch

use crate::entity::{Entity, EntityManager, Rect, VtableId, vtable_tags};

/// 控件类型。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WidgetType {
    Base,
    Button,
    Scrollbar,
    NumInput,
    List,
    Text,
    Panel,
}

/// 控件基类初始化。
/// 规格: /23-widget-and-dialog-system — base_init
pub fn widget_base_init(entity: &mut Entity) {
    entity.vtable_id = vtable_tags::TEXT;
    entity.rect = Rect::default();
    entity.flags = 0;
    entity.flag_byte = 0;
    entity.field_11 = 0;
    entity.children.clear();
    entity.parent = None;
}

/// 按钮构造函数。
/// 规格: /23-widget-and-dialog-system — button_ctor
pub fn widget_button_ctor(entity: &mut Entity, rect: Rect, callback: Option<u16>) {
    widget_base_init(entity);
    entity.vtable_id = vtable_tags::BUTTON;
    entity.rect = rect;
    entity.field_11 = callback.unwrap_or(0);
}

/// 子控件设置。
/// 规格: /23-widget-and-dialog-system — subctrl_setup
pub fn widget_subctrl_setup(
    entity: &mut Entity,
    callback: Option<u16>,
    type_id: u8,
    secondary: u16,
    geometry: u16,
) {
    entity.field_11 = callback.unwrap_or(0);
    // 偏移0x16 = geometry, 0x18 = callback, 0x1a = type_id, 0x1c = secondary
    // 简化存储到flags的高位
    entity.flags = (type_id as u16) << 8 | (geometry & 0xFF) as u16;
}

/// 按钮控件。
#[derive(Debug, Clone)]
pub struct Button {
    pub entity_id: usize,
    pub rect: Rect,
    pub callback: Option<u16>,
    pub font_data: Vec<u8>,
}

impl Button {
    pub fn new(entity_id: usize, rect: Rect) -> Self {
        Self {
            entity_id,
            rect,
            callback: None,
            font_data: Vec::new(),
        }
    }
}

/// 滚动条方向。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScrollbarDirection {
    Vertical,
    Horizontal,
}

/// 滚动条控件。
/// 规格: /23-widget-and-dialog-system — scrollbar
pub struct Scrollbar {
    pub entity_id: usize,
    pub direction: ScrollbarDirection,
    pub min: i32,
    pub max: i32,
    pub value: i32,
    pub page_size: i32,
    /// 子控件ID。
    pub thumb_id: Option<usize>,
    pub arrow_up_id: Option<usize>,
    pub arrow_down_id: Option<usize>,
}

impl Scrollbar {
    pub fn new(entity_id: usize, direction: ScrollbarDirection) -> Self {
        Self {
            entity_id,
            direction,
            min: 0,
            max: 100,
            value: 0,
            page_size: 10,
            thumb_id: None,
            arrow_up_id: None,
            arrow_down_id: None,
        }
    }

    /// 创建子控件（5个子控件）。
    /// 规格: /23-widget-and-dialog-system — init_children
    pub fn init_children(&mut self, mgr: &mut EntityManager) {
        let base_vtable = match self.direction {
            ScrollbarDirection::Vertical => 0x9D52,
            ScrollbarDirection::Horizontal => 0x9D52,
        };

        // 主箭头
        let arrow1 = mgr.create(base_vtable);
        self.arrow_up_id = Some(arrow1);

        // 次箭头
        let arrow2 = mgr.create(base_vtable);
        self.arrow_down_id = Some(arrow2);

        // 滑块
        let thumb = mgr.create(vtable_tags::RECT_INTERACTIVE);
        self.thumb_id = Some(thumb);

        // 页上区域
        let _page_up = mgr.create(vtable_tags::RECT_BASE);
        // 页下区域
        let _page_down = mgr.create(vtable_tags::RECT_BASE);
    }

    /// 设置滚动范围。
    pub fn set_range(&mut self, min: i32, max: i32) {
        self.min = min;
        self.max = max;
        self.value = self.value.max(min).min(max);
    }

    /// 设置当前值。
    pub fn set_value(&mut self, val: i32) {
        self.value = val.max(self.min).min(self.max);
    }

    /// 按页滚动。
    pub fn page_up(&mut self) {
        self.value = (self.value - self.page_size).max(self.min);
    }

    pub fn page_down(&mut self) {
        self.value = (self.value + self.page_size).min(self.max);
    }

    /// 点击测试（按优先级）。
    /// 规格: /23-widget-and-dialog-system — dispatch_hit
    pub fn hit_test(&self, px: i16, py: i16) -> Option<ScrollbarPart> {
        // 简化实现：检查点击位置
        if let Some(thumb_id) = self.thumb_id {
            // 检查滑块区域
            let _ = thumb_id;
            // 实际需要通过EntityManager获取矩形
        }
        None
    }
}

/// 滚动条部件。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScrollbarPart {
    Thumb,
    ArrowUp,
    ArrowDown,
    PageUp,
    PageDown,
}

/// 列表控件。
/// 规格: /23-widget-and-dialog-system — list
pub struct ListControl {
    pub entity_id: usize,
    pub entries: Vec<usize>,  // 子实体ID
    pub selected: Option<usize>,
}

impl ListControl {
    pub fn new(entity_id: usize) -> Self {
        Self {
            entity_id,
            entries: Vec::new(),
            selected: None,
        }
    }

    /// 创建固定大小的列表条目。
    /// 规格: /23-widget-and-dialog-system — create_entries
    pub fn create_entries(&mut self, mgr: &mut EntityManager, count: usize) {
        for _ in 0..count {
            let entry = mgr.create(vtable_tags::RECT_INTERACTIVE);
            self.entries.push(entry);
            mgr.add_child(self.entity_id, entry);
        }
    }

    /// 选择条目。
    pub fn select(&mut self, index: usize) {
        if index < self.entries.len() {
            self.selected = Some(index);
        }
    }
}

/// 面板渲染。
/// 规格: /23-widget-and-dialog-system — panel
pub struct Panel {
    pub rect: Rect,
    pub popup: bool,
}

impl Panel {
    pub fn new(rect: Rect, popup: bool) -> Self {
        Self { rect, popup }
    }

    /// 渲染弹出面板背景。
    pub fn render_popup(&self) -> Vec<u8> {
        // 生成面板像素数据
        vec![0; (self.rect.w as usize) * (self.rect.h as usize)]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_button_creation() {
        let mut entity = Entity::new(vtable_tags::TEXT);
        widget_button_ctor(&mut entity, Rect::new(10, 10, 100, 30), Some(42));
        assert_eq!(entity.vtable_id, vtable_tags::BUTTON);
        assert_eq!(entity.rect.w, 100);
    }

    #[test]
    fn test_scrollbar() {
        let mut sb = Scrollbar::new(0, ScrollbarDirection::Vertical);
        sb.set_range(0, 100);
        sb.set_value(50);
        sb.page_up();
        assert_eq!(sb.value, 40);
        sb.page_down();
        sb.page_down();
        assert_eq!(sb.value, 60);
    }

    #[test]
    fn test_list_control() {
        let mut mgr = EntityManager::new();
        let list_id = mgr.create(vtable_tags::ROOT);
        let mut list = ListControl::new(list_id);
        list.create_entries(&mut mgr, 5);
        assert_eq!(list.entries.len(), 5);
        list.select(2);
        assert_eq!(list.selected, Some(2));
    }
}
