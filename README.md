# Path Control

Path Control 是一个跨平台的 PATH 环境变量命令行管理工具。它用 `name` 管理每个 PATH 条目，并支持 `--tips` 记录说明，方便以后知道某个路径为什么被加入。

在 Windows 上，Path Control 通过注册表写入环境变量，避免高级系统设置 GUI 编辑 PATH 时遇到的长度限制。Linux 和 macOS 使用文件型后端生成 shell 配置。

## 功能

- 添加、删除、启用、禁用 PATH 条目
- 给每个 PATH 条目显示稳定 ID，支持像 Docker 一样用短 ID 查看或操作托管条目
- 用 `--name` 给条目起稳定名称，方便管理
- 用 `--tips` 给条目添加说明，方便记忆
- 支持用户级 PATH 和系统级 PATH
- Windows 使用注册表后端
- Linux/macOS 使用 shell profile 文件后端
- 提供 JSON 导出，方便备份或迁移

## 构建

需要先安装 Rust。

使用 Makefile：

```powershell
make build
```

或者直接使用 Cargo：

```powershell
cargo build --release
```

构建后的程序位于：

```text
target/release/pactrl.exe
```

Linux/macOS 下程序名通常是：

```text
target/release/pactrl
```

## 安装

默认安装到当前用户的 Cargo bin 目录，并把命令安装为 `pactrl`：

```powershell
make install
```

Windows 默认安装目录：

```text
%USERPROFILE%\.cargo\bin
```

Linux/macOS 默认安装目录：

```text
~/.cargo/bin
```

自定义安装目录：

```powershell
make install PREFIX=D:\tools\bin
```

自定义命令名称：

```powershell
make install BIN_NAME=pactrl
```

卸载：

```powershell
make uninstall
```

查看 Makefile 支持的命令：

```powershell
make help
```

## 快速开始

添加一个 PATH 条目：

```powershell
pactrl add --name rust "C:\Users\YourName\.cargo\bin" --tips "Rust 命令行工具"
```

查看已启用条目：

```powershell
pactrl list
```

第一次运行时，`list` 也会显示当前系统里已经存在的 PATH 条目。这些条目的 `SOURCE` 会显示为 `external`，表示它们不是由 Path Control 创建，但可以被你看见和核对。外部条目的 ID 根据路径内容生成，因此同一个路径通常会得到同一个 ID。

查看所有条目，包括已禁用的托管条目：

```powershell
pactrl list --all
```

查看某个条目的详细信息：

```powershell
pactrl show rust
pactrl show 018f8a9b12cd
```

禁用某个条目：

```powershell
pactrl disable rust
pactrl disable 018f8a9b12cd
```

重新启用某个条目：

```powershell
pactrl enable rust
pactrl enable 018f8a9b12cd
```

删除某个条目：

```powershell
pactrl remove rust
pactrl remove 018f8a9b12cd
```

导出所有管理条目：

```powershell
pactrl export
```

## 命令说明

### add

添加一个 PATH 条目，并立即应用。

```powershell
pactrl add --name <名称> <路径> --tips <说明>
```

示例：

```powershell
pactrl add --name node "C:\Program Files\nodejs" --tips "Node.js"
```

只保存但暂时不加入 PATH：

```powershell
pactrl add --name temp-tool "D:\tools\temp" --tips "临时工具" --disabled
```

### list

列出 PATH 条目。由 Path Control 管理的条目显示为 `managed`；之前通过其它方式添加的现有 PATH 条目显示为 `external`。两类条目都会显示稳定 `ID`。

```powershell
pactrl list
pactrl list --all
```

条目可以用名称或 ID 前缀查看、启用、禁用、删除，类似 Docker：

```powershell
pactrl show <名称或ID>
pactrl disable <名称或ID>
pactrl enable <名称或ID>
pactrl remove <名称或ID>
```

对于 `external` 条目，工具会在操作时立即和真实 PATH 保持一致：

- `show <external-id>`：查看外部条目。
- `disable <external-id>`：从真实 PATH 移除该路径，并自动接管为一个 `disabled managed` 条目，之后可以再 `enable`。
- `enable <external-id>`：外部条目本来就在 PATH 中，因此不会额外改动。
- `remove <external-id>`：直接从真实 PATH 移除该路径，不保留托管元数据。

### show

查看某个条目。

```powershell
pactrl show <名称>
```

### enable / disable

启用或禁用某个条目，并立即重新应用 PATH。

```powershell
pactrl enable <名称>
pactrl disable <名称>
```

### remove

删除某个条目，并立即从 PATH 中移除。

```powershell
pactrl remove <名称>
```

### apply

根据当前已管理条目重新生成 PATH。

```powershell
pactrl apply
```

### export

以 JSON 格式导出条目。

```powershell
pactrl export
```

## 用户级和系统级 PATH

默认管理用户级 PATH。可以用 `--scope` 明确指定要管理用户环境变量还是系统环境变量：

```powershell
pactrl --scope user list --all
pactrl --scope system list --all
```

如果要管理系统级 PATH，也可以使用快捷参数 `--system`：

```powershell
pactrl --system add --name cmake "C:\Program Files\CMake\bin" --tips "CMake"
pactrl --system list --all
```

注意：Windows 系统级 PATH 写入 `HKEY_LOCAL_MACHINE`，通常需要用管理员权限运行终端。Linux/macOS 系统级配置会写入 `/etc` 下的文件，也通常需要管理员权限。

## 平台行为

### Windows

Windows 后端会写入注册表环境变量：

- 用户级：`HKEY_CURRENT_USER\Environment`
- 系统级：`HKEY_LOCAL_MACHINE\SYSTEM\CurrentControlSet\Control\Session Manager\Environment`

条目元数据保存在：

```text
Software\PathControl
```

写入后程序会广播环境变量变更通知。已经打开的终端可能仍需要重新打开后才能读取新的 PATH。

### Linux/macOS

Linux 和 macOS 后端使用文件保存配置。

用户级条目：

```text
~/.config/path_control/entries_user.json
~/.config/path_control/path_control.sh
```

系统级条目：

```text
/etc/path_control/entries_system.json
/etc/profile.d/path_control.sh
```

如果 shell 没有自动加载生成的 `path_control.sh`，可以在自己的 shell 配置文件中手动 source。

例如 Bash/Zsh：

```sh
source ~/.config/path_control/path_control.sh
```

## 注意事项

- `name` 必须唯一，建议使用简短、稳定、容易记的名称。
- 托管条目的 `ID` 会自动生成；外部条目的 `ID` 根据路径内容生成。
- 每次执行 `list/show/export/apply/enable/disable/remove/add` 时，工具都会读取真实 PATH，并同步托管元数据的启用状态。如果用户手动删除了某个托管路径，它会自动变成 `disabled`；如果用户手动加回同一路径，它会自动变成 `enabled`。
- 对非本工具托管的 `external` 条目执行 `disable/remove` 会立即写回真实 PATH，避免列表显示和系统状态不一致。
- `tips` 只是说明文字，不影响 PATH 行为。
- 修改环境变量后，已经打开的终端可能不会立刻更新，重新打开终端最稳妥。
- 删除或禁用条目只会影响 Path Control 管理过的路径，不会清空其它 PATH 内容。
- 系统级 PATH 修改失败时，请确认终端是否具有管理员权限。
