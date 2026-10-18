# 发布 rwe 0.1.0

本目录是实际 crate 源码，已包含软件绘制功能，不是空占位包。

## 本机工具路径

此机器的 Rust 工具位于 `/Users/wanglina/.cargo/bin`，如终端找不到 Cargo，先执行：

```sh
export PATH="$HOME/.cargo/bin:$PATH"
```

## 验证和发布

在本目录打开终端执行：

```sh
cargo fmt --check
cargo test --all-targets
cargo test --doc
cargo clippy --all-targets -- -D warnings
cargo publish --dry-run
```

登录 crates.io，在账户设置中创建具有发布权限且允许新建 `rwe` 的 API Token，然后在本机执行 `cargo login` 并按提示输入。不要将 Token 粘贴到聊天、源码或截图中。

```sh
cargo login
cargo publish
```

只有实际发布成功，才取得该名称。dry-run 不会占用名称；发布前名称仍可能被他人取得。发布 `rwe` 不会预留 `rwe-*` 子包。若发布成功，不要重复发布同一版本。

许可证的版权主体目前为 `rwe contributors`。尚未绑定远程仓库，未虚构 repository/homepage 地址。当前打包可能提示缺少相关链接，不影响本地打包验证。

此发布指南不包含在 crates.io 源码包内。所有首次发布后的状态以 crates.io 实际记录为准。
