# 终端与按键

Xylem 接收终端发送的按键序列。某些终端会截留某些按键供自身使用，或者对两个不同的按键发送相同的字节序列。本页列出了这些情况及替代按键。

## 终端或桌面环境截留的按键

| 按键 | 截留方 | 替代操作 |
|---|---|---|
| F1 | GNOME Terminal（打开其自身的帮助） | Alt+H |
| F11 | GNOME Terminal（全屏切换） | Alt+B，或按 `'` 跳转到书签 |
| Alt+F10 | GNOME（最大化窗口） | F12 |
| Alt+T | 带有“Terminal”菜单的终端 | 在 `[keys.any]` 中绑定其他按键 |

## Ctrl+M

大多数终端为 Ctrl+M 和 Enter 发送相同的字节，此时 Xylem 接收到的实际上是 Enter。若要移动已标记的文件，请先按 `F4` 再按 `M`，或按 Alt+M 并在提示行中按 F6 关闭路径保留。

在支持 kitty 键盘协议的终端中，这两个按键可以被明确区分，Ctrl+M 可直接移动已标记的文件。

## kitty 键盘协议

借助该协议，终端能够单独报告 Ctrl 和 Alt 的按下与释放。当终端响应支持该协议时，Xylem 会自动启用它。

| 支持协议 | 不支持协议 |
|---|---|
| 按住 Ctrl 或 Alt 显示 CTRL 或 ALT 菜单 | 按 F4 切换显示 |
| Ctrl+M 为 Move（移动） | Ctrl+M 等同于 Enter |

| 终端 | 协议支持 |
|---|---|
| kitty, Ghostty, WezTerm, foot (Wayland), 新版 Alacritty | 是 |
| iTerm2 | Ctrl+M 为 Move；按住 Ctrl 或 Alt 显示菜单尚未验证 |
| GNOME Terminal, Ptyxis, Terminator, Tilix（基于 VTE） | 否 |

## 剪贴板

`Y` 与 Ctrl+Y 将内容写入剪贴板。通过 SSH 或在 tmux 内部时，Xylem 使用 OSC 52 转义序列（需要终端支持并允许接收）。在 X11 和 Wayland 上，退出 Xylem 后文本仍保留在剪贴板中。

## 图标与制表线

图标功能（F12 第 2 页）需要包含 Nerd Fonts 字形的字体。若字体未包含，终端会将每个图标显示为空白方框。如果开启图标后分栏发生对齐错位，说明终端将“不确定宽度 (ambiguous width)”字符渲染为了全角宽字符；请将终端的该选项设置为窄字符（半角），或直接关闭图标功能。

## 屏幕尺寸适应

| 宽度或高度 | 表现 |
|---|---|
| 低于 72 列 | 顶部标题行省略程序名称与版本 |
| 无法容纳整行菜单 | 菜单中的词语等长缩写，末尾条目替换为 `…` |
| 低于 58 列或 13 行 | 不显示欢迎界面 |

另请参阅：[菜单](menus.md)、[配置](configuration.md)、[内置终端](terminal.md)。
