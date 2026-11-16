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

- 单独的字母作用于光标下的项目。
- Ctrl+字母作用于已标记的文件：Ctrl+C 复制它们，Ctrl+D 删除它们。
- Alt+字母是命令的另一种变体：在目录树中作用于整个分支，在文件窗口中作用于已标记的文件并保留其路径。

[菜单](menus.md)说明了这条规则，以及如何用 F9 显示 Ctrl 和 Alt 菜单。[Ctrl 菜单](commands-ctrl.md)和 [Alt 菜单](commands-alt.md)列出了全部命令。

**0.8 版本变更。** Alt 不再重复 Ctrl 的功能：Alt+A、Alt+D、Alt+R、Alt+T、Alt+U、Alt+W 和 Alt+Y 有所变动。[版本说明](releases.md)中记录了每个按键的原有含义与新含义，以及用于恢复旧按键的 `config.toml` 配置行。
