# 标记与批量操作

已标记的文件（标有 `■`）是批量操作的对象：操作作用于 **当前列表中** 已标记的文件（当前目录，或已打开的 Branch 或 Showall）。

## 标记文件

| 按键 | 目录树 | 文件窗口 |
|---|---|---|
| `T` / `U` | 标记 / 取消标记该目录中的文件 | 标记 / 取消标记当前文件并下移 |
| Ctrl+T / Ctrl+U | 标记 / 取消标记所有已载入且匹配 Filespec 的文件 | 标记 / 取消标记整个列表 |
| Ctrl+S | 只显示已标记文件的 Showall | |
| `I` | 反选该目录的标记 | 反选列表的标记 |
| Ctrl+F7 / F8 / F9 | 标记 / 取消标记 / 反选该分支中已载入的文件 | |

## 对已标记文件的操作

| 按键 | 功能 |
|---|---|
| Ctrl+C / Ctrl+M | 将它们 [复制 / 移动](copy-move.md) 到目标位置 |
| Alt+C / Alt+M | 复制 / 移动它们，并 **重建其目录路径** |
| Ctrl+D | [删除](delete.md) 它们 |
| Ctrl+A | 修改它们的 [权限](attributes.md) |
| Ctrl+N | 修改它们的 [日期](attributes.md) |
| Ctrl+R | 使用掩码批量 [重命名](rename.md) |
| Ctrl+B | [批处理](execute.md)：生成每个文件占一行的脚本或列表 |
| Ctrl+F5 | 将它们打包到 [归档文件](archives.md) 中 |
| Ctrl+S / Ctrl+V | [在其中搜索文本 / 逐个查看](search.md) |
| Ctrl+O | 用默认程序打开它们 |
| Ctrl+Y | 将它们的路径复制到剪贴板 |
| Alt+O | 将它们输入到 [内置终端](terminal.md) |

在 eXecute（执行）中，`%t` 代表所有已标记的文件（见 [执行](execute.md)）。

## Ctrl 与 Alt 的含义

与 ZTreeWin 相同：

- **单独的字母**作用于光标下的项目。
- **Ctrl+字母**作用于已标记的文件：Ctrl+C 复制、Ctrl+D 删除、Ctrl+T 全部标记。
- **Alt+字母**是命令的另一种版本：在目录树中作用于整个分支（Alt+P Prune、Alt+R Release、Alt+U 磁盘用量、Alt+D 重复文件、Alt+C Compare）；在文件窗口中作用于带路径的已标记文件（Alt+C、Alt+M）；以及各种工具（Alt+S Sort、Alt+F File display、Alt+B 书签、Alt+H 帮助、Alt+I Info、Alt+X 终端）。

**0.8 版本的变化。** Alt 不再重复 Ctrl 的功能，部分按键已更改：

| 按键 | 0.7 及之前 | 0.8 起 |
|---|---|---|
| Alt+A | 修改已标记文件的权限 | 无（用 Ctrl+A） |
| Alt+D | 删除已标记的文件 | 重复文件（删除：Ctrl+D） |
| Alt+R | 重命名已标记的文件 | 释放分支 Release，同 `-`（重命名：Ctrl+R） |
| Alt+T | 全部标记 | 无（用 Ctrl+T） |
| Alt+U | 全部取消标记 | 磁盘用量（取消标记：Ctrl+U） |
| Alt+W | 磁盘用量 | 无（用 Alt+U） |
| Alt+Y | 重复文件 | 无（用 Alt+D） |

旧按键可以在[配置](configuration.md)的 `[keys.any]` 中恢复，例如 `"alt+w" = "usage"`。

## Ctrl 与 Alt 菜单 (F9)

在普通屏幕上，底部菜单共三行：前两行按字母顺序列出字母命令，第三行列出功能键和符号。当屏幕宽到足以容纳所有菜单时，只用两行：一行字母命令，一行按键。每个命令都出现在某个菜单中（DIR 或 FILE、CTRL、ALT）；在压缩包内或远程机器上，菜单会省略在那里不可用的命令。

按 **F9** 会依次切换到 Ctrl 菜单、Alt 菜单，再回到普通菜单。当其中一个菜单打开时，接下来按下的字母将视为 Ctrl+字母 或 Alt+字母（如 `F9` `M` = Ctrl+M）；按 `Esc` 取消。在支持 kitty 键盘协议的终端中，只需 **按住 Ctrl 或 Alt** 即可直接显示对应菜单。

松开按键后会回到平时的菜单，与 ZTree 一样。

终端必须单独报告 Ctrl 和 Alt 的按下与松开，而只有 kitty 键盘协议能做到：kitty、Ghostty、WezTerm、foot（Wayland）和新版 Alacritty 都支持它，在 Ubuntu 或 Debian 上大多只需安装一个软件包（`sudo apt install kitty`，或 `ghostty`、`alacritty`、`foot`）。基于 VTE 的终端（GNOME Terminal、Ptyxis、Terminator、Tilix）尚不支持，因此在这些终端中菜单仍使用 F9。

**Ctrl+M：** 在大多数终端中，Ctrl+M 发送的信号与 Enter 相同。当终端支持 kitty 键盘协议时（如 kitty、WezTerm、foot、Ghostty、新版 Alacritty、iTerm2），Xylem 会启用该协议，此时 Ctrl+M 可像在 ZTree 中一样移动已标记文件。在基于 VTE 的终端（GNOME Terminal、Ptyxis、Terminator、Tilix）中，请使用 **F9 M**（或按 Alt+M 后在提示行中按 F6）。
