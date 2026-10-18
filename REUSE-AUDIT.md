# 全模块复用排查

本次按源码职责检查 `src/` 全部模块，以及 `examples/`、`tools/`、`scripts/` 和测试设施。
结论：之前“默认零依赖、自主实现所有底层”的方向与应用框架目标不符，现已撤销。
框架应聚焦应用接口、资源边界、生命周期和组件适配，通用算法优先使用成熟库。
项目级后续约束已写入 [AGENTS.md](AGENTS.md)。

“已采用”表示已进入调用路径；“待迁移”表示现有自写实现仍在；“待验证”表示候选尚未进行兼容性试验。
本清单完成的是全模块排查，不代表所有候选已替换完成。

| 模块及代码 | 当前判断 | 决策与剩余工作 |
| --- | --- | --- |
| Flex：`flex.rs`、`layout.rs` | 本轮已采用 Taffy 0.15.0 | 删除自写扩展、收缩、水平分布和垂直对齐求解器；保留 CSS 映射、子树测量结果和绘制记录适配。 |
| 块布局：`layout.rs` | 自写，待迁移 | 当前仍保留纵向流；下一阶段用完整布局树和文本测量回调接入 Taffy，不能把 Flex 接入宣称为全布局迁移。 |
| Grid、定位、百分比 | 未接入 | Taffy 有相关布局能力，但 RWE CSS 与树适配尚未暴露；不再自行补算法，也不宣称当前支持。 |
| HTML：`dom.rs`、`html.rs` | 已采用 html5ever 0.40.1 | 删除旧解析器；TreeSink 仅适配 arena 和预算，分词、实体及恢复由库承担。已验证片段上下文、表格、raw-text、命名空间和限制。 |
| JS HTML 解析：`runtime/dom.js::parseFragment` | 已统一 Rust 入口 | 删除正则解析与自写实体解码，使用 Boa 原生调用；用 serde_json/Boa 转换扁平数据，拒绝解析时保留旧子树。DOM serializer 和其他 DOM 行为仍待替换。 |
| CSS 词法/规则/声明：`css.rs`、`layout.rs`、`stylesheet.rs` | 已采用 cssparser 0.38.0 | 替换按分号/冒号/大括号切分及手工注释删除；JS cssText 共用原生入口。属性语义、选择器匹配和级联尚未迁移 Stylo，不能称为完整 CSS 引擎。 |
| 选择器/级联：`stylesheet.rs`，JS querySelector/matches | 双端子集，P0 待迁移 | 评估 Stylo/selectors 适配；结构伪类、优先级、继承、动态状态要共同验证，避免再造另一套选择器。 |
| DOM/Event/Form：`runtime/dom.js`、`form_control.rs` | 自写子集，P0 架构风险 | Boa 是 JS 引擎，不带浏览器 DOM；需验证可复用 DOM 内核与 Boa 绑定。Rust 绘制层可以保留控件视图适配，不能继续按案例无限堆 Web API 补丁。 |
| JS 引擎/任务：`runtime/mod.rs` | 已使用 Boa 0.22.0 | 保留引擎；资源预算、作业调度是宿主职责。复用 Boa API/宿主能力，字符串转义已迁移 serde_json，原生桥接使用 Boa 值转换。 |
| 模块/前端构建：`local_page.rs::PageScript`、examples vendor | 仅 classic/IIFE，待实现 | 接入标准 Vue/React 构建产物与 Boa 模块接口；复用 Vite/Rollup/SWC 生态，不实现自己的 JS/TS 编译器或 bundler。 |
| 字体栅格化：`font.rs` | 已使用 fontdue | 保留已有库集成与资源许可；字体缓存/嵌入是适配。不能把栅格化等同于 shaping/fallback。 |
| 文本换行/编辑：`text.rs`、`runtime/dom.js::previous/next/inputKey` | 自写，P1 待迁移 | 先验证 Parley 与现有 canvas/字体接口，cosmic-text 为对比候选；使用成熟文本栈承担 shaping、字体回退、bidi/行断；编辑须按 grapheme 而非仅 code point。 |
| 绘图/控件：`canvas.rs`、`color.rs`、`form_control.rs` | 自写矩形/混合/勾选标记，P1 待迁移 | tiny-skia 为 CPU 后端候选；Vello 为更大 GPU 方案候选。先验证直通/预乘 alpha 转换和裁剪一致性，再迁移，不能直接重解释 Color 内存。简单 Rect/Color 公共数据类型保留。 |
| PNG：`png.rs`、`tests/support/png.rs` | 本轮已采用 png 0.18.1 | 删除自写 CRC、Adler、DEFLATE 和测试解码器；库承担编码、过滤、压缩和解码。验证像素及 IO 错误，不再绑定未压缩字节。 |
| 图片/SVG | 当前未提供 | 图片复用 image/png；SVG 优先验证 resvg/usvg。禁止从零实现图片格式解析。 |
| 本地资源：`local_page.rs`、`asset_bundle.rs` | 部分保留 | std::fs/Path 是现成基础设施；大小/数量/目录边界和嵌入资产策略是框架职责。未来 URL 解析用 url，HTTP 用 reqwest；不把 URL 规范化当作文件访问授权。 |
| 窗口/输入法：`desktop.rs`、`window_host.rs`、`keyboard.rs`、`ime.rs` | 已使用 winit/softbuffer | 保留窗口和软件呈现库。事件映射、页面滚动、IME 协议桥接属于框架适配；跨平台系统行为仍须人工验证。 |
| 文件监听：`window_host.rs::stamp/poll_sources` | 自写轮询，P2 待迁移 | 采用 notify 方向，处理事件合并、替换写入和新资源引用；不继续扩展自写轮询监控。 |
| CLI：`main.rs`、`window_host.rs::Sources::parse` | 两套手写解析，P2 待迁移 | 采用 clap，共享参数结构和验证；保留无窗口 help/错误先行行为，避免第三套参数解析。 |
| 日志/错误：多模块 String/自定义错误 | P2 待统一 | tracing 为结构化诊断候选；错误保留公共类型并评估 thiserror。小型错误类型不必仅为减少几行代码引入宏。 |
| 无障碍/剪贴板 | 未提供 | AccessKit、arboard 为待验证候选；接入平台能力和权限策略，不编写原生协议实现。 |
| 网络/存储 | 未提供 | 网络用成熟 HTTP/URL/异步运行时；存储明确业务语义后选择已有数据库。未实现不是“已由 std 支持”，不自写网络协议、TLS 或文件数据库。 |
| 打包：`scripts/bundle-embedded-macos.sh` | 开发包脚本，P2 待迁移 | 保留开发产物；分发优先验证 cargo-packager 或平台标准工具，签名/公证/安装器不继续手工扩展成打包系统。 |
| 门面：`application.rs`、`desktop.rs` | 保留 | Application/DesktopApp、生命周期、嵌入资源接口是框架对用户的价值；内部算法委托给库，避免把适配层误判成“重复造轮子”。 |
| 示例/工具/测试 | 保留并约束 | 示例只写业务和配置；截图工具走公共 API；测试使用解码后像素/交互语义，第三方框架运行时保留官方版本与许可。 |

## 整体内核也要先评估

[Blitz](https://github.com/DioxusLabs/blitz) 提供模块化 HTML/CSS 内核，值得在继续分别组装
HTML、CSS、文本和渲染库之前进行端到端试验。上游明确仍处 beta，并不提供现成 JavaScript
语言绑定；所以不能把它直接当作 Vue/React DOM 的可用替代，也不能仅凭能渲染 HTML 就切换主路径。
试验验收必须包含真实 Vue/React 控制表单、模块化构建产物、中文编辑、命中测试和资源边界。

## 执行顺序和验收

1. 已落地：Taffy 替换已有 Flex 求解；png 替换编码；html5ever 统一初始页面与 innerHTML。明确依赖/字节/舍入变化，保持公共 API 与错误处理。
2. P0：比较“复用 Blitz 内核并绑定 JS”与“html5ever + Stylo + Taffy”两条路线，用同一真实应用验收，选一条主路径；两端 HTML 解析已统一，下一步统一 CSS/选择器与 DOM 内核。
3. P1：接入成熟文本与绘图栈，验证字体回退、中文/emoji grapheme 编辑和 alpha/裁剪；不再扩展点阵字体和手工行断算法作为主方案。
4. P2：统一 CLI、监听、诊断、无障碍、分发；网络与存储按应用能力接入。

P0/P1 候选当前仅完成源码职责和上游文档对照，不是版本锁定、完整安全审计、性能基准或移植验证。
本轮不同时替换所有模块，以免丢失行为基线；但后续新增功能必须先执行复用评估。

## 本轮 Taffy 适配边界

目前只暴露原有单行横向 Flex CSS 子集。已有文本子树先按 Taffy 分配的宽度布局，再将实际高度
交回 Taffy 计算对齐/拉伸，RWE 只移动绘制与命中记录。块流和文本换行暂仍自写，未接通 Taffy
完整树测量、intrinsic sizing、Grid、wrap 等能力。Taffy 使用 f32，本适配拒绝超出 2^24 像素的几何值，
避免悄悄丢失整数精度；正常窗口上限远低于此值。像素舍入遵循库行为，不兼容旧算法每一处舍入。
PNG 压缩字节会变化，但解码后的 RGBA（含透明像素的 RGB）保持不变。

## 上游依据

- [Taffy](https://github.com/DioxusLabs/taffy)、[png](https://github.com/image-rs/image-png)
- [html5ever](https://github.com/servo/html5ever)、[cssparser](https://github.com/servo/rust-cssparser)、[Stylo](https://github.com/servo/stylo)
- [Boa](https://github.com/boa-dev/boa)、[Parley](https://github.com/linebender/parley)、[cosmic-text](https://github.com/pop-os/cosmic-text)
- [tiny-skia](https://github.com/linebender/tiny-skia)、[Blitz](https://github.com/DioxusLabs/blitz)
- [winit](https://github.com/rust-windowing/winit)、[softbuffer](https://github.com/rust-windowing/softbuffer)
- [notify](https://github.com/notify-rs/notify)、[clap](https://github.com/clap-rs/clap)、[url](https://github.com/servo/rust-url)、[reqwest](https://github.com/seanmonstar/reqwest)

依赖精确版本、启用特性和递归许可证见 [CORE-DEPENDENCIES.md](CORE-DEPENDENCIES.md)，当前能力见 [COMPATIBILITY.md](COMPATIBILITY.md)。

## 本轮验证记录

全功能 175 项、默认构建 116 项测试（含文档测试）及全功能全目标 Clippy 通过。
21 张既有预览经 Pillow 解码后 RGBA 完全一致；PNG 压缩字节不作为兼容性契约。
macOS 开发包构建及 Info.plist 校验通过；原生窗口尚未人工验证。

## HTML 迁移补充

html5ever 已进入默认依赖；不采用 markup5ever_rcdom，其上游描述为 unsupported 测试 DOM，
不符合本项目公共 DOM 的要求。保留薄的 TreeSink/arena 适配没有重写 tokenizer 或恢复算法。
注释节点、namespace 字段和 HTML5 恢复是公开行为变化；畸形 HTML 不再可靠地触发错误，
失败保留原页面的测试改为真正的资源限制失败。模板内容在 JS 中归属于 content。
其他 DOM、序列化、CSS 和布局能力不因解析器迁移自动完成。

HTML5 迁移验证：全功能 182 项、默认构建 119 项测试（含文档测试）及 Clippy、fmt、diff 检查通过。

## CSS 语法迁移补充

cssparser 0.38.0 已接入 Rust 样式表、BoxStyle/内联声明与 JavaScript style.cssText，
删除旧的 split 规则解析和 JS 声明切分。字符串内的注释/分隔符保留，转义由库识别，
注释不再被错误地替换为空格。默认 1 MiB 输入和库自身嵌套限制保留。
cssText 先解析后替换，语法/预算失败时保留旧值。EOF 未闭合注释/规则按库规则恢复；
未知属性、嵌套规则、at-rule、函数值和 !important 等仍受当前严格属性子集约束。
选择器匹配、级联、完整 CSSOM 以及颜色属性支持仍待成熟样式引擎迁移。
cssparser 与 dtoa-short 的 MPL-2.0 许可、来源与原始源码归档已随依赖记录打包。

CSS 迁移验证：全功能 186 项、默认构建 122 项测试（含文档测试）及 Clippy 通过；23 张既有预览经 Pillow 解码后 RGBA 完全一致。
