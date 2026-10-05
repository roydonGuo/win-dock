# Desktop Island V1

依据用户提供并要求实现的分享对话：Windows 10/11，Tauri 2 + Vue 3 + shadcn-vue + Tailwind CSS。

## 产品行为
底部居中的无边框透明信息岛显示日期、时间和可配置城市天气。点击/快捷键展开固定应用与运行应用，支持搜索、拖拽排序、右键、启动、切换和多窗口列表。Esc 收起，点击设置打开独立窗口。悬停微展开，支持离开后收起和全屏避让。

## 架构
Vue Composition API 和 TypeScript。App 为组合入口；IslandView 管理视图，DockApp 展示单个应用，SettingsView 配置，useDesktopState 对接 native 服务。shadcn-vue Button/Input/Switch/Select 组成设置中心。
Rust 管理应用窗口快照、WinEvent hook、应用标识、图标缓存、启动和窗口操作、显示器与 DPI、系统显隐、配置持久化、托盘与恢复。事件发布完整快照并定期校准，避免前端轮询系统。

## 系统控制
默认不修改任务栏或桌面图标。设置支持跟随系统/自动隐藏/隐藏；修改前将原始和已施加状态持久化。正常退出恢复，watchdog 在父进程异常终止后恢复。恢复采用条件比较，避免覆盖用户后续变更。Explorer 重启后重新获取窗口。
双击 Win 为可选功能，不吞掉 Win 组合键；保留系统行为意味着第一次 Win 可能打开开始菜单，UI 明示该限制。默认 Ctrl+Alt+Space，快捷键可配置并验证冲突。

## 窗口与材质
信息岛窗口保持底部锚点，原生窗口大小随状态变化；WebView 内使用高光、半透明、噪声和弹性动画，Windows 原生 blur 材质补充跨窗口模糊。透明留白采用 native hit testing。设置窗口独立。显示器按用户选择，DPI/显示变更重新定位。

## 验证
应用合并/排序、全屏避让、双击 Win 检测、条件恢复写纯逻辑测试；运行 vue-tsc/Vite build、cargo test/check/build。原生行为 smoke test 不自动改变系统偏好，使用只读窗口和监视器检查验证。

## 范围
交付对话 V1.0 核心；V1.5 的音乐、下载、USB 事件和 V2 插件/AI 不属于当前范围。天气真实请求失败时展示不可用，不伪造数据。
