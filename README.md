# rltviz

基于 Rust + egui + Tokio 构建的桌面端 HTTP 压测工具。

## 功能特性

- **实时指标面板** — QPS、错误率、活跃连接数、总请求数
- **延迟分位数** — P50 / P90 / P99 柱状图，附带阶梯进度
- **状态码分布** — 按状态码类别分组的水平柱状图
- **渐进式加压** — 可配置起始/结束并发数、阶梯数和每阶时长
- **curl 导入** — 粘贴 curl 命令自动填充 URL、方法、请求头和请求体
- **暂停 / 恢复** — 暂停压测而不丢失已采集的指标数据
- **自动停止** — 最后一个加压阶段完成后自动结束测试

## 快速开始

### 环境要求

- Rust 1.75+（edition 2021）
- Windows / macOS / Linux

### 构建与运行

```bash
cargo run --release
```

Release 构建会嵌入字体文件，首次构建可能需要几分钟。

## 使用方法

1. **配置目标** — 输入 URL、选择 HTTP 方法、添加请求头/请求体，或直接粘贴 curl 命令
2. **设置加压参数** — 设定起始并发、结束并发、阶梯数和每阶时长
3. **开始测试** — 点击 Start 开始压测，指标实时刷新
4. **过程控制** — Pause 暂停工作线程，Resume 继续，Stop 提前结束

## 架构

```
src/
├── main.rs              # 入口，初始化 tokio 运行时与 eframe 窗口
├── app.rs               # egui::App 实现，左右面板布局
├── config.rs            # AppConfig / HttpConfig / RampUpConfig
├── control.rs           # TestController — 生命周期与加压调度
├── engine.rs            # HttpWorkerPool — 基于信号量的异步 HTTP 工作池
├── metrics.rs           # MetricsCollector — 逐请求聚合，分位数计算
├── theme.rs             # 自定义 egui 主题
└── ui/
    ├── config_panel.rs   # 左侧面板 — URL/方法/请求头/请求体/加压表单
    ├── control_bar.rs    # 开始/暂停/恢复/停止按钮
    ├── dashboard.rs      # 右侧面板，空闲/运行/停止三种状态布局
    ├── stat_cards.rs     # QPS/错误率/连接数/总数指标卡片
    ├── latency_chart.rs  # P50/P90/P99 延迟柱状图
    └── status_chart.rs   # 状态码水平柱状图
```

### 数据流

```
HTTP 工作线程 ──mpsc channel──► MetricsCollector ──100ms tick──► Arc<RwLock<MetricsSnapshot>>
                                                                       │
                                                                       ▼
                                                            egui::App::update() 每帧读取
```

### 状态机

```
Idle ──start()──► Running ──pause()──► Paused ──resume()──► Running
                    │                                         │
                    └──── 自动停止（加压完成）──────────────────┘
                    │                                         │
                    └──────── stop() ─────────────────────────┘
                                                              ▼
                                                           Stopped
```

## 开发

```bash
cargo build                 # debug 构建
cargo test                  # 运行全部测试
cargo clippy -- -D warnings # 代码检查
cargo check                 # 快速编译检查
```

## 依赖

| Crate | 用途 |
|-------|------|
| egui / eframe 0.31 | 即时模式 GUI |
| egui_plot 0.31 | 图表渲染 |
| tokio | 异步运行时 |
| reqwest | HTTP 客户端（rustls-tls） |
| parking_lot | 共享指标的 RwLock |
| serde / serde_json | 配置序列化 |
| curl-parser | curl 命令解析 |

## 许可证

MIT
