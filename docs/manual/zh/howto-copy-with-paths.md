# 从多个文件夹复制文件并保留路径

本指南演示如何将 `~/projects` 下的所有 JPG 文件复制到 `/mnt/backup`，并使每个文件保持其原有的子目录结构。相同的操作步骤同样适用于移动（Alt+M）。

## 操作步骤

1. 在目录树中，将光标置于 `projects` 目录上。
2. 按 `B`。Xylem 载入该分支，文件窗口列出 `projects` 及其下方所有文件夹中的文件。在庞大的分支上，底部状态栏会显示载入进度，按 Esc 可随时取消。
3. 按 `F`，输入 `*.jpg` 并按 Enter。此时列表中仅保留 JPG 文件。
4. 按 Ctrl+T。列表中的每个文件都被标记（`■`）。
5. 按 Alt+C。提示框询问目标路径。
6. 输入 `/mnt/backup`。在按 Enter 之前，查看提示框中显示的路径模式，按 **F7** 直至其显示为 `Relative`。
7. 按 Enter。若 `/mnt/backup` 尚不存在，Xylem 会在创建前询问确认。

![七个步骤：在 projects 上执行 Branch、设置 filespec *.jpg、Ctrl+T、Alt+C、设置为 Relative 模式的目标路径，以及完成复制的 21 个文件](../../screenshots/howto-copy-paths.gif)

屏幕下方的说明条展示了正在执行的步骤及其对应按键。

## 路径模式的选择

步骤 6 决定了每个文件在目标目录下的路径起始层级。对于 `~/projects/photos/2025/trip/DSC_01.jpg`（在 `projects` 上打开 Branch 视图）：

| 模式 | 文件写入目标位置 |
|---|---|
| Full | `/mnt/backup/home/user/projects/photos/2025/trip/DSC_01.jpg` |
| Tree | `/mnt/backup/` 加上从 Xylem 目录树根节点开始的相对路径 |
| Current | `/mnt/backup/projects/photos/2025/trip/DSC_01.jpg` |
| Relative | `/mnt/backup/photos/2025/trip/DSC_01.jpg` |

除非配置中的 `copy_paths` 另有指定，否则提示框默认采用 `Full` 模式。

## 变体用法

- 不保留路径：改用 Ctrl+C 替代 Alt+C，所有 JPG 文件将直接汇集存入 `/mnt/backup`。若同名文件发生冲突，Xylem 会逐一询问处理方式。
- 按日期或大小筛选：在步骤 3 中输入 `*.jpg,>TODAY-30`，可仅保留最近 30 天内修改过的 JPG。[Filespec](filespec.md) 说明了具体语法。
- 针对整个磁盘：用 `*` 载入更多分支，在步骤 2 中按 `S` (Showall)。
- 手动挑选：跳过步骤 4，使用 `T` 逐一标记所需的文件。

## 异常排查

- **Alt+C 触发了 Compare（比较）。** 焦点当前位于目录树上。Alt+C 仅在文件窗口中才表示保留路径复制；按 Enter 进入文件窗口即可。
- **列表中缺少部分文件。** 此前设置的 Filespec 依然生效。查看文件窗口标题栏；按 `F`，输入 `*` 并回车即可清除过滤。
- **目标目录中的层级从 `/home` 开始。** 当前使用了 `Full` 模式。请参考步骤 6 调整为 `Relative`。

按 Ctrl+U 可取消当前列表中的所有标记，按 `B` 可恢复为仅显示当前目录自身的文件。

另请参阅：[复制与移动](copy-move.md)、[标记](tagging.md)、[目录树窗口](tree.md)。
