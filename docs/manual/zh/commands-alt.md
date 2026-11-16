# Alt 菜单

Alt 配合字母是命令的另一种变体。在目录树中，它覆盖光标下的整个分支。在文件窗口中，Alt+C 和 Alt+M 作用于已标记的文件并保留其路径。其他条目则打开不针对单个文件的工具。

若要显示此菜单，可按住 Alt 或按两次 F9。参见[菜单](menus.md)。

## 命令列表

按菜单中的排列顺序如下。“目录树”与“文件”标明按键生效的窗口。

| 按键 | 命令 | 目录树 | 文件 | 功能说明 |
|---|---|---|---|---|
| Alt+B | Bookmarks | 是 | 是 | 打开书签管理器：跳转到已保存的目录、命名或删除书签。[书签](bookmarks.md) |
| Alt+C | Compare | 是 | | 按相对路径将整个分支与另一个窗口进行比较，并打开带有标记差异文件的 Branch 视图。[比较](compare.md) |
| Alt+C | Copy | | 是 | 复制已标记的文件并在目标目录下重建每个文件的相对路径。在提示行中按 F7 可设置路径的起始位置。[复制与移动](copy-move.md) |
| Alt+D | Duplicates | 是 | 是 | 在分支中查找内容完全相同的文件和文件夹。未经确认绝不删除任何内容。[重复文件](duplicates.md) |
| Alt+F | File display | 是 | 是 | 更改文件窗口每行显示的内容：分栏名称、大小、详细信息、所有者、类型、长文件名。[文件窗口](files.md) |
| Alt+H | Help | 是 | 是 | 打开帮助，等同于 F1。 |
| Alt+I | Info | 是 | 是 | 显示文件或目录的类型、大小、权限、所有者及日期。[信息](info.md) |
| Alt+K | Kbyte | 是 | | 在目录树每个目录旁显示大小、文件数或百分比。[信息](info.md) |
| Alt+L | Log | 是 | 是 | 将光标下的目录设为目录树的根节点。之前已载入的内容和标记将被放弃。[Treespec 与 Log](treespec.md) |
| Alt+M | Move | | 是 | 移动已标记的文件并在目标目录下重建每个文件的相对路径。[复制与移动](copy-move.md) |
| Alt+P | Prune | 是 | | 在键入确认后永久删除目录及其包含的一切。[删除](delete.md) |
| Alt+Q | Quit here | 是 | 是 | 退出并将 shell 保留在当前目录中。需要安装[退出](quit.md)中的 shell 函数。 |
| Alt+R | Release | 是 | | 从内存中释放分支，等同于 `-`。其中的标记将丢失。 |
| Alt+S | Sort | 是 | 是 | 按名称、扩展名、日期或大小对文件窗口排序。[排序](sort.md) |
| Alt+U | Usage | 是 | 是 | 按占用空间从大到小列出分支中的文件夹和文件。[磁盘用量](usage.md) |
| Alt+N | iNdex | 是 | 是 | 为分支中已载入文件的词语建立索引，以便 SQL 条件中的 `words()` 能即时返回结果。[索引](sql.md#完整单词与索引words) |

## 其他按键

| 按键 | 窗口 | 功能说明 |
|---|---|---|
| Alt+X | 两者 | 打开内置终端，再次按下返回。[内置终端](terminal.md) |
| Alt+Z | 两者 | 全屏打开终端。 |
| Alt+O | 两者 | 将已标记的文件路径带引号输入到终端命令行中。 |
| Alt+F5 | 文件 | 将光标下的归档文件作为目录树打开。[归档文件](archives.md) |
| Alt+F10 | 两者 | 打开配置，等同于 F12。在 GNOME 上，Alt+F10 会最大化窗口而无法传递给 Xylem。[配置](configuration.md) |
| Alt+Shift+Del | 两者 | 永久删除已标记的文件，等同于 Ctrl+Shift+Del。 |

## 在归档文件内或远程主机上

| 环境 | 未列出的命令 |
|---|---|
| 归档文件 | Log、Move、Prune、终端相关按键。Alt+C 按照归档文件内部的路径解压已标记的文件。 |
| 远程主机 | Log (Alt+L)、Alt+O。Alt+X 在该机器上打开 shell。 |

## 0.8 版本变更

在 0.7 及之前版本中，Alt+A、Alt+D、Alt+R、Alt+T 和 Alt+U 与对应字母的 Ctrl 功能相同，Alt+W 为 Usage，Alt+Y 为 Duplicates。
[版本说明](releases.md)中包含了对照表以及用于恢复旧按键的 `config.toml` 配置行。

另请参阅：[DIR 和 FILE 菜单](commands.md)、[Ctrl 菜单](commands-ctrl.md)。
