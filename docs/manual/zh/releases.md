# 版本说明

最新版本在前。每个部分列出新增功能、与旧版本相比的变更，以及调整位置的按键。完整的发布说明见：
`https://github.com/andrelnfernandes/xylem/releases`。

## 0.8.0

**变更：部分 Alt 按键。** Alt 不再重复 Ctrl 的功能，而是如同 ZTreeWin 那样作为命令的另一种变体。

| 按键 | 0.7 及之前 | 0.8 起 |
|---|---|---|
| Alt+A | 修改已标记文件的权限 | 无（用 Ctrl+A） |
| Alt+D | 删除已标记的文件 | 重复文件 Duplicates（删除用 Ctrl+D） |
| Alt+R | 重命名已标记的文件 | 从内存释放分支 Release，同 `-`（重命名用 Ctrl+R） |
| Alt+T | 全部标记 | 无（用 Ctrl+T） |
| Alt+U | 全部取消标记 | 磁盘用量 Usage（取消标记用 Ctrl+U） |
| Alt+W | 磁盘用量 Usage | 无（用 Alt+U） |
| Alt+Y | 重复文件 Duplicates | 无（用 Alt+D） |

若要保留旧按键，可将它们加入 `config.toml` 中（可单独添加或全部添加）：

```toml
[keys.any]
"alt+a" = "attributes_tagged"
"alt+d" = "delete_tagged"
"alt+r" = "rename_tagged"
"alt+t" = "tag_all"
"alt+u" = "untag_all"
"alt+w" = "usage"
"alt+y" = "duplicates"
```

**变更：底部状态栏。** 所有命令均纳入菜单中，状态栏由三行构成，而在足够宽的屏幕上只需两行即可容纳整行菜单。在较窄的屏幕上，词语会等长缩写，尽量不丢弃条目。在归档内部和远程机器上，菜单仅列出可在对应环境中运行的命令。[菜单](menus.md)

**变更：提示框。** ↑ 以列表形式打开提示框的历史记录；↓ 不再逐项遍历条目。在 Branch 或 Showall 视图中按 `B` 可返回当前目录自身的列表。小文件窗口占据屏幕高度的四分之一。

**新增。**

- `W` 在列表中仅保留符合 SQL 条件的文件，Ctrl+W 在已标记文件中筛选。`contains('text')` 读取文件内容，`words('text')` 使用通过 Alt+N 构建的索引。[SQL 条件](filespec.md#sql-条件-w)
- `P` 打开搜索框，根据输入的词语列出文件。[搜索框](filespec.md#搜索框-p)
- CSV、TSV、JSON Lines、DBF 和 SQLite 表格在查看器中不再有行数限制。电子表格仍保留 50,000 行的上限。[查看器](viewer.md)
- Alt+L 将光标下的目录设为目录树的根节点。
- 在 Treespec 中，Ctrl+← 从路径中去掉最后一级文件夹，Ctrl+→ 将其恢复。
- `clock_seconds` 在顶部标题行的时钟中显示秒数。
- 本手册更新：新增屏幕说明、菜单、各菜单命令、术语以及使用指南等页面。

## 0.7.0

**变更：载入机制。** 仅在您明确要求时才读取目录。将光标移至未载入的目录时显示 `Not logged`；按 `+` 或 → 载入它。此前光标经过目录时会自动读取。[目录树窗口](tree.md)

**变更：提示框。** Log 命令，以及在非分屏模式下的 Copy 和 Move，打开时输入框为空。按 ↑ 可列出之前使用过的位置。

**新增。**

- 远程主机不再是只读的：支持向其复制、移动、重命名、新建目录、删除、Prune、修改权限、修改日期以及查找重复文件。[远程主机](remote.md)
- `docker://container/path` 通过 `docker exec` 打开容器。
- 书签支持自定义名称，远程机器上的书签会保存其访问地址。在 Log、Copy 和 Move 提示框中按 F11 可将书签填入提示框。[书签](bookmarks.md)
- rclone 挂载的文件夹与网络共享在后台异步读取；目录树即时呈现。[云存储](cloud.md)
- Usage 视图支持在其内部直接标记、删除、Prune 和查看文件。
- 欢迎屏幕、顶部标题行中的程序名称与版本、按类型着色的文件名以及圆角边框。[配置](configuration.md)

## 0.6.1

更新了电子表格解析器（calamine 0.38）以修复安全公告 RUSTSEC-2026-0436。

## 0.6.0

**新增。**

- 查看器将电子表格、CSV、JSON Lines、DBF、SQLite 和 Parquet 作为表格展示，并支持 SQL 筛选。SQLite 数据库还支持完整的只读查询。[查看器](viewer.md)
- 查看器中的 Ctrl+B 可根据表格记录或文本行生成文件：JSON、CSV、XML、HTML、模板文件或 SQLite 数据表。[批处理](execute.md)
- Duplicates 可在本地磁盘及通过 rclone 报告的哈希值查找完全相同的文件和文件夹。原按键为 Alt+Y；从 0.8 起变更为 Alt+D。[重复文件](duplicates.md)
- 查看器原生渲染 Markdown 文件。按 `T` 显示源码。
- 查看器支持用正则表达式筛选行 (`&`)。[筛选行](filter.md)
- 在 rclone 挂载的文件夹上按 `*` 可在单次请求中获取整个分支。
- 浏览 Git 仓库绝不自动执行仓库指定的外部程序，除非该目录位于 `git_trusted` 白名单中。[Git](git.md)

## 0.5.0

**变更。** 目录树中的 `-` 从内存中释放分支；← 仍仅折叠目录。`E` 在 Xylem 自身的界面框架内打开编辑器。

**新增。**

- 针对恶意文件的安全防护机制：解压上限、提取路径校验、拦截发往终端的控制字符。[安全](security.md)
- 查看器支持自动折行 (`W`)、显示行号 (`#`)、跳转至指定行 (`g`)、解压查看 `.gz`、`.xz`、`.zst` 和 `.bz2` 文件，并支持 CP437、CP850、Windows-1252 和 UTF-16 编码。
- 六种语言的手册支持；在帮助中按 `L` 切换语言。
- 目录树中的 Ctrl+S 仅列出所有已载入文件中的已标记文件。
- Git 菜单中新增 Graph 提交图。
- Nerd Fonts 图标支持，默认关闭。
- 提供 `.deb` 和 `.rpm` 安装包。

## 0.4.0

**新增。**

- Usage 磁盘用量视图：按占用空间列出分支中的文件夹和文件。原按键为 Alt+W；从 0.8 起变更为 Alt+U。[磁盘用量](usage.md)
- Alt+K 百分比模式显示各目录占其上级目录的比例份额。
- 通过 SSH 访问远程主机（该版本中为只读）。[远程主机](remote.md)
- 通过 rclone 支持 Google Drive、S3 等云存储。[云存储](cloud.md)
- 新增 slate、graphite、daylight 和 vivid 配色主题。
- `config.toml` 中的 `[editors]` 可按文件名选择编辑器，`E` 拒绝在没有匹配编辑器的情况下打开二进制文件。
- `xy --man` 显示 man 手册页。

## 0.3.0 及更早版本

- 0.3.0：F1 帮助与 man 手册页；支持 7z、tar.xz、tar.bz2 和 tar.zst 归档；新增所有者与类型显示模式 (Alt+F)；在复制冲突时同时显示双方文件详情。
- 0.2.0：程序名称由 xt 更名为 Xylem，命令更名为 `xy`。支持放入回收站的删除与撤销恢复 (Oops)、使用掩码批量重命名以及剪贴板支持。
- 0.1.0：首个版本，名为 xt。
