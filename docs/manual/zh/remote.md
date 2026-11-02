# 远程主机 (SSH)

Xylem 可以通过 SSH 将另一台机器上的文件作为目录树显示出来：既可以单独显示，也可以在 [分屏](split.md) 模式下与本机并排显示。

## 打开远程连接

在 `L`（[Log](treespec.md)）中输入地址，或者在命令行启动 `xy` 时直接指定：

- `user@host:` — 主目录；`user@host:/srv/www` — 指定目录；
- `ssh://[user@]host[:port][/path]` — 指定端口，或省略用户名（使用 `~/.ssh/config` 中的配置）。

Xylem 直接调用系统自带的 `ssh`，因此您在终端中能用的配置在这里同样有效：`~/.ssh/config`（主机别名、端口、ProxyJump）、密钥、ssh-agent 以及 `known_hosts`。无需在远程机器上安装任何额外软件：Xylem 使用所有 SSH 服务器都自带的 SFTP 协议通信（支持 Linux、BSD、macOS、Windows）。

## 密码与验证提示

当 ssh 需要询问信息时，提示会显示在底部状态栏中：

- **密码** 或密钥的 **通行短语 (passphrase)**：输入时不回显；按 Enter 发送，按 Esc 放弃连接。Xylem 只负责将其传递给 ssh，不会保存任何密码。
- **未知主机 (unknown host)**：显示主机名及其密钥指纹，供您在输入 `yes` 前核对（确认后 ssh 会像往常一样将其记录到 `known_hosts` 中）。

连接过程中按 `Esc` 可取消。如果配置了密钥和 ssh-agent，则无需任何手动输入。

![ssh 在 Xylem 底栏询问 deploy@webserver 的密码](../../screenshots/remote-password.png)

## 支持的功能

目录树、文件窗口、[Filespec](filespec.md)、Branch、Showall、标记、[查看器](viewer.md)、[Ctrl+S 与 Ctrl+V](search.md)、自动预览 (Autoview)、Info（Alt+I）、各种显示模式（所有者模式显示 SFTP 返回的用户 ID 和组 ID）、[比较](compare.md) 以及打开（`O` 会下载副本并在本机打开）。

**复制 (Copy)** 可将远程文件下载到本机，并保留其权限和修改日期（精确到秒，与 SFTP 一致）；在分屏状态下，会默认建议将本机窗口作为目标目录。Alt+C（保留路径）同样可用，路径从远程窗口的根目录算起。

![分屏：左侧远程机器上已标记三个备份文件，即将复制到右侧本机的文件夹中](../../screenshots/remote.png)

**重命名**（单个文件、目录，或用掩码重命名已标记的文件）和 **新建目录** 在远程主机上的用法与本机相同。重命名绝不会不经询问就覆盖文件；回答"是"之后，如果服务器支持（OpenSSH 支持），会一步完成替换，否则 Xylem 会提示并保留两个文件。

**复制到远程主机** 也已支持：在本机窗口中操作；分屏时，远程窗口会作为建议的目标；也可以输入远程窗口中的路径，或其地址（`用户@主机:/srv/www`，须位于打开该窗口时所在的文件夹内）。每个文件先写入目标旁边的临时文件，核对大小，设置权限（去掉 setuid 和 setgid）和日期，然后才放到目标位置；按 Esc 会删除临时文件。在远程主机内部复制也是同样的方式。符号链接不会在远程重建，并会给出提示。

**移动** 在各个方向上都可用：从本机移到远程主机再移回（只有在副本已放到位并核对无误后才会删除原文件），以及在远程主机内部移动（即在远程重命名）。**删除**、**Prune** 以及 Usage 中的相同操作也都可用。远程主机上 **没有回收站**：确认提示会说明这一点，Oops 也无可恢复。任何删除都不会穿过符号链接：链接只删除其本身。Prune 会跳过分支内挂载的其他磁盘，这依靠 OpenSSH 的 `statvfs` 扩展来区分；服务器不支持该扩展时，Prune 会被拒绝。

**权限属性** 使用八进制权限（`644`），与 Unix 主机相同，即使从 Windows 操作也是如此；**修改日期** 精确到秒。两者都不会穿过符号链接。[重复文件](duplicates.md) 在远程同样可用。

编辑和执行命令暂不支持。按 F3 重新读取目录；远程目录不会自动刷新。

在远程窗口中按 **F8** 会打开一个本机窗口（位于启动 Xylem 时的文件夹），方便将远程与本地并排操作。

## 远程主机上的终端

在远程窗口中按 **Alt+X** 会在该远程机器上打开一个 shell（`ssh -t`），初始位于当前窗口所在的远程目录；在本地窗口中按 Alt+X 则像以前一样打开本地 shell（切换窗口时，两个 shell 都会在后台保持运行）。在 Linux 和 macOS 上，远程 shell 会复用已建立的 SSH 连接，因此无需再次输入密码；在 Windows 上，ssh 会在终端内提示输入密码。

![远程机器的 shell 位于其目录树下方，执行 cd logs 后，目录树也跟随定位到 logs](../../screenshots/remote-terminal.png)

当远程 shell 通过 OSC 7 序列上报其当前目录时，目录树会自动跟随远程 shell 的 `cd` 移动。许多系统默认已配置好此功能；对于 bash，可在 **远程主机上** 的 `~/.bashrc` 中添加以下内容：

```bash
PROMPT_COMMAND='printf "\033]7;file://%s%s\007" "$HOSTNAME" "$PWD"'
```

Xylem 绝不会向远程 shell 自动输入命令（因为它无法判断远程终端里是否正在运行其他程序），而且 Alt+O 和 `$XY_TAGGED` 仅适用于本地终端。当远程终端打开时退出 Xylem 会先询问确认。

## 登录时自动启动的 tmux 或 screen

有些用户会在远程主机的 `~/.bashrc` 中配置 SSH 登录时自动启动 tmux（或 screen），这样即使连接中断（如 Wi-Fi 不稳定、VPN 重连）也不会终止正在运行的程序，重新登录即可恢复同一会话。例如：

```bash
if [ -n "$SSH_CONNECTION" ] && [ -z "$TMUX" ]; then
    tmux attach-session -t remote 2>/dev/null || tmux new-session -s remote
fi
```

如果这样配置，Xylem 的远程终端也会在 tmux 内部打开。除了 **目录树无法跟随 shell 的 `cd`** 之外，其余功能均正常：因为 tmux 位于 shell 和 Xylem 之间，它会拦截 OSC 7 序列而不向外传递。

Xylem 启动远程 shell 时也会像本地终端一样设置 `XY_TERMINAL=1`，以便登录脚本区分这两种情况。在判断条件中加入对该变量的检查，即可在普通 `ssh` 登录时继续使用 tmux，而在 Xylem 连接时不启动 tmux：

```bash
if [ -n "$SSH_CONNECTION" ] && [ -z "$TMUX" ] && [ -z "$XY_TERMINAL" ]; then
    tmux attach-session -t remote 2>/dev/null || tmux new-session -s remote
fi
```

在 Xylem 内部，shell 不需要靠 tmux 来保活：只要 Xylem 打开着，它就会一直运行；而如果网络连接断开，远程窗口本身也会随之断开。

## 容器（Docker、Podman）

本机上的容器也可以用同样的方式打开，容器内无需 `sshd`：

- `docker://容器` —— 容器的工作目录；`docker://容器/app` —— 该目录；`docker://用户@容器/app` —— 以该用户身份（`docker exec -u`）。

Xylem 运行 `docker exec`，并在容器内启动容器自带的 `sftp-server`，也就是 SSH 服务器所用的同一个程序。大多数镜像没有它：请在容器内安装（Alpine 上 `apk add openssh-sftp-server`，Debian 和 Ubuntu 上 `apt install openssh-sftp-server`，Fedora 上 `dnf install openssh-server`），否则 Xylem 会提示缺少该程序。镜像需要有 `sh`，因此 distroless 镜像不受支持。

上述所有功能在容器中的用法与远程主机相同，**Alt+X** 会在容器内、窗口所在的文件夹中打开 shell。使用 **Podman** 时，请以 `XY_DOCKER=podman` 启动 Xylem。`DOCKER_HOST` 和 docker 上下文照常生效。

Xylem 只会运行 `docker exec`：不会启动、修改或下载任何容器。写入的内容进入容器，但从本机挂载的文件夹（`-v`）除外：那些是本机的文件。

关于 Google Drive、S3 等云端存储，请参阅 [云存储](cloud.md)。
