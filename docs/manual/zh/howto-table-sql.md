# 用 SQL 筛选 CSV 文件或电子表格

本指南演示如何将 `customers.csv` 作为表格打开，并保留其中 `plan` 为 `team` 或 `balance` 大于 1000 的数据行。

## 操作步骤

1. 在文件窗口中，将光标置于 `customers.csv` 上并按 `V`。查看器以表格形式展示该文件，首行自动作为列名。
2. 按 `&`。提示框询问筛选条件。
3. 输入 `plan = 'team' OR balance > 1000` 并按 Enter。

![三个步骤：在 customers.csv 上按 V 展示九行数据、& 提示框，以及条件筛选后保留的三行数据](../../screenshots/howto-table-sql.gif)

底部状态栏显示 `Filter matched 3 rows`。各行保留其在原文件中的行号。按 `&` 并在提示框中直接回车可重新显示所有行。

## 支持作为表格打开的文件格式

| 格式 | 扩展名 |
|---|---|
| CSV, TSV | `.csv` `.tsv` `.tab` |
| JSON Lines | `.jsonl` `.ndjson` |
| 电子表格 | `.xlsx` `.xls` `.xlsb` `.ods` |
| dBase | `.dbf` |
| SQLite | `.db` `.sqlite` `.sqlite3` |
| Parquet | `.parquet` |

文件必须位于本地磁盘上。远程机器上或归档内部的文件将作为普通文本打开。

## 编写筛选条件

| 筛选目标行 | 条件写法 |
|---|---|
| 某列等于指定文本 | `plan = 'team'` |
| 数值位于某个区间内 | `balance >= 0 AND balance < 100` |
| 文本以特定内容开头 | `name LIKE 'Ana%'` |
| 文本包含特定内容（不区分大小写） | `city ILIKE '%paulo%'` |
| 单元格为空 | `email IS NULL` |
| 列名包含特殊字符无法直接引用 | 使用列字母代号：`C = 'pro'` |

看起来像数字的单元格会被当作数字处理：应编写为 `code = 7`，而非 `code = '7'`。[SQL 筛选](viewer.md#sql-筛选)列出了所有可用运算符。

## 电子表格与数据库中的特殊操作

- 电子表格没有固定的列名：列依次为 `A`、`B`、`C`。按 `[` 与 `]` 可切换至上一个或下一个工作表，按 Tab 可列出所有工作表。
- 在 SQLite 数据库中，按 Tab 可列出所有数据表和视图。同一个提示框还接受以 `SELECT` 或 `WITH` 开头的完整查询：

```
SELECT plan, count(*), sum(balance) FROM customers GROUP BY plan
```

Xylem 仅以只读方式访问数据库。`UPDATE`、`DELETE` 等写入操作均会被拒绝。

## 保存筛选结果

按 Ctrl+B 可将筛选保留的行导出保存为文件：CSV、JSON、HTML、SQL 脚本或 SQLite 数据表。[批处理](execute.md)详细说明了各种导出格式。

## 异常排查

- **提示 `SQL error`。** 条件语句无法被解析。此前生效的筛选继续保留。
- **明明能看到的列却没有匹配到任何行。** 不存在的列名会被当作空值处理，且不会报错。请仔细检查拼写，或直接使用列字母代号。
- **文件作为普通文本打开。** 其扩展名未包含在上表中，或者文件经过压缩、位于远程机器上或归档内部。当两者均存在时，按 `T` 可在表格模式与文本模式之间切换。

另请参阅：[查看器](viewer.md)、[用 SQL 挑选文件](howto-sql-filter.md)。
