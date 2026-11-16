# 用 SQL 条件挑选文件

本指南演示如何列出 `~/projects` 下所有大于 1 MB 的 JPG 文件并进行标记，以便后续执行复制、移动或删除。

## 操作步骤

1. 在目录树中，将光标置于 `projects` 上并按 `B`。文件窗口列出该分支的全部文件。
2. 按 `W`。提示框询问 SQL 条件。
3. 输入 `ext = 'jpg' AND size > 1000000` 并按 Enter。列表中仅保留符合条件的文件，底部状态栏显示匹配的文件数量。
4. 按 Ctrl+T。列表中的匹配文件全部被标记。

![四个步骤：在 projects 上执行 Branch、按 W、输入条件以及用 Ctrl+T 标记筛选出的 12 个文件](../../screenshots/howto-sql-filter.gif)

文件窗口的标题栏显示当前条件：`Branch: * where ext = 'jpg' AND size > 1000000 (12)`。再次按 `W` 并在提示框中直接回车可恢复完整列表；已有标记将继续保留。

## 编写条件

条件内容等同于 SQL 查询中 `WHERE` 子句的内容。文本需用单引号括起，数字需完整输入。

| 筛选目标 | SQL 条件 |
|---|---|
| 最近一周修改过的文件 | `age_days < 7` |
| Markdown 与文本文件 | `ext IN ('md', 'txt')` |
| 文件名以 `report` 开头 | `name LIKE 'report%'` |
| 尚未标记的文件 | `NOT tagged` |
| 包含指定文本的文件 | `contains('invoice')` |

[SQL 条件](filespec.md#sql-条件-w)列出了所有可用的数据列。

## 异常排查

- **提示 `No column "jpg"`。** 文本未用单引号括起。`ext = jpg` 会被当作比较两个数据列；`ext = 'jpg'` 才是与字符串比较。
- **预期的文件未出现在列表中。** `W` 仅对已载入的文件进行筛选。请先打开 Branch 视图，或使用 `*` 载入更多目录后再使用 `S` (Showall)。
- **列表没有任何变动。** 没有文件满足该条件，此时列表保持原状不变。
- **`contains` 在运行前弹出提示。** 它必须读取待检查的每个文件，提示会说明待读取的文件数量与总字节大小。

另请参阅：[Filespec](filespec.md)、[标记](tagging.md)、[保留路径复制](howto-copy-with-paths.md)。
