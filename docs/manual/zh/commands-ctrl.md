# Ctrl 菜单

Ctrl 配合字母会在已标记的文件上执行命令。在文件窗口中，这些文件是当前列表中的已标记文件（当前目录，或打开的 Branch 或 Showall 视图）。在目录树中，则是光标所在目录中的已标记文件。标有“所有已载入文件”的行属于例外情况。

如果没有标记任何文件，这些命令会提示 `No tagged files` 且不执行任何操作。

若要显示此菜单，可按住 Ctrl 或按 F9。参见[菜单](menus.md)。

## 命令列表

按菜单中的排列顺序如下。“目录树”与“文件”标明按键生效的窗口。

| 按键 | 命令 | 目录树 | 文件 | 功能说明 |
|---|---|---|---|---|
| Ctrl+A | Attributes | 是 | 是 | 修改已标记文件的权限。[权限与日期](attributes.md) |
| Ctrl+B | Bookmark | 是 | | 将光标所在的目录保存为书签。不使用标记状态。[书签](bookmarks.md) |
| Ctrl+B | Batch | | 是 | 根据掩码为每个已标记的文件生成脚本或列表的一行。[批处理](execute.md) |
| Ctrl+C | Copy | 是 | 是 | 将已标记的文件复制到同一目标目录中，不保留其路径。Alt+C 则保留路径。[复制与移动](copy-move.md) |
| Ctrl+D | Delete | 是 | 是 | 询问后删除已标记的文件。启用 `delete_to_trash` 时放入回收站。[删除](delete.md) |
| Ctrl+G | Git | 是 | 是 | 打开 Git 菜单。仅在启用 Git 选项时显示。[Git](git.md) |
| Ctrl+M | Move | 是 | 是 | 将已标记的文件移动到同一目标目录中。在不支持 kitty 键盘协议的终端中，请按 `F9` `M`。[复制与移动](copy-move.md) |
| Ctrl+N | New date | | 是 | 修改已标记文件的修改日期。[权限与日期](attributes.md) |
| Ctrl+O | Open | | 是 | 用系统默认程序逐个打开已标记的文件。[文件窗口](files.md) |
| Ctrl+P | Phrase | 是 | 是 | 在已标记的文件上打开搜索框：列出包含所输入词语的文件。[搜索框](filespec.md#搜索框-p) |
| Ctrl+R | Rename | 是 | 是 | 用掩码批量重命名已标记的文件。[重命名](rename.md) |
| Ctrl+S | Showall | 是 | | 列出所有已载入的已标记文件。[目录树窗口](tree.md) |
| Ctrl+S | Search | | 是 | 在已标记文件中搜索文本，并取消不包含该文本文件的标记。[搜索已标记的文件](search.md) |
| Ctrl+T | Tag | 是 | 是 | 文件窗口：标记整个列表。目录树：标记所有符合 Filespec 的已载入文件。[标记](tagging.md) |
| Ctrl+U | Untag | 是 | 是 | 文件窗口：取消标记整个列表。目录树：取消标记所有符合 Filespec 的已载入文件。 |
| Ctrl+V | View | | 是 | 在查看器中依次打开已标记的文件；按 `N` 和 `P` 切换文件。[搜索已标记的文件](search.md) |
| Ctrl+W | Where | 是 | 是 | 在列表中仅保留符合 SQL 条件的已标记文件。[SQL 条件](filespec.md#sql-条件-w) |
| Ctrl+Y | Yank | 是 | 是 | 将已标记文件的路径复制到剪贴板。 |

## 功能键

| 按键 | 窗口 | 功能说明 |
|---|---|---|
| Ctrl+F5 | 文件 | 将已标记的文件打包到新归档文件中。[归档文件](archives.md) |
| Ctrl+F7 | 目录树 | 标记光标所在分支中已载入的文件。 |
| Ctrl+F8 | 目录树 | 取消标记它们。 |
| Ctrl+F9 | 目录树 | 反选它们的标记状态。 |
| Ctrl+F11 | 两者 | 保存书签，如同目录树中的 Ctrl+B。 |
| Ctrl+Shift+Del | 两者 | 永久删除已标记的文件，不放入回收站。 |
| Ctrl+Z | 两者 | Oops：从回收站恢复最近删除的内容。 |

## 在压缩包内或远程机器上

菜单会省略在对应环境中无法运行的命令。

| 环境 | 未列出的命令 |
|---|---|
| 压缩包 | Attributes、Delete、Git、Move、New date、Rename、Ctrl+F5。Copy 执行解压操作。 |
| 远程机器 | Batch、Git、Ctrl+F5。 |
