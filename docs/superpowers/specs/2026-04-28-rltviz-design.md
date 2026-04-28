# rltviz — 可视化 HTTP 压测工具 设计文档

**日期**: 2026-04-28  
**状态**: 待实现  

## 概述

基于 rlt + egui 的桌面 HTTP 压测可视化工具。目标用户为小团队内部使用，纯 GUI 表单配置，零代码操作。

## 技术栈

- **GUI 框架**: egui (via eframe)
- **压测引擎**: rlt (HTTP)
- **图表**: egui_plot
- **异步运行时**: Tokio
- **共享状态**: Arc + parking_lot::RwLock

## 架构

三层结构，GUI 与引擎之间通过 AppCore 中间层解耦：

```
GUI 层 (egui)        配置面板 / 控制栏 / 指标仪表盘
  ↕ 共享状态读取
AppCore 层           配置管理 / 生命周期控制 / 指标聚合
  ↕ rlt API 调用
引擎层 (rlt)         HTTP 压测执行，输出 metrics 事件
```

关键设计理由：GUI 不直接依赖 rlt API，引擎更换或升级时只改 AppCore 层。

## 数据流

```
rlt Executor → 原始事件流
  ↓
MetricsCollector (独立 tokio task)
  → 滑动窗口聚合 (1秒粒度)
  → 计算 QPS / 延迟分位数 / 错误率 / 状态码分布
  → 写入 Arc<RwLock<MetricsSnapshot>>
  ↓
egui 每帧读取 MetricsSnapshot → 渲染图表
```

MetricsSnapshot 数据结构：

```rust
struct MetricsSnapshot {
    qps: f64,
    latency_p50: Duration,
    latency_p90: Duration,
    latency_p99: Duration,
    error_rate: f64,
    status_codes: HashMap<u16, u64>,
    active_connections: u32,
    total_requests: u64,
    elapsed: Duration,
    current_step: u32,       // 当前阶梯编号
    step_progress: f64,      // 当前阶梯进度 (0.0~1.0)
}
```

## 功能范围

### MVP 功能

- HTTP/HTTPS 压测（GET/POST/PUT/DELETE）
- 纯 GUI 表单配置（URL、Method、Headers、Body、并发参数）
- 阶梯式加压（起始并发 → 最终并发，N 步，每步固定时长）
- 运行时控制：启动 / 暂停 / 恢复 / 停止
- 实时指标面板：
  - 统计卡片：QPS、错误率、活跃连接数
  - 延迟折线图：P50 / P90 / P99（带阶梯标记竖线）
  - 状态码柱状图：2xx / 3xx / 4xx / 5xx 分组
- 运行结束后显示汇总指标

### 不做（YAGNI）

- 不保存历史记录
- 不支持 gRPC / WebSocket / Thrift
- 不支持脚本/代码配置
- 不支持多用户
- 不支持分布式压测

## UI 布局

单窗口左右分栏：

- **左侧面板 (~35% 宽度)**：配置表单 + 控制按钮
  - URL 输入框、HTTP Method 下拉
  - Headers 键值对编辑（可增删行）
  - Body 文本编辑区（POST/PUT 时显示）
  - 阶梯加压参数：起始并发、最终并发、阶梯数、每阶时长
  - 实时预览柱状图
  - 控制栏：启动 / 暂停(恢复) / 停止
  - 压测运行时左侧可折叠收窄
- **右侧面板 (~65% 宽度)**：实时指标仪表盘
  - 顶部统计卡片行（QPS / 错误率 / 连接数 / 阶梯进度）
  - 延迟折线图（主图，占据大部分空间）
  - 状态码柱状图（底部）

## 阶梯式加压

### 配置参数

| 参数 | 说明 | 默认值 |
|------|------|--------|
| 起始并发 | 第一阶段的并发连接数 | 10 |
| 最终并发 | 最终阶段的并发连接数 | 100 |
| 阶梯数 | 中间递增步数（不含起始阶段） | 5 |
| 每阶时长 | 每个阶梯持续秒数 | 30 |

### 阶段计算

总阶段数 = 阶梯数 + 1（含起始阶段）  
并发步长 = (最终并发 - 起始并发) / 阶梯数

示例（起始10, 最终200, 5步, 30s/步）：共6阶段，每步 +38 并发。

### 运行时行为

- **启动**：从起始并发开始，每阶结束自动递增
- **暂停**：保持在当前阶梯，当前并发不变，恢复后继续计时
- **停止**：立即终止，显示汇总
- 延迟图上用竖虚线标记阶梯切换点

## 项目结构

```
src/
├── main.rs              # 入口
├── app.rs               # eframe::App impl，持有全局状态
├── config.rs            # HttpConfig + RampUpConfig
├── control.rs           # TestController 生命周期管理
├── metrics.rs           # MetricsCollector + MetricsSnapshot
├── ui/
│   ├── mod.rs
│   ├── config_panel.rs  # 配置表单
│   ├── control_bar.rs   # 控制按钮
│   ├── dashboard.rs     # 指标面板布局
│   ├── stat_cards.rs    # 统计卡片
│   ├── latency_chart.rs # 延迟折线图
│   └── status_chart.rs  # 状态码柱状图
└── theme.rs             # egui 视觉风格
```

## 依赖 (Cargo.toml)

```toml
[dependencies]
eframe = "0.31"
egui_plot = "0.31"
rlt = "..."              # 版本待确认
tokio = { version = "1", features = ["full"] }
serde = "1"
serde_json = "1"
parking_lot = "0.12"
```

## 待确认事项

1. rlt crate 的具体版本号和 API —— 实现前需先查阅 rlt 文档
2. rlt 是否支持暂停/恢复操作 —— 如不支持，需在 AppCore 层自行实现（停止当前 executor + 记录状态 + 重建 executor）
