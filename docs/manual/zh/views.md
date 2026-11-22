# 视图 (Alt+V)

**Alt+V** 打开列表的视图：以其他方式查看文件窗口中的文件，用来回答目录树和列表不易回答的问题。一个小菜单会询问打开哪一个，每个都有自己的字母：

| 按键 | 视图 | 回答的问题 |
|---|---|---|
| `S` | [扫描](#scan) | 这些文件中有什么值得一看？ |
| `T` | [时间线](#timeline) | 什么发生了变化，在什么时候？ |
| `L` | [表格](#table) | 哪些文件满足这个条件，以表格形式查看？ |

视图作用于**当前列表**：目录、Branch 或 Showall，以及当前生效的 Filespec 和 `W` 条件。视图只读取：视图中的任何操作都不会删除、移动或修改文件。您在视图中的选择会作为筛选回到经典列表，之后 Ctrl+T、Ctrl+C 或批处理照常工作。**不带文本的 W** 会去掉这个筛选，就像去掉条件一样。

每个视图也可以在 `[keys]` 中拥有自己的按键（[配置](configuration.md)），默认没有：`view_scan`、`view_timeline`、`view_table`。

## 扫描

[查看器](viewer.md)为可能有恶意的文件（可执行文件、Office 文档、补丁）生成的报告，平时用 `V` 一次查看一个文件；扫描把它们应用到整个列表上：您下载的 ZIP、您克隆的仓库、一个 U 盘。**Alt+V, S**（要包含下层文件夹，先按 `B` 打开 Branch）。

```
      1  Names with control or text-direction characters
      1  Names that deceive (fatura.pdf.exe, spaces, a trailing dot)
      3  setuid, setgid, writable by anyone
      2  Links that leave the folder, or point to nothing
      1  Executables: packed, without PIE or NX, writable and executable
      1  Office: macros, remote template, embedded objects, hidden text
      0  Patches: dangerous paths, hidden characters, hidden changes
      0  Not analysed: over a limit, or the reader could not take it
```

| 类别 | 检查的内容 |
|---|---|
| Names with control or text-direction characters | 每个文件和文件夹的名称：控制字符、使文本反向的字符（显示为 `exe.pdf` 而实际是 `fdp.exe` 的名称）、不是 UTF-8 的字节 |
| Names that deceive | 文档扩展名位于可执行扩展名之前（`fatura.pdf.exe`、`photo.jpg.scr`）、扩展名前有空格、名称以空格或点结尾 |
| setuid, setgid, writable by anyone | 每个文件以及每个文件夹的权限模式：任何人都可写入的文件夹只有在没有粘滞位时才计入，`/tmp` 就有粘滞位 |
| Links that leave the folder, or point to nothing | 每个链接指向哪里，依据其文本：绝不会跟随链接去读取目标内容 |
| Executables | ELF、PE 和 Mach-O，依据前几个字节而非名称识别：没有 PIE、没有 NX 或 DEP、没有 ASLR、可执行栈、经过加壳（UPX 等）或同时可写可执行的节、最后一个节之后的数据 |
| Office | Word、PowerPoint 和 OpenDocument：宏、文档打开时从外部获取的内容（远程模板）、嵌入对象、隐藏文本和隐藏的幻灯片 |
| Patches | `.patch` 和 `.diff`：用 `..` 跳出目录、使用绝对路径、进入 `.git` 或变为可执行的文件；带隐藏字符的名称；计数不符的 hunk；只修改了看不见的内容的行 |
| Not analysed | 读取器无法处理的文件，各自附有原因：超过限制、包已损坏、读取器失败 |

名称和文件的权限模式已经在内存中，会立即显示。文件夹的权限模式、链接和内容需要访问磁盘，在后台进行：屏幕随结果陆续填充，显示还剩多少，按 **Esc** 停止。

内容经过与[查看器](viewer.md)（`V`）报告相同的读取器，使用相同的限制，一次一个文件。超过限制的文件，或读取器无法处理的文件，会连同原因归入 **Not analysed**：绝不会悄无声息地漏掉。最多读取 `scan_max_files` 个文件（10000）的内容；超出的文件列为未分析（[配置](configuration.md)）。

| 按键 | 功能 |
|---|---|
| ↑ ↓ | 移动 |
| Enter | 该类别的文件，每个都附有归入原因 |
| Enter，在文件上 | 回到文件窗口，只保留该类别的文件，光标位于此文件上 |
| Enter，在文件夹上 | 在目录树中显示该文件夹 |
| `V`，在文件上 | 查看该文件及其报告；Esc 回到扫描 |
| `R` | 重新扫描 |
| Esc | 回到类别，然后回到列表 |

在列表上按 Esc 会保留扫描结果：对同一列表再按 Alt+V, S 会按原样打开它，不重新读取。`R` 会重新读取。

### 不检查的地方

- **Windows 驱动器以及 FAT、exFAT、NTFS、SMB、rclone 挂载。** 在这些地方，驱动程序给每个文件相同的权限模式，因此权限模式说明不了什么，扫描不会计入它们：该类别会说明这一点。在 WSL 下就是 `/mnt/c` 和其他 Windows 驱动器。
- **归档文件内**（Alt+F5），文件的权限模式是归档文件记录的模式，会计入：TAR 中的 setuid 文件值得知道。每个成员在限制内（64 MiB）解压，并像在磁盘上一样读取。不检查文件夹和链接。在 TAR 中，位置很靠后的成员不会解压，列为未分析。
- **云存储挂载上**（rclone）不读取内容：否则每个文件都要下载。名称和其余项目照常检查。
- **远程主机上**目前只检查名称和文件的权限模式：读取内容需要通过 SSH 传输每个文件。

扫描不是杀毒软件。它指出值得一看的内容，并不表示其余内容是安全的。

## 时间线

以直方图按修改日期显示列表中的文件：某个文件夹何时最忙碌、上周进来了什么、哪些照片来自那次旅行。它不从磁盘读取任何内容：只使用内存中已有的日期。

```
 2026-11-18: 312 files, 1.4 GiB

                                    █
                                    █▆
 ▁▁▂▁▁▁▁▁▃▁▁▂▁▁▁▁▁                  ██▁
                                    ▲
 11-01     11-05     11-09     11-13     11-17
```

刻度根据日期自动选择：小时、天、周、月或年。1980 年之前和未来的日期分别归入两端各自的柱中，因为错误或恶意的日期确实存在，否则会把图表压平。没有日期的文件在图表下方计数。

| 按键 | 功能 |
|---|---|
| ← → PgUp PgDn Home End | 在柱之间移动；顶部一行显示该柱的文件数和字节数 |
| Shift+← → | 选择一个范围的柱 |
| `+` `-` | 更细或更粗的刻度（`+` 从天到小时） |
| `B` | 柱的高度按字节数而不是文件数 |
| Enter | 回到文件窗口，只保留该柱或该范围中的文件 |
| Esc | 返回，不做任何更改 |

Enter 把选择转换为针对 `modified` 的 [W 条件](sql.md)，例如 `modified >= '2026-11-18 00:00:00' AND modified < '2026-11-19 00:00:00'`，并用 AND 与原有的条件连接。文件窗口的标题会显示它，**不带文本的 W** 会去掉它。

## 表格

列表在[表格查看器](tables.md)中打开，每行一个文件，列与 [W 条件](sql.md)的相同：`name`、`ext`、`dir`、`size`、`disk`、`modified`、`age_days`、`mode`、`kind` 和 `tagged`；当列表中有文件具有 Git 状态或属于某个[重复文件](duplicates.md)组时，还有 `git` 和 `dup`。这些值就是 `W` 看到的值，因此在一处有效的条件在另一处也有效。

在那里，列表拥有表格的所有功能：SQL 筛选（`\` 或 `&`）、以表单形式查看记录（Enter）、复制记录、标记（`m`）以及[批处理](execute.md)（Ctrl+B），每行输出一行。数百万个文件的列表也能立即打开：表格像大文件一样分窗口读取。

| 按键 | 功能 |
|---|---|
| `\` 或 `&` | 表格的 SQL 筛选 |
| `s` | 按光标所在的列排序；再按一次 `s` 则反向排序；筛选保持不变 |
| `W` | 回到文件窗口，以该筛选作为其 W 条件，并用 AND 与原有的条件连接 |
| Esc | 返回，不做任何更改 |

表格是打开时的列表：之后标记或修改的文件，要到下次打开时才会显示在其中。`type` 需要读取每个文件，因此不是这里的列；`W` 中仍然可用。
