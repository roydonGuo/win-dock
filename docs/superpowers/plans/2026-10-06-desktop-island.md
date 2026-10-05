# Desktop Island Implementation Plan

**Goal:** 实现分享对话的 Windows 信息岛与 Dock V1。
**Architecture:** Vue 负责视图与交互，Rust 是系统状态唯一来源；设置窗口独立，条件恢复保护系统状态。
**Tech Stack:** Tauri 2, Vue 3, TypeScript, shadcn-vue, Tailwind 4, Win32.

- [x] 配置 Vite/Tauri/权限，安装依赖并生成 shadcn-vue 基础组件。
- [x] src/domain/apps.test.ts 先验证固定/运行合并、排序、搜索，再实现 apps.ts。
- [x] src-tauri/src/policy.rs 为 Win 按键状态机和恢复判断写测试并实现。
- [x] src-tauri/src/windows.rs 实现只读枚举、图标、激活、启动、显示器、材质和 hit testing。
- [x] src-tauri/src/recovery.rs 实现事务日志与独立恢复进程。
- [x] src-tauri/src/lib.rs 实现命令、事件、配置、托盘、快捷键与窗口状态。
- [x] src/composables 与 services 对接 native 和显式浏览器预览；组件实现信息岛、Dock、多窗口和设置。
- [x] 运行 npm test/build、cargo test/check/build，检查浏览器交互与原生只读 smoke test，修复错误，完善 README。
