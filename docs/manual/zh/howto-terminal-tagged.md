# 在 shell 命令中使用已标记的文件

本指南演示如何在内置终端中，对 Xylem 中标记的文件运行 `wc -c`。相同的操作步骤适用于任何接受文件名作为参数的命令。

## 操作步骤

1. 在目录树中，将光标置于目标目录上，按 → 读取它，按 Enter 进入其文件列表。
2. 标记所需文件：按 `T` 逐个标记，或按 Ctrl+T 标记整个列表。
3. 按 Alt+X。屏幕下半部分打开终端，shell 自动定位在上方所显示的目录中。
4. 输入命令加一个空格，如 `wc -c `，然后按 Alt+O。Xylem 会在光标处自动输入已标记文件的名称，每个名称均加上引号。
5. 按 Enter。

![五个步骤：notes 的文件列表、Ctrl+T、Alt+X 打开终端、wc -c 配合 Alt+O 自动输入两个文件名，以及命令输出结果](../../screenshots/howto-terminal-tagged.gif)

按 Alt+X 可返回 Xylem。shell 将在后台继续运行并保留其命令历史，直至输入 `exit` 关闭。

## Alt+O 输入的内容格式

文件名转换为相对于当前 shell 目录的相对路径并加上引号，因此包含空格的文件名会作为独立的参数正确传递：

```
wc -c 'ideas.md' 'todo.md'
```

在 Branch 或 Showall 视图中，已标记的文件分布在多个目录中，每个文件名都会带上相对于当前 shell 目录的完整相对路径。

## 环境变量支持

进入终端时，Xylem 还会将已标记的文件存入环境变量 `XY_TAGGED` 中。

| Shell | 使用方式 |
|---|---|
| bash | `cp "${XY_TAGGED[@]}" ~/backup` |
| zsh, fish | `cp $XY_TAGGED ~/backup` |
| sh, dash | 未设置 |

该变量包含进入终端那一刻已标记的文件。若后续标记了更多文件，下次进入终端时变量值会自动更新。

## 使用 eXecute 运行单条命令

按 `X` (eXecute) 会提示输入单条命令行并执行。在命令中，`%t` 代表所有已标记的文件，`%f` 代表光标下的文件：

```
tar czf photos.tar.gz %t
```

[执行](execute.md)列出了其他可用的占位符。

## 异常排查

- **终端首行显示 ` . '…/sync.sh'`。** 这是 Xylem 用于将 shell 切换到目录树所在目录并设置 `XY_TAGGED` 所自动执行的脚本。开头的空格确保其不会被记录进 bash 和 zsh 的历史记录中。
- **Alt+O 没有输入任何内容。** 当前未标记任何文件。
- **shell 没有自动切换目录。** 终端中此前正在运行其他程序。Xylem 仅在 shell 处于命令提示符等待输入时才会自动输入 `cd`。
- **F1 无法打开帮助。** 在终端内部，除 Alt+X、Alt+Z 和 Alt+O 外，所有按键均直接传递给 shell。

另请参阅：[内置终端](terminal.md)、[执行](execute.md)、[标记](tagging.md)。
