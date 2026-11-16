# 通过 SSH 从另一台机器复制文件

本指南演示如何在本地机器旁并排展示 `deploy@webserver`，并将其中的三个备份文件复制到本地目录。

## 操作步骤

1. 按 F8。屏幕分屏为两个窗口，均处于当前目录。左侧窗口获得焦点。
2. 按 `L`，输入 `deploy@webserver:` 并按 Enter。左侧窗口随后显示该机器上该用户的主目录。
3. 将光标移至 `backups`，按 → 读取它，然后按 Enter 进入其文件列表。
4. 在需要复制的每个文件上按 `T` 标记。
5. 按 Ctrl+C。提示框自动建议另一个窗口当前所在的目录作为目标路径。
6. 按 Enter。

![六个步骤：F8 分屏、Log 到 deploy@webserver、backups 的文件列表、标记三个文件、Ctrl+C 自动建议本地窗口目录，以及完成复制的三个文件](../../screenshots/howto-ssh-copy.gif)

底部状态栏显示 `Copied 3 of 3 files`。

## 地址格式

| 地址格式 | 打开目标 |
|---|---|
| `user@host:` | 用户主目录 |
| `user@host:/srv/www` | 指定目录 |
| `ssh://user@host:2222/srv/www` | 同上，连接到非默认端口 |
| `docker://container/path` | 正在运行的容器，通过 `docker exec` 访问 |

Xylem 直接运行系统自带的 `ssh`。`~/.ssh/config` 中的主机别名、私钥、ssh-agent 以及 `known_hosts` 均完全适用，且无需在远程机器上安装任何软件。

## 密码与密钥短语

若 ssh 请求输入密码或私钥密码短语，提示将显示在底部状态栏中，输入内容自动隐藏。Xylem 将其直接转交给 ssh，自身绝不存储凭据。对于首次连接的主机，状态栏会展示其密钥指纹并等待输入 `yes`。

## 反向复制

若要将文件复制到远程机器，按 Tab 切换到本地窗口，标记所需文件并按 Ctrl+C：提示框会自动建议远程窗口的目录。Move、Rename、Delete、Makedir、Attributes 和 New date 命令在远程一侧同样可用。

## 异常排查

- **文件窗口显示 `Not logged`。** 目标目录尚未被读取。按 Esc 返回目录树并在其上按 →。步骤 3 即用于执行此读取。
- **Ctrl+C 提示目标路径时未出现任何自动建议。** 屏幕未处于分屏状态。请手动输入本地路径，或先按 F8 开启分屏。
- **菜单中没有 Edit 和 eXecute。** 这两项操作在远程机器上不可用。按 Alt+X 可在该机器上直接打开远程 shell。
- **没有回收站。** 在远程机器上执行删除是永久删除，确认提示中会予以明确说明。

若想日后免去重新输入地址的繁琐，可按 Ctrl+F11 将该目录保存为书签。

另请参阅：[远程主机](remote.md)、[分屏](split.md)、[书签](bookmarks.md)。
