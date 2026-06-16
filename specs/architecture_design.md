# 三国志4 Rust重写 — 架构设计文档

## 一、技术栈

- **语言**: Rust (edition 2021+)
- **图形**: wgpu + winit (跨平台) / 或 SDL2
- **音频**: rodio (音频播放) + MIDI支持
- **构建**: Cargo workspace
- **测试**: 内置 `#[test]`

## 二、顶层Cargo Workspace结构

```
sangokushi4-rs/
├── Cargo.toml                    # workspace root
├── crates/
│   ├── s4-core/                  # 核心逻辑层（平台无关）
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── data/             # 数据模型
│   │   │   ├── ai/               # AI决策系统
│   │   │   ├── battle/           # 战斗系统
│   │   │   ├── city/             # 城市管理
│   │   │   ├── diplomacy/        # 外交系统
│   │   │   ├── event/            # 事件调度
│   │   │   ├── map/              # 地图与移动
│   │   │   ├── round/            # 回合处理
│   │   │   ├── scene/            # 场景引擎
│   │   │   └── util/             # 数学/随机数
│   │   └── Cargo.toml
│   ├── s4-platform/              # 平台抽象层
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── traits.rs         # 抽象接口
│   │   │   ├── fs.rs             # 文件系统
│   │   │   ├── display.rs        # 显示输出
│   │   │   ├── input.rs          # 输入设备
│   │   │   ├── audio.rs          # 音频
│   │   │   ├── time.rs           # 时间/延迟
│   │   │   └── rng.rs            # 随机数
│   │   └── Cargo.toml
│   ├── s4-resource/              # 资源加载与解压缩
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── lzss.rs           # LZSS解压缩
│   │   │   ├── rle.rs            # RLE图像解码
│   │   │   ├── bitstream.rs      # 伽马编码比特流
│   │   │   ├── loader.rs         # 资源加载管道
│   │   │   └── ems.rs            # EMS内存模拟
│   │   └── Cargo.toml
│   ├── s4-render/                # 渲染层
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── viewport.rs       # 视口管理
│   │   │   ├── tile.rs           # 瓦片绘制
│   │   │   ├── sprite.rs         # 精灵绘制
│   │   │   ├── font.rs           # 字体渲染
│   │   │   ├── palette.rs        # 调色板管理
│   │   │   ├── scroll.rs         # 滚动动画
│   │   │   ├── fade.rs           # 淡入淡出
│   │   │   └── text.rs           # 文字渲染
│   │   └── Cargo.toml
│   ├── s4-ui/                    # UI框架
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── entity.rs         # 实体系统
│   │   │   ├── widget.rs         # 控件基类
│   │   │   ├── dialog.rs         # 对话框引擎
│   │   │   ├── scrollbar.rs      # 滚动条
│   │   │   ├── numinput.rs       # 数字输入
│   │   │   ├── list.rs           # 列表控件
│   │   │   └── menu.rs           # 菜单系统
│   │   └── Cargo.toml
│   └── s4-app/                   # 应用程序入口
│       ├── src/
│       │   ├── main.rs
│       │   ├── platform_wgpu.rs  # wgpu平台实现
│       │   ├── platform_sdl2.rs  # SDL2平台实现
│       │   └── game_loop.rs      # 主游戏循环
│       └── Cargo.toml
├── assets/                       # 游戏资源文件
│   ├── scenarios/                # 剧本数据
│   ├── maps/                     # 地图数据
│   ├── sprites/                  # 精灵图像
│   ├── fonts/                    # 字体文件
│   └── music/                    # 音乐文件
└── specs/                        # 规格文档缓存
    └── specs_combined.md
```

## 三、核心模块详细设计

### 3.1 数据模型 (s4-core/data/)

基于规格文档中定义的所有数据结构：

#### 武将 (Officer)
```rust
/// 武将记录 — 布局A（20字节原始布局）
/// 对应规格：/15-bitfield-based-stat-accessors, /16-officer-slot-management
pub struct Officer {
    pub id: u16,                    // 武将ID (1-400, 0=无效)
    pub off3: u8,                   // 分类字节（高4位+低4位=两个子分类）
    pub off4: u8,                   // INT属性
    pub off5: u8,                   // POL属性
    pub off6: u8,                   // CHR属性
    pub off7: u16,                  // 标志字A
    pub off9: u16,                  // 标志字B（bit15=主禁用标志）
    pub loyalty: u8,                // 忠诚度 (0xFF=无效)
    pub flag_e: u16,                // 偏移0xE标志字
    pub flag_10: u16,               // 偏移0x10标志字
    pub flag_12: u16,               // 偏移0x12标志字
}

/// 扩展记录 — 布局B（30字节）
pub struct OfficerExtB {
    pub field_14: u16,
    pub field_16: u16,
    pub field_18: u8,
    // ... 其他字段
}

/// 替代记录 — 布局C（32字节）
pub struct OfficerExtC {
    pub alt_off0: u8,
    pub war: u8,                    // 军事属性
    pub alt_off4: u8,
    pub nibble_byte: u8,            // 位3处的半字节打包分类
    pub flag_d: u16,                // 偏移0xD标志字
    // ... 其他字段
}
```

#### 城市 (City)
```rust
/// 城市记录 — 87字节(0x57)
/// 对应规格：/17-resource-accumulation-and-depletion
pub struct City {
    pub id: u8,
    pub ownership: u8,              // 所有权(0=无, 5=过渡)
    pub officer_ptr: u16,           // 偏移0x01, 武将列表指针
    pub off_0b: u16,                // 任命槽位1(太守)
    pub off_0d: u16,                // 任命槽位2(军师)
    pub off_0f: u16,                // 主要资源(金)
    pub off_11: u16,                // 次要资源(粮)
    pub off_13: u16,                // 第三资源(兵)
    pub off_15: u16,                // 治理属性
    pub off_17: u8,                 // 武将字节
    pub off_18: u8,                 // 发展参数A
    pub off_19: u8,                 // 发展参数B
    pub off_1a: u8,                 // 条件属性
    pub off_1c: u8,                 // 乘数属性
    pub off_1d: u8,                 // 可消耗资源A
    pub off_1e: u8,                 // 可消耗资源B
    pub off_1f: u8,                 // 加权属性输入
    pub off_20: u16,                // 资源属性20
    pub off_22: u16,                // 资源属性22
    pub off_24: u16,                // 资源属性24
    pub off_26: u16,                // 资源属性26
    // 0x28-0x2E: 扩展可消耗属性
    pub roster: Vec<u16>,           // 偏移0x37, 武将名册(每行4字节=2武将)
    pub bitflags: u16,              // 位域标志
    pub relation_matrix: Vec<u8>,   // 邻接关系矩阵
}
```

#### 地图 (Map)
```rust
/// 地图网格 — 20列×11行
/// 对应规格：/20-viewport-and-map-rendering
pub struct GameMap {
    pub width: u16,                 // 20 (0x14)
    pub height: u16,                // 11 (0x0B)
    pub tiles: Vec<u16>,            // 20×11=220个16位条目
    pub scroll_x: i16,
    pub scroll_y: i16,
    pub pixel_width: u16,           // width << 4
    pub pixel_height: u16,          // height << 4
}

impl GameMap {
    /// 获取瓦片类型（低4位）
    pub fn get_tile_type(&self, col: u8, row: u8) -> u8 {
        let idx = (row as usize) * 20 + (col as usize);
        (self.tiles[idx] & 0x0F) as u8
    }
    
    /// 获取瓦片完整值
    pub fn get_tile(&self, col: u8, row: u8) -> u16 {
        let idx = (row as usize) * 20 + (col as usize);
        self.tiles[idx]
    }
    
    /// 设置瓦片值
    pub fn set_tile(&mut self, col: u8, row: u8, val: u16) {
        let idx = (row as usize) * 20 + (col as usize);
        self.tiles[idx] = val;
    }
}
```

#### 势力 (Faction)
```rust
pub struct Faction {
    pub id: u8,
    pub cities: Vec<u8>,            // 控制的城市ID列表
    pub color: u8,                  // 势力颜色
    pub gold: u16,
    pub food: u16,
}
```

#### 部队 (Unit)
```rust
pub struct Unit {
    pub id: u16,
    pub officer_id: u16,
    pub col: u8,
    pub row: u8,
    pub flags: u8,                  // flag5, flag6等状态位
    pub attr_1b: u8,                // 移动力
    pub attr_17: u16,               // 综合属性
}
```

### 3.2 AI决策系统 (s4-core/ai/)

基于规格：/12-turn-execution-pipeline, /13-officer-scoring-and-selection, /14-move-evaluation-and-pathfinding

```rust
/// AI决策管道
pub struct AiEngine {
    officer_shuffle: [u8; 256],     // 武将顺序随机化表
    score_context: ScoreContext,    // 评分上下文寄存器
}

/// 评分上下文 — 对应0x937E, 0x9380, 0x9382
pub struct ScoreContext {
    pub action_param: u16,          // 0x937E
    pub threshold: u16,             // 0x9380
    pub position_flag: u8,          // 0x9382
}

impl AiEngine {
    /// 完整回合扫描 — 11×20网格
    /// 规格：/11-round-processing-and-turn-dispatch
    pub fn execute_full_pass(&mut self, game: &mut GameState) {
        for row in 0..11 {
            for col in 0..20 {
                if row == 10 && (col & 1) != 0 {
                    continue; // 跳过第10行奇数列
                }
                self.execute_turn(game, col, row);
            }
        }
    }
    
    /// 单回合执行
    pub fn execute_turn(&mut self, game: &mut GameState, col: u8, row: u8) {
        // 1. 验证单位存在
        // 2. 获取瓦片类型
        // 3. 检查游戏标志
        // 4. 执行最多3次移动
        // 5. 可选第4次移动（flag6设置时）
        // 6. 场景渲染
    }
    
    /// 武将评分分派 — 策略模式
    pub fn officer_score_dispatch<F>(&self, officers: &[Officer], scorer: F) -> Option<(usize, u16)>
    where F: Fn(&Officer) -> u16
    {
        let mut best_id = -1i16;
        let mut best_score = 0u16;
        for (i, officer) in officers.iter().enumerate() {
            let score = scorer(officer);
            if score > best_score {
                best_score = score;
                best_id = i as i16;
            }
        }
        if best_id >= 0 {
            Some((best_id as usize, best_score))
        } else {
            None
        }
    }
    
    /// 军事评分
    pub fn score_military(&self, officer: &Officer, game: &GameState) -> u16 { ... }
    
    /// 攻击评分（含财富门槛）
    pub fn score_attack(&self, officer: &Officer, game: &GameState) -> u16 { ... }
    
    /// 发展评分
    pub fn score_develop(&self, officer: &Officer, game: &GameState) -> u16 { ... }
    
    /// 招募评分
    pub fn score_recruit(&self, officer: &Officer, game: &GameState) -> u16 { ... }
    
    /// 特殊评分（含反雪球机制）
    pub fn score_special(&self, officer: &Officer, game: &GameState) -> u16 { ... }
}
```

### 3.3 移动系统 (s4-core/map/)

基于规格：/14-move-evaluation-and-pathfinding

```rust
/// 移动评估器
pub struct MovementEvaluator;

impl MovementEvaluator {
    /// 两阶段评估管道
    pub fn eval_two_phase(unit: &Unit, game: &GameState) -> i32 {
        if unit.flags & (FLAG5 | FLAG6) == 0 {
            return Self::evaluate_move_cost(unit, game);
        }
        // 阶段1: 探测并评估
        let phase1 = Self::probe_and_eval(unit, game);
        if phase1 == -1 { return -1; }
        // 阶段2: 从原位评估
        let phase2 = Self::evaluate_move_cost(unit, game);
        if phase2 == -1 { return -1; }
        // 饱和加法合并
        Self::saturating_add_wrap(phase1, phase2)
    }
    
    /// 完整移动成本评估
    pub fn evaluate_move_cost(unit: &Unit, game: &GameState) -> i32 {
        let cost = Self::lookup_cost_table(unit);
        if cost == 0xFF { return -1; }          // 不可通过
        if cost > unit.attr_1b { return -1; }   // 移动力不足
        // ... 更多检查
        cost as i32
    }
    
    /// 距离启发式（交错网格优化）
    pub fn calc_distance_heuristic(
        from_col: u8, from_row: u8,
        to_col: u8, to_row: u8
    ) -> u16 {
        let dx = (from_col as i16 - to_col as i16).abs() as u16;
        let dy = (from_row as i16 - to_row as i16).abs() as u16;
        // 切比雪夫距离变体，考虑交错网格对角线移动
        if (from_row & 1) != 0 {
            let adjusted = dx / 2;
            adjusted.max(dy) + adjusted
        } else {
            let adjusted = (dx + 1) / 2;
            adjusted.max(dy) + adjusted
        }
    }
    
    /// 地形成本表查找
    fn lookup_cost_table(unit: &Unit) -> u8 { ... }
    
    /// 可达瓦片谓词
    pub fn pred_reachable_tile(unit: &Unit, game: &GameState) -> bool { ... }
}
```

### 3.4 场景引擎 (s4-core/scene/)

基于规格：/10-scene-manager-and-state-transitions, /9-main-game-loop

```rust
/// 场景管理器
pub struct SceneManager {
    current_scene: Option<SceneId>,
    scene_stack: Vec<SceneState>,
    data_buffers: HashMap<u16, Vec<u8>>,  // 段标识→数据缓冲
}

/// 场景状态描述符（位域编码）
pub struct SceneState {
    pub raw_value: u16,              // 位0-1=场景类型, 位8=EMS, 位9=LoadAndExec
    pub handle: u16,
    pub data: Vec<u8>,
}

impl SceneManager {
    /// 主分派循环
    pub fn dispatch(&mut self, game: &mut GameState) {
        // 1. 消息缓冲推送
        // 2. 动画延迟
        // 3. 数据缓冲检查
        // 4. 条件处理
        // 5. 最终化
    }
    
    /// 场景处理（两个分支）
    pub fn process(&mut self, game: &mut GameState) {
        let state = self.lookup_scene_state();
        if state == -1 {
            self.exit_cleanup(game);
            self.evaluate_officers(game);
        } else {
            self.route(state);
            self.setup_state(game);
        }
    }
    
    /// 状态初始化（检查然后执行模式）
    pub fn init_state(&mut self, state: SceneState) -> Result<(), StateError> {
        // 检查EMS位、LoadAndExec位
        // 调用场景管理器获取句柄
        // 验证场景
        // 创建新状态句柄
        Ok(())
    }
}
```

### 3.5 资源管道 (s4-resource/)

基于规格：/25-resource-loading-and-decompression, /26-lzss-decompression

```rust
/// 资源管理器
pub struct ResourceManager {
    resource_table: Vec<ResourceEntry>,
    ems_buffer: Option<EmsBuffer>,  // EMS内存模拟
    decompressor: LzssDecompressor,
}

/// 资源句柄
pub struct ResourceHandle {
    pub file_handle: Option<File>,
    pub source_ptr: Option<Vec<u8>>,
    pub mem_handle: Option<u32>,
    pub mode: ResourceMode,          // 磁盘/内存
}

pub enum ResourceMode {
    Disk,
    Memory,
}

/// LZSS解压缩器（伽马编码比特流）
pub struct LzssDecompressor {
    current_byte: u8,
    bit_counter: i8,
    buffer: Vec<u8>,
    buffer_pos: usize,
    source: Box<dyn ByteSource>,
}

impl LzssDecompressor {
    /// 伽马编码比特流读取
    pub fn gamma_read(&mut self) -> Option<i32> {
        // 阶段1: 一元前缀（连续1-bit计数）
        let mut prefix_value: i32 = 0;
        let mut prefix_bits: i32 = 0;
        loop {
            let bit = self.read_bit()?;
            if bit {
                prefix_value = prefix_value * 2 + 1;
                prefix_bits += 1;
            } else {
                prefix_bits += 1;
                break;
            }
        }
        // 阶段2: 二进制后缀
        let mut suffix_value: i32 = 0;
        for _ in 0..prefix_bits {
            let bit = self.read_bit()?;
            suffix_value = suffix_value * 2 + bit as i32;
        }
        Some(prefix_value * 2 + suffix_value)
    }
    
    /// LZSS解压缩循环
    pub fn decompress(&mut self, output: &mut Vec<u8>, size: usize) -> bool {
        let mut remaining = size;
        while remaining > 0 {
            let token = match self.gamma_read() {
                Some(v) => v,
                None => return false,
            };
            if token < 0x100 {
                // 文字标记：查表后写入
                let byte = self.lookup_literal_table(token as u8);
                output.push(byte);
                remaining -= 1;
            } else {
                // 后向引用标记
                let distance = token as usize - 0x100;
                let length_code = self.gamma_read().unwrap_or(0);
                let length = (length_code as usize) + 3;
                let src_pos = output.len().saturating_sub(distance);
                for i in 0..length {
                    let byte = output[src_pos + i % distance];
                    output.push(byte);
                }
                remaining -= length;
            }
        }
        true
    }
}
```

### 3.6 渲染层 (s4-render/)

基于规格：/18-vga-mode-and-display-setup, /19-tile-and-sprite-blitting-engine, /20-viewport-and-map-rendering, /21-fade-and-scroll-animation

```rust
/// 视口管理器
pub struct Viewport {
    pub width: u16,                 // 640
    pub height: u16,                // 480
    pub tile_w: u8,                 // 15
    pub tile_h: u8,                 // 16
    pub scroll_x: i16,
    pub scroll_y: i16,
    pub clip_rect: Rect,
    pub content_buffer: Option<Vec<u8>>,
}

/// 瓦片渲染器
pub struct TileRenderer {
    tile_buffer: Vec<u8>,           // 128字节瓦片缓冲
    palette: Palette,
}

impl TileRenderer {
    /// 交错等距投影坐标变换
    /// 规格：/20-viewport-and-map-rendering
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
    
    /// 渲染瓦片（含分层合成）
    pub fn render_tiles(
        &mut self, 
        viewport: &Viewport,
        map: &GameMap,
        flags: u16,                  // 位域控制叠加层
        col: u8, row: u8
    ) {
        // 处理2×2子瓦片块
        for i in 0..4 {
            let sub_col = (i & 1) as u8 + col;
            let sub_row = (i >> 1) as u8 + row;
            // 绘制基础地形
            self.draw_map_tile(map, sub_col, sub_row);
            // 根据flags叠加层合并
            if flags & 0x001 != 0 { self.merge_event_4(); }
            if flags & 0x008 != 0 { self.render_unit(); }
            if flags & 0x020 != 0 { self.merge_event_3(); }
            // ...
        }
    }
    
    /// 平面合并（透明精灵合成）
    /// 规格：/19-tile-and-sprite-blitting-engine
    pub fn merge_planes_simple(dst: &mut [u8], src: &[u8]) {
        for i in 0..16 {
            let mask = !(src[i] | src[i+16] | src[i+32] | src[i+48]);
            dst[i]      = (dst[i] & mask) | src[i];
            dst[i+16]   = (dst[i+16] & mask) | src[i+16];
            dst[i+32]   = (dst[i+32] & mask) | src[i+32];
            dst[i+48]   = (dst[i+48] & mask) | src[i+48];
        }
    }
}

/// 调色板管理
pub struct Palette {
    colors: [Color; 256],
}

/// 淡入淡出动画
pub struct FadeAnimator {
    accumulator: u32,               // 32位定点累加器
    velocity: u32,                  // 速度向量
}

impl FadeAnimator {
    /// 每帧更新（定点积分）
    pub fn tick(&mut self) -> u16 {
        self.accumulator = self.accumulator.wrapping_add(self.velocity);
        (self.accumulator >> 16) & 0x7FFF
    }
    
    /// 线性插值
    pub fn lerp(start: i32, end: i32, t: u16) -> i32 {
        let normalized = t as i32 * 100 / 0xB0;
        start + normalized * (end - start) / 100
    }
}
```

### 3.7 UI框架 (s4-ui/)

基于规格：/22-entity-vtable-and-event-dispatch, /23-widget-and-dialog-system

```rust
/// 实体系统（vtable多态调度）
pub struct EntitySystem {
    entities: Vec<Entity>,
    root_entity_id: Option<usize>,
}

pub struct Entity {
    pub vtable_id: u16,             // vtable类型标识
    pub rect: Rect,
    pub flags: u16,
    pub flag_byte: u8,              // bit0=可见, bit7=模态
    pub children: Vec<usize>,       // 子实体ID列表
    pub parent: Option<usize>,
    pub viewport_origin: (i16, i16),
    pub work_rect: Rect,
}

/// 对话框引擎（标志驱动变体）
pub struct DialogEngine {
    mode_flag: bool,                // 0x7702
    ready_flag: bool,               // 0x75B0
    result_value: i16,              // 0x75B2
}

impl DialogEngine {
    /// 主显示循环
    pub fn show(
        &mut self,
        text: &str,
        flags: u8,                  // 位域控制UI元素
        options: Option<&[&str]>,
        mapping: Option<&[u8]>,
    ) -> DialogResult {
        // 根据flags位设置UI元素
        // 进入do-while循环
        // 等待输入
        // 返回结果
        unimplemented!()
    }
    
    /// 阻塞输入等待
    pub fn input_wait(&mut self) -> i16 {
        // 重置标志
        // 事件处理循环
        // 返回0x75B2值
        unimplemented!()
    }
}
```

## 四、回合处理管道

基于规格：/11-round-processing-and-turn-dispatch

```
┌─────────────────────────────────────────────────────┐
│                  round_process()                     │
├─────────────────────────────────────────────────────┤
│ Phase 1: AI移动与回合执行                            │
│   ├─ 检查dialog_slot_attr15 & bitratio_a            │
│   ├─ unit_move_with_ai() (最多3次迭代)              │
│   ├─ map_set_tile_value()                           │
│   └─ ai_execute_turn()                              │
├─────────────────────────────────────────────────────┤
│ Phase 2: 城市资源重算                                │
│   ├─ bitratio_e → Off13路径 (兵)                    │
│   ├─ bitratio_d → Off11路径 (粮)                    │
│   └─ 条件Off13二次修改                              │
├─────────────────────────────────────────────────────┤
│ Phase 3: 场景引导与清理                              │
│   ├─ scene_bootstrap() → 城市经济模拟               │
│   ├─ dialog_list_remove()                           │
│   ├─ officer_loyalty_ratio_handler()                │
│   ├─ dialog_slot_init_all_attrs()                   │
│   ├─ gfx_dual_refresh()                             │
│   └─ map_find_first_occupied()                      │
└─────────────────────────────────────────────────────┘
```

## 五、关键设计决策

### 5.1 现代化vs原版行为对照

| 原版概念 | Rust现代对应 | 设计选择 |
|---------|------------|---------|
| DOS段:偏移寻址 | Rust引用/指针 | 不需要段模型，直接用内存地址 |
| 覆盖系统(OVR) | 模块化crate | 现代动态库替代覆盖加载 |
| EMS扩展内存 | Vec\<u8\>/HashMap | 现代内存管理器，无需分页 |
| 全局状态地址 | GameState结构体 | 所有状态封装在GameState中 |
| vtable指针 | trait对象/dyn Trait | Rust的动态分派机制 |
| INT 21h/10h调用 | 平台抽象层trait | 通过trait封装系统调用 |
| VGA模式X平面渲染 | wgpu纹理 | GPU加速的现代渲染管线 |
| 伽马编码比特流 | Rust迭代器/Read trait | 标准库I/O抽象 |

### 5.2 GameState — 统一状态容器

```rust
/// 游戏全局状态 — 替代原版所有固定地址全局变量
pub struct GameState {
    // 核心数据
    pub officers: Vec<Officer>,          // 401个武将
    pub cities: Vec<City>,               // 42个城市
    pub factions: Vec<Faction>,          // 7个势力
    pub map: GameMap,                    // 20×11地图
    pub units: Vec<Unit>,                // 活跃部队
    
    // 时间系统
    pub turn: u16,
    pub year: u16,
    pub month: u8,
    pub day: u8,
    pub time_ticks: u32,                 // (hours×60+min)×75+sec-150
    
    // 场景状态
    pub scene_manager: SceneManager,
    pub active_unit: Option<usize>,
    pub game_mode: u8,                   // 0=战略, 3/4=战斗
    
    // 资源管理
    pub resource_manager: ResourceManager,
    
    // 随机数
    pub rng: Rng,
    
    // 音频状态
    pub audio: AudioState,
    
    // 事件标志
    pub flags: GameStateFlags,
}

pub struct GameStateFlags {
    pub exit_flag: bool,                 // 对应0x39EC
    pub normal_state: bool,              // 对应game_check_flag
    pub alternate_mode: bool,            // 对应0x8564
    // ... 其他标志
}
```

### 5.3 平台抽象层接口

```rust
/// 平台抽象 — 替代DOS中断调用
pub trait Platform {
    /// 文件I/O
    fn open_file(&mut self, path: &str) -> Result<FileHandle, IoError>;
    fn read_file(&mut self, handle: FileHandle, buf: &mut [u8]) -> Result<usize, IoError>;
    fn close_file(&mut self, handle: FileHandle) -> Result<(), IoError>;
    
    /// 显示
    fn set_video_mode(&mut self, mode: VideoMode) -> Result<(), DisplayError>;
    fn draw_pixels(&mut self, x: u16, y: u16, w: u16, h: u16, data: &[u8]) -> Result<(), DisplayError>;
    fn present(&mut self) -> Result<(), DisplayError>;
    fn wait_vsync(&mut self);
    
    /// 输入
    fn poll_key(&mut self) -> Option<KeyEvent>;
    fn poll_mouse(&mut self) -> Option<MouseEvent>;
    
    /// 音频
    fn play_midi(&mut self, track: u8) -> Result<(), AudioError>;
    fn play_sound(&mut self, effect: u8) -> Result<(), AudioError>;
    
    /// 时间
    fn get_ticks(&self) -> u64;
    fn delay(&mut self, ms: u32);
    
    /// 随机数
    fn random_u32(&mut self) -> u32;
}
```

## 六、构建系统

```toml
# 根 Cargo.toml
[workspace]
members = [
    "crates/s4-core",
    "crates/s4-platform",
    "crates/s4-resource",
    "crates/s4-render",
    "crates/s4-ui",
    "crates/s4-app",
]
resolver = "2"
```

## 七、实施阶段建议

### 阶段一（已完成）: 架构规划 ✓

### 阶段二: 逐模块实现

按以下顺序：
1. **s4-platform** — 平台抽象层（文件、时间、随机数）
2. **s4-core/data** — 数据结构定义
3. **s4-resource** — LZSS/RLE解压缩
4. **s4-core/map** — 地图与移动系统
5. **s4-core/city** — 城市管理
6. **s4-core/ai** — AI决策系统
7. **s4-core/battle** — 战斗系统
8. **s4-core/round** — 回合处理管道
9. **s4-core/scene** — 场景引擎
10. **s4-render** — 渲染管道
11. **s4-ui** — UI框架
12. **s4-app** — 集成与主循环

### 阶段三: 集成与验证

1. 全模块集成编译
2. 功能性测试（核心游戏流程：开局→内政→战斗→结局）
3. 与规格文档对齐验证

## 八、规格文档引用索引

每个Rust模块实现时必须在文档注释中引用对应规格章节：

| Rust模块 | 规格页面 | 核心内容 |
|---------|---------|---------|
| data/officer | /15, /16 | 武将位域访问器、槽位管理 |
| data/city | /17 | 资源积累与消耗 |
| data/map | /20 | 地图数据结构 |
| ai/* | /12, /13, /14 | AI管道、评分、寻路 |
| round/* | /11 | 回合处理 |
| scene/* | /10 | 场景管理器 |
| map/movement | /14 | 移动评估 |
| resource/lzss | /26 | LZSS解压缩 |
| resource/loader | /25 | 资源加载管道 |
| render/* | /18, /19, /20, /21 | VGA、瓦片、视口、动画 |
| ui/entity | /22 | 实体vtable系统 |
| ui/dialog | /23 | 对话框引擎 |
| core/game_loop | /9 | 主游戏循环 |
