# 布局：定宽文件与多记录类型文件

系统之间交换的许多文件是没有列名的文本：每一行是一条记录，哪些字符属于哪个字段写在手册里。银行文件（CNAB 240）、税务文件（SPED）以及 dBase 和 Clipper 的 SDF 导出都是这样。布局（layout）告诉 Xylem 每个字段在哪里，查看器便把文件显示为表格：每种记录类型一个工作表，每一行以它在文件中的行号开头。

## 按布局打开文件

Xylem 能根据第一行识别某些文件，并自行以表格形式打开，`V` 和自动预览（F7）中都是如此。提示会说明使用了哪个布局：`Layout: FEBRABAN CNAB 240 — Padrão Geral  ·  K changes it`。对于其他文件，请按 `K`。

| 按键 | 作用 |
|---|---|
| `K` | 布局列表：输入名称的一部分来缩小范围，Enter 应用，**F2** 输入布局文件的路径，Esc 返回。已在使用布局时，第一项会关闭布局并以文本显示文件 |
| `[` `]` | 上一种或下一种记录类型 |
| `Tab` | 记录类型列表，附带每种类型的行数 |
| `i` | 记录的字段：位置、大小、类型、说明及每个代码的含义；上方是布局、其来源、合计校验及其结果 |
| `T` | 跳到文本中光标所在记录的那一行，再按一次返回 |
| `\` 或 `&` | SQL 筛选，按字段名：`valor_pagamento > 1000` |
| Ctrl+B | 对当前工作表的记录执行 Batch（[查看器中的 Batch](execute.md)） |

工作表按每种记录类型在文件中首次出现的顺序排列：`0 - Header de Arquivo (1)`、`3A - Segmento A (420)`……不符合任何记录的行会整行放入 `! No record type` 工作表。第一列 `#line` 是在文件中的行号，银行和税务机关的校验程序正是用它来指出错误。字段的代码有含义时，表格最后一行会显示：`Content: 2 — Retorno (Banco → Cliente)`。

数值按布局读取：代码保留前导零（`00012345`），日期变为 `2026-11-09`，时间变为 `14:30:00`，金额保留精确的小数位（`1234.50`）；不做任何舍入。不符合布局的值（数字中有字母、2 月 31 日）按原样显示，并由 `i` 计数。

### 合计校验

尾记录（trailer）带有合计：批次的行数、批次数、付款总额。Xylem 在读取时以整数计数和求和，并在 `i` 中说明哪些相符、哪些不符，附上行号、文件中的值和统计出的值。有不符时，标题栏会提示：`1 check failed: i shows`。只读了一部分的文件（被 Esc 或时间限制中止）不做校验。

## 布局从哪里来

| 位置 | 是否自动使用？ |
|---|---|
| Xylem 的目录，内置于程序中 | 是，当第一行符合其 `[detect]` 时 |
| 您的文件夹：`~/.config/xylem/layouts/`（Windows 上为 `%APPDATA%\xylem\layouts\`），最多三层子文件夹 | 是，且优先于内置目录 |
| 文件旁边（`K`，`Here` 部分），或任意路径（`K` 后按 F2） | 只在您选择时 |

正在浏览的文件夹中的布局文件绝不会被自动读取：下载或解压出的文件夹可能带有一个专门用来欺骗的布局。您选择它时，它会像数据一样被严格读取（见下文“安全”）。

目前目录中包含：

| 布局 | 读取的内容 |
|---|---|
| FEBRABAN CNAB 240 — Padrão Geral | 文件头和文件尾；通过账户入账、支票、OP、TED 和 Pix 付款（A、B、C、Z 段）；支付托收单据（J、J-52）；托收（发送时的 P、Q、R；回执中的 T 和 U）。依据 FEBRABAN 手册 11.0 版 |
| SPED — genérico | 任何以竖线分隔字段的 SPED 文件（EFD ICMS/IPI、EFD-Contribuições、ECD、ECF……）：每种记录一个工作表，每个区块的开始与结束记录以及带校验的第 9 区块。未声明的记录其字段按编号命名（`CAMPO_02`、`CAMPO_03`……）。已签名文件中 `9999` 之后的内容（数字签名）不会读取 |

[代码仓库](https://github.com/andrelnfernandes/xylem/tree/main/layouts)中的 `layouts/` 文件夹列出了计划中的布局，以及如何贡献布局，无需编写 Rust。

### 由 .dbf 得到 SDF

dBase 和 Clipper 中的 `COPY TO 文件 SDF` 把记录的各字段按宽度依次并排写出。在 `K` 中，文件旁边的 `.dbf`（或用 F2 指定的）会给出其布局：字段按顺序排列，日期为 `YYYYMMDD`，数字带小数点。SDF 不导出备注（memo）字段，因此不包括它们。

## 编写布局

布局是一个文件名以 `.xlayout.toml` 结尾的 TOML 文件。定位式示例：

```
schema = 1
name = "旧系统的客户"
line_length = 40
[discriminator]
pos = [1, 1]
[[records]]
id = "0"
name = "文件头"
role = "header"
fields = [
  { name = "type", pos = [1, 1], type = "code" },
  { name = "created", pos = [2, 9], type = "date", format = "YYYYMMDD" },
  { pos = [10, 40], type = "filler" },
]
[[records]]
id = "1"
name = "客户"
fields = [
  { name = "type", pos = [1, 1], type = "code" },
  { name = "code", pos = [2, 7], type = "code" },
  { name = "name", pos = [8, 30], type = "text" },
  { name = "balance", pos = [31, 40], type = "decimal", decimals = 2, sign = "trailing" },
]
```

位置从 1 开始并包含两端：`pos = [8, 30]` 表示第 8 到第 30 个字符。`start = 8, len = 23` 含义相同；与 `pos` 并列的 `len` 必须与之一致，这样可以发现输入错误。启用 `strict_positions`（默认开启）时，一条记录的字段必须覆盖整行，不能有空隙或重叠：请把空白部分设为 `filler` 类型。

| 键 | 含义 |
|---|---|
| `schema` | 始终为 `1` |
| `name` | 在 `K` 和标题栏中显示的名称 |
| `format` | `"positional"`（默认）或 `"delimited"` |
| `encoding` | `"auto"`（查看器识别出的编码；`O` 可切换）、`"utf-8"`、`"cp1252"`、`"cp850"` 或 `"cp437"` |
| `offset_unit` | 定位式：`"chars"`（默认）或 `"bytes"` |
| `line_length`、`strict_positions` | 定位式：一行的长度，以及字段是否必须覆盖整行 |
| `hide_filler` | `filler` 字段不显示在表格中（默认 `true`）；`i` 始终列出它们 |
| `delimiter`、`outer_delimiters` | 分隔式：分隔符（一个字符），以及行是否也以它开头和结尾（如 SPED） |
| `allow_undeclared_records` | 分隔式：布局未声明的记录各自拥有一个工作表 |
| `decimal_sep`、`date_format` | 所有字段的默认值：`"implied"`、`","` 或 `"."`；以及日期格式 |
| `stop_after_trailer` | 读到 `trailer` 记录即停止（之后的内容不读） |
| `[detect]` | 如何根据第一行识别文件：`line_length`、`first_line`（条件）和 `extensions` |
| `[discriminator]` | 记录代码所在的位置：`pos`，分隔式则为 `col` |
| `description`、`country`、`category`、`spec_source`、`spec_version`、`spec_url`、`status` | 关于布局及其手册；与手册核对后 `status` 为 `"verified"` |

每个 `[[records]]` 有一个 `id`（供识别器使用的代码；`ids` 可添加更多代码）、一个 `name`、一个 `role`（`header`、`batch_header`、`detail`、`batch_trailer` 或 `trailer`）以及它的 `fields`。仅凭代码无法区分记录时，`match` 添加针对本行的条件，`batch_match` 添加针对所在批次头记录那一行的条件：在 CNAB 240 中，付款的 A 段和扣款的 A 段只在批次头的操作类型上不同。条件由一个位置（`pos` 或 `col`）和 `value = "3"`、`in = ["1", "2"]` 或 `regex = "^[0-9]+$"` 之一组成。代码相同的记录中，条件多的先尝试。

| 字段键 | 含义 |
|---|---|
| `name` | 列名，也用于 SQL 筛选 |
| `pos`、`start` 加 `len`，或 `col` | 所在位置 |
| `type` | `text`、`code`（不表示数量的数字：税号、账号、邮编）、`int`、`decimal`、`date`、`time` 或 `filler` |
| `decimals`、`decimal_sep` | 小数位数，以及小数点的位置：`"implied"`（COBOL 的 `V99`）、`","` 或 `"."` |
| `sign` | `"none"`、`"leading"`、`"trailing"` 或 `"overpunch"`（COBOL 的区位符号：`{`、`A`-`I`、`}`、`J`-`R`） |
| `format` | 日期：`YYYYMMDD`、`DDMMYYYY`、`YYMMDD`、`DDMMYY` 或 `YYYYDDD`；时间：`HHMMSS` 或 `HHMM`。两位年份 00 至 69 属于 2000 年代 |
| `trim` | 从文本中去掉的空格：`"right"`（默认）、`"left"`、`"both"` 或 `"none"` |
| `desc`、`values` | 说明，以及每个代码的含义：`values = { "0" = "新增", "9" = "删除" }` |
| `check` | 该值必须与什么相符，见下表 |

| `check` | 该值必须等于 |
|---|---|
| `total_lines` | 文件到本行为止的行数 |
| `record_count:1*` | 具有此 id 的记录行数；`*` 表示所有以此开头的 id，`+` 可添加更多 |
| `sum:3A.valor` | 整个文件中某记录某字段之和；`+` 可添加更多 |
| `batch_lines` | 从批次头到本行的行数 |
| `batch_sum:3A.valor+3J.valor` | 批次内的合计 |
| `count_by_record:REG_BLC` | 代码等于指定字段值的行数（SPED 的 9900） |
| `block_lines` | 代码开头与本行相同的行数（SPED 的 x990） |

COBOL copybook（`copybook = ...`）、EBCDIC 和压缩数字（`COMP-3`）已在计划中，目前尚不支持。

## 安全

数据以及从您的文件夹之外选择的布局都可能是恶意的。布局会像数据一样被严格读取：最大 1 MiB，任何未知的键或类型都会被拒绝，每个位置和大小都会被检查，名称中不能含有控制字符或文本方向字符，记录、字段、代码、条件和正则表达式都有上限。正则表达式以线性时间运行。布局不能读取其他文件：目前 `copybook` 会被拒绝。

数据按表格模式的限制读取：3 秒（自动预览中为 100 毫秒）、每个工作表 50,000 行、256 列，超过 512 KiB 的行不会拆分为字段。Esc 可停止读取。单元格与其他内容一样会做终端安全处理：控制字符和文本方向字符显示为 `·`。

另见：[查看器](viewer.md)。
