# DIR 和 FILE 菜单

单独按下字母键会在光标所在的项目上执行对应命令：目录树中的目录（DIR 菜单）或文件窗口中的文件（FILE 菜单）。

某些字母在不同窗口中对应不同的命令。`C` 在目录树中为比较 (Compare)，在文件窗口中为复制 (Copy)；`M` 在目录树中为新建目录 (Makedir)，在文件窗口中为移动文件 (Move)。

## 命令列表

按菜单中的排列顺序如下。“目录树”与“文件”标明按键生效的窗口。

| 按键 | 命令 | 目录树 | 文件 | 功能说明 |
|---|---|---|---|---|
| `A` | Available | 是 | | 列出已挂载的文件系统及其总空间、已用空间和可用空间。[信息](info.md) |
| `A` | Attributes | | 是 | 以八进制修改文件权限。[权限与日期](attributes.md) |
| `B` | Branch | 是 | 是 | 列出该目录及其下方所有内容中的文件，若尚未载入则先载入该分支。在 Branch 或 Showall 视图中按 B 则恢复为仅显示当前目录的文件。 |
| `C` | Compare | 是 | | 在分屏模式下，将当前目录与另一窗口进行比较，并按选定方式标记存在差异的文件。[比较](compare.md) |
| `C` | Copy | | 是 | 复制文件。提示框询问目标位置。[复制与移动](copy-move.md) |
| `D` | Delete | 是 | 是 | 询问后删除文件。在目录树中，若目录为空则将其删除。[删除](delete.md) |
| `E` | Edit | | 是 | 在编辑器中打开文件。[编辑](execute.md) |
| `F` | Filespec | 是 | 是 | 设置窗口显示哪些文件，按名称、日期和大小筛选。[Filespec](filespec.md) |
| `I` | Invert | 是 | 是 | 反选标记状态：目录树中反选当前目录下的文件，文件窗口中反选当前列表中的文件。 |
| `L` | Log | 是 | | 打开另一个目录或远程机器，作为当前窗口的目录树根节点。[Treespec 与 Log](treespec.md) |
| `M` | Makedir | 是 | | 在光标所在目录下创建子目录。 |
| `M` | Move | | 是 | 移动文件。[复制与移动](copy-move.md) |
| `N` | New date | 是 | 是 | 修改文件或目录的修改日期。[权限与日期](attributes.md) |
| `O` | Oops | 是 | | 从回收站恢复最近删除的内容。在文件窗口中对应 Ctrl+Z。[删除](delete.md) |
| `O` | Open | | 是 | 用系统默认程序打开文件。[文件窗口](files.md) |
| `P` | Phrase | 是 | 是 | 打开搜索框：列出包含所输入词语的已载入文件。[搜索框](filespec.md#搜索框-p) |
| `R` | Rename | 是 | 是 | 重命名文件或目录。[重命名](rename.md) |
| `S` | Showall | 是 | 是 | 列出当前窗口中所有已载入的文件。 |
| `T` | Tag | 是 | 是 | 文件窗口：标记文件并将光标下移。目录树：标记当前目录中的所有文件。[标记](tagging.md) |
| `U` | Untag | 是 | 是 | 文件窗口：取消标记文件并将光标下移。目录树：取消标记当前目录中的所有文件。 |
| `V` | View | | 是 | 在查看器中打开文件：支持文本、十六进制或表格。[查看器](viewer.md) |
| `W` | Where | 是 | 是 | 在列表中仅保留符合 SQL 条件的文件。[SQL 条件](filespec.md#sql-条件-w) |
| `X` | eXecute | 是 | 是 | 运行 shell 命令。`%f` 代表文件，`%t` 代表已标记的文件。[执行](execute.md) |
| `Y` | Yank | 是 | 是 | 将文件或目录的路径复制到剪贴板。 |
| `Q` | Quit | 是 | 是 | 询问后退出。[退出](quit.md) |

菜单最后一行中的按键（F1、`*`、`?`、`\`、F3、Enter、F5、F7、F8、F9）参见[功能键与符号](commands-keys.md)。

## 未在菜单中列出的等效按键

这些是表中命令的辅助按键。

| 按键 | 等同于 |
|---|---|
| Del | `D` |
| Shift+Del | `D`，直接永久删除而不放入回收站 |
| F10 | `Q` |
| Ctrl+Insert | `Y` |
| Ctrl+Z | `O` (Oops)，在两个窗口中均有效 |

## 在压缩包内或远程机器上

菜单标题为 ARCHIVE 或 REMOTE，其中无法运行的命令将不会列出。

| 环境 | 未列出的命令 |
|---|---|
| 压缩包 | Delete、Edit、Log、Makedir、Move、New date、Oops、Rename、Attributes、eXecute。Copy 执行解压操作。 |
| 远程机器 | Edit、Oops、eXecute。远程机器上没有回收站。 |

参见[归档文件](archives.md)与[远程主机](remote.md)。

另请参阅：[Ctrl 菜单](commands-ctrl.md)、[Alt 菜单](commands-alt.md)、[菜单](menus.md)。
