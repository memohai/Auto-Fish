# Auto Fish

[English](./README.md) | 中文

Auto Fish 是一个 Android 设备控制服务，配套确定性 CLI 客户端 `af`。

## 源码构建要求

如果你要从源码构建（APK 或 CLI），请先准备：

- JDK 17
- Android SDK（包含 `adb`）
- Rust 工具链（`cargo`）
- `just`

推荐环境变量：

- `ANDROID_HOME` 指向你的 Android SDK 路径

## 快速开始

### 1）在 Android 设备上部署服务

#### 方式 A：安装预编译 APK

安装 APK 并打开 App，然后完成以下步骤：

1. 为 Auto Fish 开启无障碍权限。
2. 在首页打开 **服务** 开关。
3. 记录 App 中显示的连接信息：
   - 设备 IP
   - 端口
   - 令牌

#### 方式 B：本地源码构建并安装

```bash
just build
just install
```

然后按上面相同步骤完成配置。

### 2）安装并使用 `af` CLI

源码构建：

```bash
cd cli
cargo build --release
```

设置环境变量（替换为你的真实值）：

```bash
export AF_URL="http://<设备IP>:<端口>"
export AF_TOKEN="<令牌>"
export AF_DB="./af.db"
```

执行首批命令：

```bash
./target/release/af health
./target/release/af observe top
./target/release/af observe screen --max-rows 80 --fields id,text,desc,resId,flags
./target/release/af observe refs --max-rows 80
./target/release/af act tap --x 540 --y 1200
./target/release/af act tap --by text --value "设置"
```

## 常用 CLI 命令

```bash
af observe screenshot --annotate --max-marks 120
af act swipe 100,1200,900,1200 --duration 300
af act tap --by resid --value "com.android.settings:id/title" --exact-match
af observe refs --max-rows 120
af act tap --by ref --value @n3
af verify text-contains --text "设置"
af verify node-exists --by text --value "设置"
af recover back --times 2
```

说明：

- 如果未设置 `AF_URL`，则必须传 `--url`。
- 对受保护命令，如果未设置 `AF_TOKEN`，则必须传 `--token`。
- 命令输出为单行 JSON。

## 开发者入口

```bash
just check
just build
cd cli && cargo test
```

更多文档：

- [CLI 说明](./cli/README.md)
- [设计文档](./docs/)
