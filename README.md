# Desktop Island

Windows 底部信息岛与展开式 Dock。Tauri 2 + Vue 3 + TypeScript + shadcn-vue + Tailwind CSS 4。

## 开发

需要 Node.js 20.19+、Rust 1.94+、MSVC C++ 构建工具、Windows SDK 和 WebView2 Runtime。

```powershell
npm install
npm run tauri dev
```

仅预览界面：`npm run dev`，打开 http://localhost:1420。预览不伪造运行应用、系统操作或天气；设置仅存于浏览器。

## 使用

- 默认底部信息岛显示时间和日期；点击或 Ctrl+Alt+Space 展开 Dock。
- 添加 `.exe`、`.lnk`、`.url`，也可拖入 Dock；固定应用可拖动排序。
- 点击运行应用切换窗口；当前活动的单窗口应用再次点击最小化；多个窗口显示选择列表。
- 右键应用可固定、取消固定、新开应用；列表支持最小化和关闭窗口。
- 搜索应用名称或窗口标题；Enter 打开首项，Esc 返回。
- 设置中心提供主题、边距、收起、全屏避让、快捷键、显示器、城市天气、自启与系统恢复。
- 任务栏默认跟随系统，桌面图标默认保持原有状态。切换图标开关后可显式显示或隐藏，也可返回跟随系统；退出时条件恢复修改前状态。
- 托盘左键展开/收起；右键可打开设置、恢复系统、恢复并退出。

## 构建与验证

```powershell
npm test
npm run build
cargo test --manifest-path src-tauri/Cargo.toml
npm run tauri build
```

安装包位于 `src-tauri/target/release/bundle/nsis/`。

## 实现边界

- 双击 Win 是观察式实验功能，不拦截系统组合键，所以第一次按 Win 可能打开开始菜单。独立快捷键是稳定入口。
- 任务栏完全隐藏和桌面图标显隐依赖 Explorer 窗口，Windows 更新、Explorer 重启和第三方 Shell 可能影响兼容性。任务栏自动隐藏使用官方 AppBar 接口。
- 应用身份优先采用窗口/进程/快捷方式 AppUserModelID，否则采用规范化 EXE 路径；固定 EXE 通过目标路径关联运行窗口，具有独立 AUMID 的快捷方式保持区分。打包应用使用 AppsFolder 激活。
- 不枚举后台服务，仅展示可切换的可见顶层窗口；更高权限的应用可能无法读取路径或切换。
- 全屏避让按选定屏幕上的前台窗口边界判断。系统虚拟桌面 cloaked 窗口会过滤。
- 原生 blur 与 WebView 高光/半透明层组合模拟玻璃；不宣称实现光学折射。
- 天气默认关闭，手动选择城市后访问 Open-Meteo。需要网络；失败显示不可用。
- 本版单个信息岛可选择显示器，支持 DPI 与显示器变化；不创建每屏独立实例。
- 音乐、下载、设备动态事件、Workspace、插件、AI 属于分享对话后续版本，不包含在 V1。

## 状态与恢复

配置存放在 `%APPDATA%/DesktopIsland/config.json`。系统更改前写入 `recovery-<pid>.json`，独立 `--recovery-watchdog` 进程等待父进程结束后执行条件恢复。正常退出也恢复。若用户已改变被管理的值，恢复不会覆盖该新状态；未完成的恢复日志保留供下次启动重试。

源码：`src/components/island` 信息岛、`src/components/settings` 设置，`src/composables` 状态，`src-tauri/src/windows.rs` Windows 适配、`events.rs` Hook、`recovery.rs` 恢复、`lib.rs` 命令与应用生命周期。
