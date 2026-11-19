# Xylem 手册

Xylem 是一款继承了 XTree 与 ZTreeWin 屏幕布局与按键习惯的文本模式文件管理器：上方是目录树，下方是当前目录中的文件，右侧是统计信息。它会将目录载入（读取）到内存中，因此一个列表即可包含整个分支的文件，一条命令即可对其中标记的所有文件完成复制、移动、删除、比较或重命名。

Xylem 将外部文件视为潜在恶意：绝不应因文件内容或文件名而导致崩溃、卡死、执行代码、越界写入或耗尽内存。[安全](security.md)列出了防护措施及未涵盖的范围。

**F1**（若终端占用了 F1，也可按 **Alt+H**）打开与当前操作相关的帮助：在提示行中打开该提示的主题；在目录树或文件窗口中打开该窗口的主题。

| 帮助中的按键 | 功能 |
|---|---|
| Tab, Shift+Tab | 选择下一个或上一个链接 |
| Enter | 访问链接 |
| Backspace | 返回 |
| `I` | 本索引 |
| `K` | 当前生效的按键 |
| `L` | 切换语言 |
| Esc | 关闭 |

本手册的西班牙语、法语、德语和中文版尚未经过母语人士审校。纠错与建议：`https://github.com/andrelnfernandes/xylem/issues`。

## 从这里开始

- [Xylem 是什么](overview.md)：载入、标记与命令涵盖的功能
- [安装与最初五分钟](start.md)
- [屏幕](screen.md)：目录树、文件窗口、统计信息与底部状态栏
- [菜单、Ctrl 与 Alt](menus.md)：单独字母、配合 Ctrl、配合 Alt 与 F9
- [The mouse](mouse.md) (in English for now): turning it on in F12, the wheel, clicks and the footer
- [术语](glossary.md)：Log、Branch、Tag、Filespec

## 命令

- [DIR 和 FILE 菜单](commands.md)：单独的字母
- [Ctrl 菜单](commands-ctrl.md)：已标记的文件
- [Alt 菜单](commands-alt.md)：分支、路径与工具
- [Git](git.md)：Git 菜单 (Ctrl+G)
- [功能键与符号](commands-keys.md)
- [当前生效的按键](keys.md)：包含配置文件的更改

## 窗口与视图

- [目录树窗口](tree.md)：移动、载入、Branch 与 Showall
- [文件窗口](files.md)：显示模式、Autoview、打开文件
- [分屏](split.md)：并排显示两个窗口
- [查看器](viewer.md)：文本、十六进制与表格、搜索、语法高亮
- [筛选行](filter.md)：查看器中的 `&`，附正则表达式指南
- [搜索已标记的文件](search.md)：Ctrl+S 与 Ctrl+V
- [信息](info.md)：Info、Avail、统计信息与目录大小
- [磁盘用量](usage.md)：空间去向 (Alt+U)
- [重复文件](duplicates.md)：内容相同的文件和文件夹 (Alt+D)

## 文件操作

- [标记与批量操作](tagging.md)
- [复制与移动](copy-move.md)、[选择文件夹](picker.md)
- [删除、回收站与 Prune](delete.md)
- [重命名与批量重命名](rename.md)
- [权限与日期](attributes.md)
- [Filespec：文件过滤器](filespec.md)
- [Treespec 与 Log](treespec.md)
- [比较](compare.md) 与 [排序](sort.md)
- [编辑、执行与批处理](execute.md)
- [归档文件](archives.md)：ZIP、7z 与 TAR

## 超越 XTree

- [内置终端](terminal.md)
- [远程主机 (SSH)](remote.md)：作为目录树浏览另一台机器或容器
- [云存储 (rclone)](cloud.md)：Google Drive、S3 等
- [用 SQL 查找文件](sql.md)：针对文件及其内容的条件（W），以及文件中的单词（P）
- [表格](tables.md)：以表格形式查看电子表格、CSV、dBase、SQLite 和 Parquet，可用 SQL 筛选，并以表单形式查看每条记录
- [书签](bookmarks.md)

## 实用指南

- [从多个文件夹复制文件并保留路径](howto-copy-with-paths.md)
- [用计数器批量重命名文件](howto-rename-mask.md)
- [用 SQL 条件挑选文件](howto-sql-filter.md)
- [查找重复文件并保留一份](howto-duplicates.md)
- [找出占满磁盘的内容](howto-disk-space.md)
- [用 SQL 筛选 CSV 文件或电子表格](howto-table-sql.md)
- [通过 SSH 从另一台机器复制文件](howto-ssh-copy.md)
- [在 shell 命令中使用已标记的文件](howto-terminal-tagged.md)

## 参考

- [配置](configuration.md)：F12 界面、主题、按键
- [命令行](cli.md)
- [终端与按键](terminals.md)：F1、Ctrl+M、kitty 键盘协议
- [安全](security.md)
- [退出并切换 shell 目录](quit.md)
- [Windows](windows.md)
- [与 ZTreeWin 的比较](ztreewin.md)
- [版本说明](releases.md)
