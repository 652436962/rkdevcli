# rkdevcli

[![CI](https://github.com/652436962/rkdevcli/actions/workflows/ci.yml/badge.svg)](https://github.com/652436962/rkdevcli/actions/workflows/ci.yml)
[![License: GPL-3.0](https://img.shields.io/badge/license-GPL--3.0--only-blue.svg)](LICENSE)

面向 Rockchip Maskrom / Loader 设备的原生烧录命令行工具，使用 Rust + [rockusb](https://crates.io/crates/rockusb) + [nusb](https://crates.io/crates/nusb) 直接访问 USB，运行时不依赖 `rkdeveloptool`、Python 或 Qt。

本仓库同时提供可复用的引擎库 `rkdev`：桌面版 [RKDevSuite](https://github.com/652436962/rkdevsuite) 以 git 依赖方式复用它，USB 枚举、固件解析、分区、烧录与任务状态逻辑只有一份实现。

## 下载

到 [Releases](https://github.com/652436962/rkdevcli/releases) 下载对应平台的压缩包，解压即可运行，无需安装：

| 平台 | 文件 |
| --- | --- |
| Linux x86_64（glibc 2.35+） | `rkdevcli-<版本>-linux-x64.tar.gz` |
| Debian / Ubuntu x86_64 | `rkdevcli_<版本>_amd64.deb` |
| macOS Apple Silicon | `rkdevcli-<版本>-macos-arm64.tar.gz` |
| Windows x64 | `rkdevcli-<版本>-windows-x64.zip` |

每个 Release 附带 `SHA256SUMS`。Debian / Ubuntu 也可以直接安装 deb（含命令与 udev 规则）：

```bash
sudo apt install ./rkdevcli_0.1.1_amd64.deb
rkdevcli scan
```

也可以从源码构建，见下文。

## 功能

- `scan`：枚举 RockUSB 设备，区分 Maskrom / Loader，Loader 模式附带芯片型号与容量
- `inspect`：解析 RKFW、RKAF、RAW、MiniLoader 与受支持的压缩包，不触发写入
- `unpack`：校验容器后解包整包固件到新目录，失败时清理临时输出
- `flash`：按整包/RAW/分区写入，支持从 Maskrom 下载匹配的 Loader
- `serve`：仅监听 127.0.0.1 的 MCP 服务，默认只读，`--allow-flash` 才允许烧录
- `maintenance`：Loader 容器重打包、稀疏镜像、GPT 读写等维护操作

## 构建

需要 Rust 1.88+。

```bash
cargo build --release
./target/release/rkdevcli --help
```

生成的二进制：

| 二进制 | 说明 |
| --- | --- |
| `rkdevcli` | 主命令行工具 |
| `rkdev-maintenance` | 独立的维护命令（也可通过 `rkdevcli maintenance` 调用） |

桌面版需要的可选 feature 为 `desktop`，默认不编译：

```bash
cargo check --features desktop
```

## 使用示例

```bash
# 查看设备
rkdevcli scan

# 检查固件
rkdevcli inspect update.img

# 从 Maskrom 烧录整包（先下载内置 Loader，再写入）
rkdevcli flash update.img --device <ID> --allow-flash

# 只烧录分区 rootfs
rkdevcli flash update.img --device <ID> --partition rootfs --allow-flash --reboot

# 解包到指定目录下新建的子目录
rkdevcli unpack update.img ./out

# 启动只读 MCP 服务，供支持 MCP 的客户端调用
rkdevcli serve --state-dir ~/.local/state/rkdevcli
```

`scan` 与 `inspect` 输出 JSON，便于脚本处理。烧录会校验目标设备、镜像大小和分区边界；写入中断可能导致设备无法启动，请先备份。

## Linux USB 权限

普通用户访问开发板需要 udev 规则授权 Rockusb 接口（VID `2207`）。不要用 root 运行本工具。deb 包自带 `/usr/lib/udev/rules.d/99-rkdevcli.rules`；安装时开发板已插着的话，执行 `sudo udevadm control --reload-rules && sudo udevadm trigger --subsystem-match=usb` 或重新插拔生效。使用压缩包时手动创建同样的规则：

```udev
SUBSYSTEM=="usb", ATTR{idVendor}=="2207", MODE="0666", TAG+="uaccess"
```

## 与上游的关系

本仓库派生自 [FantasyGmm/rkdevtool-next](https://github.com/FantasyGmm/rkdevtool-next)（GPL-3.0-only），把原本与 Tauri 桌面端耦合的引擎拆为可独立构建的库与 CLI。上游的作者与贡献者信息见 [ACKNOWLEDGEMENTS.md](ACKNOWLEDGEMENTS.md)。桌面界面见 [RKDevSuite](https://github.com/652436962/rkdevsuite)。

## 许可

GPL-3.0-only，见 [LICENSE](LICENSE)。
