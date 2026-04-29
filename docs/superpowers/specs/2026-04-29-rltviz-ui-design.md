# rltviz UI 全面升级设计规格

**日期**: 2026-04-29
**目标**: 将 rltviz 的 egui 0.31 UI 升级为简洁有质感的 Apple 浅色风格

## 设计原则

- 单色系灰阶 + 单一 Blue 强调色 (`#007AFF`)
- 用留白和微妙背景色差分层，不用粗重边框 / separator
- 纯平设计，仅用极细线 + 微圆角，不加阴影
- 字重 + 灰度区分信息层级，减少字号变化

---

## 1. 配色系统

在 `src/theme.rs` 中统一定义 color palette 常量，所有 UI 代码通过名称引用，不再出现裸 `Color32::from_rgb()`。

| Token | 色值 | 用途 |
|---|---|---|
| `bg_primary` | `#FFFFFF` | 右侧主面板背景 |
| `bg_secondary` | `#F5F5F7` | 左侧侧栏背景 |
| `bg_tertiary` | `#EBEBED` | 输入框/卡片填充底 |
| `border` | `#D1D1D6` | 分割线、输入框边框 |
| `text_primary` | `#1D1D1F` | 主文字 |
| `text_secondary` | `#6E6E73` | 辅助说明 |
| `text_tertiary` | `#AEAEB2` | placeholder、弱提示 |
| `accent` | `#007AFF` | 强调色（启动按钮、QPS 指标、选中态） |
| `accent_hover` | `#0062CC` | 按下/hover |
| `positive` | `#34C759` | 低错误率 |
| `negative` | `#FF3B30` | 高错误率/停止按钮 |
| `warning` | `#FF9500` | 暂停/中间状态 |

---

## 2. 排版系统

全局 Proportional 用系统默认 + Noto Sans SC fallback，Monospace 用 JetBrains Mono + Noto Sans SC fallback。

| Token | 字号 | 字重 | 颜色 | 场景 |
|---|---|---|---|---|
| `heading` | 15px | Semibold | `text_primary` | section 标题 |
| `body` | 13px | Regular | `text_primary` | form label、普通文字 |
| `body_small` | 11px | Regular | `text_secondary` | 图表 label、辅助信息 |
| `metric_value` | 28px | Bold | `accent` | 核心指标数值 |
| `metric_label` | 11px | Medium | `text_tertiary` | 指标名称 |
| `mono` | 12px | Regular | `text_primary` | URL/Body/Headers 编辑 |
| `btn_text` | 13px | Semibold | `#FFFFFF` | 按钮文字 |

---

## 3. 布局与间距

### 整体框架
- 左侧 `SidePanel::left`：默认 340px，最小 280px，背景 `bg_secondary`
- 右侧 `CentralPanel`：背景 `bg_primary`，24px 内边距

### 间距尺度
- **4px**：表单内 label↔控件
- **8px**：同组控件之间、按钮间距
- **18px**：cards/sections 之间
- **24px**：右侧不同 chart 区块之间

### 左侧面板组织
Section 间用留白分层，不用 `ui.separator()`：

```
┌─ "压测配置" (heading) ────────┐
│  [从 cURL 导入] (文字链接)    │  ← 12px
├─ URL ─────────────────────────┤  ← 8px
│  Method dropdown              │
├─ Headers ─────────────────────┤  ← 可折叠
│  [+] 添加 Header               │
├─ Body (仅 POST/PUT) ─────────┤
│  textarea                     │
│                              │  ← 18px
├─ "阶梯加压" (heading) ───────┤
│  DragValue 控件               │
│  阶段预览 bars                  │
│                              │  ← 18px
├─ 控制按钮 ────────────────────│
└──────────────────────────────┘
```

### 右侧面板组织
- Idle 状态：居中引导文字
- Running/Paused：4 个指标卡片 → 延迟图 → 状态码图，section 间距 24px
- Stopped：汇总卡片（无 separator）

---

## 4. 组件设计

### 按钮
- 主按钮（启动/恢复）：`accent` 填充白字，圆角 6px，高 32px，最小宽 100px
- 暂停按钮：`warning` 填充白字
- 停止按钮：`negative` 填充白字
- 次要按钮（取消、导入、添加 Header）：纯文字 + `accent` 色，无背景填充

### 输入控件
- `TextEdit`：圆角 6px，边框 `border`，focus 态变 `accent`
- `DragValue`：去除默认装饰，简洁输入框
- 下拉框：与 TextEdit 统一圆角和边框

### 指标卡片（右侧顶部）
- 纯白底 + 底部 1px `border` 分割线（替代现有的 `#FAFAFA` 填充卡）
- 数值 28px Bold `accent` 色，标签 11px `text_tertiary`
- `ui.columns()` 等宽排布，内边距 12px
- 错误率 >5% 时数值变 `negative`

### 延迟柱状图
- 横柱高 32px
- 颜色统一 `accent` 蓝（不用绿/橙/红三色）
- P50/P90/P99 标签左侧，值右侧
- 底部阶梯信息用 `body_small` 弱化

### 状态码分布图
- 颜色收拢为灰阶：2xx=深灰，3xx=中灰，4xx=浅灰，5xx=accent 蓝（标识关注）
- 横条高 20px，间距 6px

### Section 标题
- 极简纯文字 + 留白区分组，不加装饰条或 separator

---

## 5. 实现策略

### 修改清单

| 文件 | 改动内容 |
|---|---|
| `src/theme.rs` | 扩写为 palette 常量 + font token 函数 + `apply_theme()` 设置全局 visuals |
| `src/app.rs` | 调整 panel 样式（frame 背景色、margin），调整 `update()` 布局 |
| `src/ui/stat_cards.rs` | 改卡片样式（白底+底线），统一次要文字颜色 |
| `src/ui/latency_chart.rs` | 柱体颜色统一 accent，调整尺寸和排版 |
| `src/ui/status_chart.rs` | 颜色改灰阶+accent，调整尺寸 |
| `src/ui/control_bar.rs` | 按钮样式统一（填充色、圆角、大小），次要按钮改文字链接 |
| `src/ui/config_panel.rs` | 输入控件统一样式，section 组织改用留白，import 按钮改次要风格 |
| `src/ui/dashboard.rs` | 去除 separator，调整间距，Idle/Stopped 状态排版优化 |

### 不变部分
- `src/config.rs` / `src/control.rs` / `src/engine.rs` / `src/metrics.rs` — 完全不改
- 窗口大小 1280x800 不变
- 字体文件不变

---

## 6. 验证方案

1. `cargo check` — 编译通过
2. `cargo clippy -- -D warnings` — 无 lint 警告
3. `cargo test` — 所有现有测试通过
4. `cargo run --release` — 启动 GUI：
   - 检查 Idle 状态左侧/右侧布局
   - 填写配置、修改 Headers、调整阶梯参数
   - 启动压测，观察指标卡片、延迟图、状态码图实时更新
   - 暂停/恢复/停止，确认各状态 UI 切换正确
   - 测试 cURL 导入弹窗
