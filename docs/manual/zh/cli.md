# 命令行

```
xy [--cwd-file FILE] [WHERE]
```

## 打开目标

| `WHERE` | 打开目标 |
|---|---|
| 留空 | 当前目录 |
| `DIRECTORY` | 指定目录，作为目录树的根节点 |
| `user@host:` 或 `user@host:PATH` | 通过 SSH 访问的远程主机，定位到用户主目录或 `PATH` |
| `ssh://[user@]host[:port][/PATH]` | 同上，支持指定端口 |
| `docker://[user@]container[/PATH]` | 正在运行的容器，通过 `docker exec` 访问 |

在 Xylem 内部的 `L` (Log) 中支持相同的地址格式。参见[远程主机](remote.md)。

## 选项

| 选项 | 功能说明 |
|---|---|
| `--cwd-file FILE` | 按 Alt+Q 时将当前目录写入 `FILE`。shell 函数读取该文件并切换当前工作目录。[退出](quit.md)提供了该函数。 |
| `--default-config` | 输出包含全部配置选项、默认值及注释说明的配置文件内容。 |
| `--man` | 显示 man 手册页。重定向输出时输出手册页源码。 |
| `-h`, `--help` | 显示选项及配置文件路径。 |
| `-V`, `--version` | 显示版本信息。 |

## 安装 man 手册页

`.deb` 和 `.rpm` 安装包会自动安装它。若通过二进制文件或 Cargo 安装：

```bash
mkdir -p ~/.local/share/man/man1 && xy --man > ~/.local/share/man/man1/xy.1
```

## 生成带注释的配置文件

```bash
mkdir -p ~/.config/xylem && xy --default-config > ~/.config/xylem/config.toml
```

注意：这会覆盖已存在的 `config.toml`。

## 环境变量

| 变量 | 效果 |
|---|---|
| `XDG_CONFIG_HOME` | 从 `$XDG_CONFIG_HOME/xylem/` 读取配置，而非 `~/.config/xylem/`。 |
| `SHELL` | 内置终端使用的 shell。 |
| `VISUAL`, `EDITOR` | 未在配置中指定编辑器时，`E` 命令调用的编辑器。 |
| `XY_DOCKER` | 用于 `docker://` 地址的程序。设置 `XY_DOCKER=podman` 可使用 Podman。 |

在内置终端内部，Xylem 会设置 `XY_TERMINAL=1`，并在 bash、zsh 和 fish 中将已标记的文件存入 `XY_TAGGED`。参见[内置终端](terminal.md)。

另请参阅：[配置](configuration.md)。
