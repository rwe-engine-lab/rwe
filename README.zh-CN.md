> 已新增实验性静态 HTML 与 Vue/React 交互支持，见文末与 APPLICATIONS.md。

# rwe

实验性纯 Rust 应用框架，包含自研 HTML/CSS 渲染器及可选的 JavaScript 和桌面窗口宿主。

0.1.0 已实现可运行的软件绘制基础：CSS 十六进制颜色解析、RGBA 透明混合、自动裁剪的矩形画布、PPM 示例和回归测试。默认渲染器使用 html5ever、cssparser、Taffy 和 png，项目自身源码禁止 unsafe。可选 macOS 窗口宿主使用经审核的 Rust 依赖。

工作区开发版包含扁平 DOM 和 html5ever HTML5 解析，通过 `cargo test --test html` 自动验证节点树。

开发版现已接通 HTML、内联样式、块布局和背景绘制；现支持简易点阵文字；尚无完整浏览器或 Electron 兼容性；可选 JavaScript 功能见文末（macOS 窗口宿主见下文）。项目处于早期开发阶段，1.0 前 API 可能调整。具体 API、示例和限制见 [English README](README.md)。

## 本地验证

```sh
cargo test --all-targets
cargo test --doc
cargo run -- render examples/static-html/index.html --width 480 --format png > preview.png
cargo run --features window --bin rwe-window -- examples/static-html/index.html --watch
cargo run --features window,javascript --bin rwe-window -- examples/vue-counter/index.html --watch
cargo run --features window,javascript --bin rwe-window -- examples/react-counter/index.html --watch
```

`render` 案例通过框架向标准输出写入 PNG 图片。颜色解析仅接受完整的 `#RGB`、`#RGBA`、`#RRGGBB`、`#RRGGBBAA`，不自动去空格，不支持颜色名称或函数语法。透明混合在存储的通道数值上进行，尚无色彩管理或线性光转换。

## 开发约束

所有模块遵循成熟库优先原则：有适合的优秀库就复用，不以零依赖或全部自研为目标。
框架聚焦应用接口、资源边界、生命周期和薄适配层。全模块结论见 [REUSE-AUDIT.md](REUSE-AUDIT.md)，
后续开发约束见 [AGENTS.md](AGENTS.md)。本轮已采用 Taffy 替换 Flex 求解，png 替换手写 PNG；
HTML/CSS/DOM、文本、绘图等尚待迁移的部分明确列出，不能误称全部完成。
依赖版本与许可见 CORE-DEPENDENCIES.md；其他可选组件仍有各自依赖记录。

下一步开发完整字体与行内排版、扩展 CSS/DOM 支持与应用兼容性。嵌入方应对不可信输入设置画布尺寸限制。

## 许可证

MIT，见 [LICENSE](LICENSE)。


## HTML5 文档与片段解析

HTML 解析已统一接入 html5ever 0.40.1，删除 Rust 手写解析器及 JavaScript 正则片段/实体解析器。
`Document::parse` 使用 div 片段上下文；`parse_page` 自动构建 html/head/body；
`parse_fragment` 接受标签名和命名空间，支持 table、SVG 等上下文。
HTML 省略闭合、重复属性、实体与非法字符引用按 HTML5 规则恢复，不再作为语法错误拒绝。
非空 HTML 标签末尾的斜杠不会将它关闭。

公开节点树仍为扁平索引结构，新增 Comment 节点与 namespace 字段。注释不参与文字汇总和绘制；
template 内容对静态接口映射为模板的子节点，对 JS 则放入 template.content。
初始页面和 innerHTML 使用同一个 Rust 入口；innerHTML 先构建离线片段，解析被拒绝时保留旧内容。

默认限制保持 1 MiB、100,000 输出节点和 256 层元素。解析前检查字节数，构造期间限制临时节点
（含四个设置节点余量）和深度，按 UTF-8 边界分段喂入，失败后停止；输出阶段执行精确结构限制。
限制不是总内存配额。HtmlError 的偏移表示最后处理到的字节位置。
HTML5 解析不代表完整 DOM、CSS、SVG 绘制或浏览器脚本行为已经实现。

## 开发版块布局与背景渲染

运行 `cargo run -- render examples/static-html/index.html --width 480 --format png > preview.png`，将 `examples/static-html/index.html` 渲染为图片。示例现包含实际点阵文字、继承文字样式和换行段落。

`BlockLayout::new(&document, viewport_width)` 读取元素的 `style` 属性，生成可检查的 `boxes()`，再用 `paint(&mut canvas)` 绘制。支持 `width`、`height`（整数 px、0 或 auto）、`margin`、`padding`（1～4 个非负整数 px 或 0，及四边单独属性）、`background-color`（十六进制或 transparent）和 `display`（block 或 none）。重复属性以后者为准，不支持的声明返回错误。

所有元素按垂直块排列，无浏览器默认样式；宽高使用内容盒模型，自动宽度填满父内容区扣除内外边距与边框后的空间，最低为 0，自动高度容纳子元素。外边距不折叠。固定尺寸默认允许子元素溢出，可用 overflow: hidden 裁剪，背景覆盖整个边框盒（包括边框下面），按父先子后顺序透明混合，最终由祖先 overflow 裁剪区域与画布边界共同裁剪。隐藏元素跳过整个子树及其样式验证。

文本现在按下述规则参与布局和绘制；文字颜色、大小、对齐和空白处理继承，暂不支持内联注释、百分比、小数、负边距、未列出的简写或 `!important`。布局使用显式栈，坐标超出非负 i32 范围时返回错误。`boxes()` 包含透明文档根盒；`height()` 是根流高度，不包含子元素溢出。调用者负责画布大小和内存限制。

## 开发版样式表与级联

新增 `Stylesheet::parse(css)` 和 `BlockLayout::with_stylesheet(&document, &stylesheet, width)`，支持独立 CSS 字符串。应用案例使用 `examples/static-html/index.html` 与 `examples/static-html/style.css`，通过公共 Application API 加载内嵌 style 与本地 link 样式表，仍无网络加载。

支持标签、通配符、类、ID、复合选择器（如 `section.card.active`）及逗号列表。标签名忽略 ASCII 大小写；类名、ID 区分大小写，类属性按 ASCII 空白分词匹配。标签名以 ASCII 字母开头；类和 ID 标识符以 ASCII 字母、下划线或连字符开头，后续可含数字。

逐属性按 ID 数、类数、标签数的优先级比较，同级以后出现者为准；选择器列表取匹配成员的最高优先级。内联声明最终覆盖样式表，支持用 auto、0、transparent 等显式重置。文字颜色、大小、对齐和空白处理在布局时继承，不支持 `!important`、兄弟组合符（+、~）、属性选择器、伪类、转义、at-rule 和嵌套规则。

样式表上限为 1 MiB，支持块注释（替换为空格），解析时验证所有规则，包括未匹配规则；声明属性范围与内联样式相同。错误返回 `RenderError`，目前不含 CSS 源码位置。`computed_style` 可查看元素级联后的声明（未声明的文字字段仍为 None，由布局继承）；非元素或无效节点返回 None。

## 开发版点阵文字

新增自研 MIT 授权 5×7 点阵字形，无字体依赖，支持拉丁字母、数字和常用标点；小写沿用大写字形。中文、emoji 等未覆盖的 Unicode 字符显示占位框，空格和 NBSP 留白。尚无完整字体、抗锯齿、复杂文字整形或双向排版。

CSS 新增 `color`（十六进制或 transparent）与 `font-size`（8～64px，须为 8 的整数倍）。颜色和字号从父元素继承，根默认黑色和 16px，支持样式表及内联覆盖；下述 text-align 与 white-space 也继承。其他属性不继承；暂不接受 inherit/initial/unset 关键字。

文字按 6×8 字符单元整数缩放。默认 `white-space: normal` 折叠 ASCII 空白并去除节点首尾空白，优先保留完整单词换行；超长单词按 Unicode 标量拆分，不使用完整 Unicode 断行或字素算法。NBSP 保留在单词内，但超长时仍可能拆分。宽度不足一个字符（包括 0）时，每行仍排一个字符，允许溢出。

`white-space: pre` 保留空格、空行与显式换行，CRLF/CR 统一为 LF，制表符展开至下一个四列位置，不自动换行。空行占一行高度；结尾换行结束前一行而不额外增加虚拟空行，空字符串不占行。不执行 HTML pre 的首换行移除；pre 标签本身无默认样式，须显式声明。

`text-align: left | center | right` 支持每行左对齐、居中和右对齐。以整个字符单元宽度计算，居中偏移向下取整；超出内容区的行从左侧溢出。对齐与空白处理支持继承、样式表及内联覆盖，可通过 left 和 normal 显式重置。

每个文本节点仍独立占据垂直流，不与兄弟节点共享行，br 暂无专门换行语义。不支持 pre-wrap、nowrap、两端对齐、双向排版和 tab-size。最终由画布边界裁剪。

自动高度包含文字行，固定高度允许文字溢出。文字与背景按源码顺序绘制，后续兄弟背景可覆盖前面的溢出文字。`text_runs()` 可检查每行的文本、源节点、矩形及最终继承后的颜色和大小。原有“文本被忽略”的行为已更新。


页面案例现展示按单词换行的段落、居中按钮文字、右对齐页脚，以及保留缩进和换行的代码块。


## 开发版结构选择器

新增后代选择器（`main p`）和子元素选择器（`main > p`），可与标签、类、ID、通配符和逗号列表混用，例如 `#app > section.card p.note`。后代匹配严格祖先，子元素要求直接父元素；文档根不当作元素匹配。`>` 两侧空白可省略，注释由 cssparser 正确处理，不自动产生后代空格；`.a/**/.b` 表示同一元素上的两个类。

整个选择器链的 ID、类和标签数量共同决定优先级，关系符与通配符不增加权重。匹配采用沿祖先路径的迭代动态计算，保留混合链所需的多个祖先候选，不使用递归或指数回溯。每条候选选择器的结构匹配开销最高随祖先深度与复合选择器数量的乘积增长（另加各复合条件比较）；现有 1 MiB 样式表限制不等同于 CPU 配额，调用者应按使用场景控制输入。

页面案例已用结构选择器重写样式，仍保留原来的布局和图像。暂不支持兄弟组合符、属性选择器、伪类与伪元素。


## 开发版命令行渲染工具

```sh
cargo run -- render examples/static-html/index.html --css examples/static-html/style.css --width 480 --output page.ppm
cargo run -- render examples/static-html/index.html --css examples/static-html/style.css --width 480 --height 440 > stdout.ppm
cargo run -- --help
```

新增 `rwe render`，读取一个 UTF-8 HTML 文件和可选的 `--css` 文件。相对路径按当前工作目录解析；带空格路径须加引号，以连字符开头的输入路径可放在 `--` 后。不支持标准输入或 URL；现可加载内嵌 style 和本地 link 样式表。

`--width` 默认 800；`--height` 默认为 auto，按根流高度生成图片，空文档最低 1 像素，不含子元素溢出，可显式指定高度。`--background` 接受 CSS 十六进制颜色或 transparent，默认白色；透明背景仅适用于 PNG，PPM 仍要求不透明背景。每个尺寸须为 1～16384，总像素最多 16,777,216。HTML/CSS 各最多读取 1 MiB，HTML 继续沿用默认节点数和深度限制；这些并非 CPU 配额。

`--output PATH` 创建新的图片文件并拒绝覆盖已有路径；省略或传入 `-` 时向标准输出写二进制图片。解析与布局成功后才创建文件，写入阶段失败可能留下部分新文件。使用 shell 的 `>` 时，覆盖行为由 shell 决定。工具没有强制覆盖选项。

参数、读取、解析、布局及写入失败会返回非零退出码，并将错误写到 stderr；成功渲染不在 stdout 混入状态文字。

画布新增 `Canvas::write_ppm(writer, opaque_background)`：按行输出 P6 RGB，并将透明像素混合到指定背景色。零尺寸或非不透明背景返回 InvalidInput，写入错误向上传递；调用者控制缓冲与 flush。现有两个图片案例已改用该共享编码器。


## 开发版原生 PNG 导出

```sh
cargo run -- render examples/static-html/index.html --css examples/static-html/style.css --width 480 --output page.png
cargo run -- render fragment.html --format png --background transparent > transparent.png
```

`Canvas::write_png(writer)` 委托 image-rs/png 处理压缩、滤波和数据块，保留 RGBA 与透明像素底下的 RGB。
适配层每次缓冲一行；输出字节不再与旧未压缩编码器一致，兼容性验证以解码后像素为准。
不增加色彩管理或动画。零尺寸在输出前拒绝，原始写入错误向上传递。

命令行按 `.png` 扩展名自动选择 PNG（不区分大小写），其他扩展名及 stdout 保留 PPM 默认值。`--format png|ppm` 显式覆盖格式，向标准输出写 PNG 时应指定 `--format png`。PNG 可接受 transparent 或带 alpha 的十六进制背景色；默认背景仍为白色。两种格式共用已有文件保护、输入与画布限制。


## 开发版四边间距与实线边框

`margin` 和 `padding` 支持 1～4 值：全相同、上下/左右、上/左右/下、上/右/下/左；另支持各自的 `-top`、`-right`、`-bottom`、`-left`。简写与单边属性按影响的边分别参与级联，支持选择器优先级与内联覆盖。仍不支持负值、auto、逻辑方向和外边距折叠。

新增 `border-width`（单个非负整数 px 或 0）、`border-style`（solid/none）、`border-color`（十六进制、transparent 或 currentcolor），以及严格按“宽度 样式 颜色”排列的 `border` 简写，例如 `border: 1px solid #cbd5e1`。`border: none` 重置为宽度 0、样式 none、颜色 currentcolor。默认边框宽度为 0，none 样式不占布局空间；currentcolor 在文字样式继承后取当前元素的文字颜色，边框本身不继承。

指定宽高仍是内容尺寸，布局将四边间距和边框计入外盒；先画背景，再画边框，最后画后代。半透明边框四角只混合一次，后代溢出可覆盖父边框。暂不支持四边独立边框、圆角、虚线、阴影或 border-box 尺寸模型。

开发版 API 调整：`BoxStyle.margin` 和 `.padding` 从单个整数改为 `Edges { top, right, bottom, left }`；可用 `Edges::all(n)` 创建等宽四边。`LayoutBox.rect` 现在是含边框的矩形，新增 `content_rect` 表示纯内容区；无边框页面保持原几何行为。


## 开发版溢出裁剪

```sh
cargo run -- render tests/fixtures/rendering/overflow.html --css tests/fixtures/rendering/overflow.css --width 480 --output overflow.png
```

新增 `overflow: visible | hidden`，默认 visible，不继承，支持常规级联和内联覆盖。hidden 将所有后代背景、边框和点阵文字裁剪到容器的内边框边缘，即包含 padding 的区域；容器自身背景和边框不受自身裁剪影响，仍受祖先裁剪约束。嵌套裁剪取交集，后续兄弟元素恢复各自的祖先裁剪范围，空裁剪区域不绘制后代像素。

该属性仅影响绘制，不改变盒与文字尺寸、根流高度、命令行 auto 高度或布局溢出错误。子元素设 visible 无法逃离 hidden 祖先。`LayoutBox.clip_rect` 暴露元素自身的局部裁剪矩形，visible 时为 None，不是与祖先相交后的最终范围。

绘制使用显式裁剪栈，可向不同尺寸画布重复绘制。新增 `Rect::intersection` 和 `Canvas::fill_rect_clipped`，使用扩大后的整数计算边缘，单次画布裁剪不污染后续操作。暂不支持滚动条、滚动、overflow-x/y、auto/scroll/clip、圆角裁剪，以及浏览器的格式化上下文行为。


## 可选 macOS 原生窗口

```sh
cargo run --locked --features window --bin rwe-window
cargo run --locked --features window --bin rwe-window -- examples/static-html/index.html --css examples/static-html/style.css --watch
./scripts/bundle-macos.sh
open target/RWE.app
```

无参数时打开内置演示；文件路径相对当前目录。窗口可调整大小并重新布局，按显示缩放比例呈现点阵图。滚轮、上下键、PageUp/PageDown、Home/End 滚动；R 重新读取 HTML/CSS，失败保留上次成功页面并在标题及终端提示；Escape 或关闭按钮退出。无需持续刷新，空闲时等待系统事件。

仅审核并验证 Apple Silicon macOS。`window` 为可选功能，默认库及命令行导出保持零依赖；未使用 WebView。winit 负责系统窗口与事件，softbuffer 将 RWE 画布呈现到窗口。完整依赖版本、语言、许可证与平台边界见 [WINDOW-DEPENDENCIES.md](WINDOW-DEPENDENCIES.md)。项目自身仍禁止 unsafe；平台依赖内部使用 FFI。

HTML/CSS 各最多 1 MiB。逻辑页面和物理窗口缓冲分别限制每边 16384、总计 16777216 像素。整页缓存后滚动，超过范围会报错；可滚动高度使用根流高度，不包含固定盒的可见溢出。视口高度不足时可滚动，无可视滚动条。使用 `--watch` 或按 W 启用自动重载，每 500ms 读取并比较受大小限制的文件内容，文件无变化时不重新绘制。读取或解析失败时保留原页面，直到下次修改或按 R 重试。可选 JavaScript 功能支持点击事件；仍无完整输入事件、链接导航、文字选择或网络加载。此窗口呈现当前支持的 HTML/CSS 能力，CSS 仍为严格子集。

本地打包脚本生成开发版、未签名的 `.app`，不用于公证或发布。支持单独指定输出目录。验证：`cargo test --all-targets --features window`、`cargo test --doc` 和 `cargo clippy --all-targets --features window -- -D warnings`。


窗口现在支持将本地 `.html`/`.htm` 文件拖入打开，再拖入 `.css` 文件关联样式；新 HTML 会清除上一个文件的 CSS，成功打开后滚动到顶部并自动启用重载。CSS 仅在已有本地 HTML 时可拖入，不自动查找同名文件。任何失败都保留原路径、样式和页面。标题显示当前文件、自动重载状态及错误详情。布局超限时保留上次画面，缩小窗口或修复文件后可恢复；物理显示缓冲超限时等待缩小窗口，不再直接退出。

## 静态 HTML、Vue 与 React（实验性）

```sh
cargo run --locked --features window,javascript --bin rwe-window -- examples/static-html/index.html --watch
cargo run --locked --features window,javascript --bin rwe-window -- examples/vue-counter/index.html --watch
cargo run --locked --features window,javascript --bin rwe-window -- examples/vue-template/index.html --watch
cargo run --locked --features window,javascript --bin rwe-window -- examples/react-counter/index.html --watch
```

`Document::parse_page` 与 `LocalPage::load` 现在使用 html5ever 的完整文档恢复解析；
内嵌 CSS 和本地 link 样式仍按文档顺序加载。`Document::parse` 改为 HTML5 片段解析，
不再保留旧的严格语法拒绝行为。静态 render 仍不执行脚本。

可选 JavaScript 功能通过 Rust Boa 0.22.0 引擎和 RWE 自己的 DOM、事件、布局及画布运行应用，不使用 WebView。JavaScript 构建需要 Rust 1.91+，默认渲染器保持 Rust 1.85 要求，使用 html5ever、cssparser、Taffy 和 png。示例使用未经修改的 Vue 3.5.13 与 React/ReactDOM 18.3.1 官方生产包，运行时无 CDN。已验证 Vue 渲染函数、Vue 模板与列表、React useState、点击冒泡与捕获、Promise 更新、延迟定时器及更新后的实际像素。

窗口自动重载覆盖 HTML 引用的本地 CSS/JS。文件加载失败保留原页面；重载会重建 JavaScript 运行时并重置组件状态。本地资源引用不能逃出 HTML 目录，不支持 URL、查询参数或片段。源码与合并 CSS 各限 1 MiB，页面总资源 8 MiB，外部引用最多 32 个。

目前仅限经典脚本/IIFE/UMD 应用和有限 DOM/CSS 子集，不能直接运行任意现成 Vue/React 工程。`.vue`、JSX/TSX 需要应用侧预编译；ES 模块、Vite 开发服务器、动态 import、fetch、完整路由、表单/键盘事件、完整字体、图片及 flex/grid 仍不支持。详细范围与运行说明见 [APPLICATIONS.md](APPLICATIONS.md)，依赖审核见 [JS-DEPENDENCIES.md](JS-DEPENDENCIES.md)。


## 框架应用入口

案例不再自行拼接运行时、DOM 快照、布局、画布和图片写入流程。桌面应用通过公共 API 启动：

```rust
fn main() -> Result<(), String> {
    rwe::DesktopApp::new("index.html").watch(true).run()
}
```

无窗口验证使用 `Application::load`、`click`、`render`、`save_png`。窗口事件循环和图片导出都复用同一个 Application 实现，CLI 静态导出使用 `Application::load_static`。独立应用案例通过 main.rs 调用框架的 DesktopApp 启动窗口，HTML/CSS/JavaScript 分别定义应用内容；验证工具见 `tools/app-snapshots.rs`，案例说明见 `examples/README.md`。


案例与验证按职责分开：`examples/` 保留每个应用的薄启动入口和应用资源；解析、像素绘制及溢出裁剪的回归断言在 `tests/`，输入样本在 `tests/fixtures/`；批量截图工具为 `tools/app-snapshots.rs`，通过 `cargo run --features javascript --bin rwe-app-snapshots -- 输出目录` 调用。

## 项目职责划分

- `src/`：可复用框架实现及 Application、DesktopApp 公共 API。
- `examples/`：按应用分目录，包含薄启动入口 main.rs 及 HTML/CSS/JavaScript 内容。
- `tests/`：自动断言；`tests/fixtures/` 存放确定性的回归输入。
- `tools/`：批量截图等开发工具。
- `scripts/`：本地开发打包。

案例入口和命令见 [examples/README.md](examples/README.md)，测试说明见 [tests/README.md](tests/README.md)，开发工具见 [tools/README.md](tools/README.md)。


独立应用目录现为 `examples/static-html/`、`examples/vue-counter/`、`examples/vue-template/`、`examples/react-counter/`。例如运行 `cargo run --features window,javascript --example vue-counter`。每个 `main.rs` 仅选择 index.html 并调用 DesktopApp，窗口创建与渲染循环仍由框架实现。


## 应用窗口配置

```rust
rwe::DesktopApp::new(entry)
    .title("Vue Counter")
    .size(960, 640)
    .min_size(320, 240)
    .background(rwe::Color::rgba(237, 244, 239, 255))
    .resizable(true)
    .watch(true)
    .run()
```

窗口配置仍由框架统一处理，案例只声明选项。也可用 `window_options(WindowOptions { .. })` 批量配置；`options()` 可在启动前读取配置。尺寸为逻辑像素，最小尺寸不能超过初始尺寸；标题非空、不含控制字符且最多 256 字符；背景必须不透明。创建窗口和读取页面前统一校验尺寸、像素预算及其它配置。

通用启动器新增 `--title`、`--width`、`--height`、`--min-width`、`--min-height`、`--background`、`--fixed-size`，与 Rust API 共用验证和渲染路径。背景在缩放、重载、换页、脚本更新后保持，CSS 背景覆盖其上。窗口失焦时取消尚未完成的点击。无窗口导出仍使用 RenderOptions 的背景设置。

## 单行文本输入

运行 `cargo run --features window,javascript --example text-input` 验证资源内嵌的输入案例。
框架统一实现 value/placeholder 绘制、光标、选区替换、删除与左右/Home/End 移动、
Shift 扩展选区、Control/Command+A、只读/禁用/maxlength 以及 beforeinput/input/change。
Vue 模板 v-model 和 React 受控 value/onChange 已使用官方运行库验证。
案例 main.rs 只配置窗口和资源，app.js 只实现问候语和清空业务。

`Application::insert_text` 支持其他宿主和无窗口验证；`KeyEvent.text` 可传入原生按键文本。
剪贴板、撤销、鼠标定位选区、多行 textarea、其它输入类型与
表单提交尚未实现。点击输入框目前将光标置于末尾；Unicode 按码点编辑，点阵字体字形限制仍适用。
详见 [APPLICATIONS.md](APPLICATIONS.md#single-line-text-input)。

## 实验性输入法组合

`ImeEvent::Preedit/Commit/Cancel` 已接入公共应用接口与 macOS 宿主，支持预编辑下划线、
UTF-8 光标范围转换、组合事件、正式提交、取消及候选框定位。预编辑不修改已提交 value，
Vue v-model 和 React 受控表单已用中文提交验证。`text-input` 案例显示组合状态。
原生系统输入法和候选窗口尚需手动验证；默认点阵字体缺少中文字形；启用 fonts 可使用下述内嵌中文回退字体。
详见 [输入法协议与限制](APPLICATIONS.md#input-method-composition)。

## 可选中文字体

```sh
cargo run --features window,javascript,fonts --example chinese-input
```

`fonts` 功能嵌入已记录来源与 OFL 授权的中文字体子集，使用经审核的纯 Rust 字体栅格化依赖。
默认构建仍零依赖，ASCII 点阵输出保持原样；常用简繁中文、扩展 A 和中文标点按实际字宽绘制。
换行/对齐、文本框水平滚动、选区、光标及输入法候选位置共用字宽计算。字体不依赖系统或网络。

中文案例只包含薄窗口入口及问候语业务。字体约增加 9.8 MB 的未压缩数据；依赖与字体授权见
[FONT-DEPENDENCIES.md](FONT-DEPENDENCIES.md) 和 [字体来源](src/assets/fonts/README.md)。
目前仍没有 CSS 字体族/字重选择、复杂文字整形、完整 Unicode 断行和 emoji；原生输入法候选窗口仍待手动验证。

设置表单案例：`cargo run --features window,javascript,fonts --example settings-form`。
能力范围与下一步：[兼容性清单](COMPATIBILITY.md)。

布局进展：支持显式像素基准的单行横向 Flex、gap、整数 flex-grow、
主轴分布和交叉轴对齐；范围及报错边界见 APPLICATIONS.md 的 Flex 章节。

CSS 语法已统一接入 cssparser 0.38.0：样式表、内联声明与 JS style.cssText 共用
同一原生解析入口，删除手工分号/冒号/大括号切分。字符串中的分隔符不会拆坏声明。
语法或输入限制失败时 cssText 保留旧值；EOF 注释/规则恢复遵循库行为。
属性、选择器、级联和完整 CSSOM 尚未完成成熟引擎迁移；MPL-2.0 依赖记录见 CORE-DEPENDENCIES.md。
