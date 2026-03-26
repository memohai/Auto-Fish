# Auto Fish

English | [中文](./README_ZH.md)

Auto Fish is an Android device control service with a deterministic CLI client (`af`).

## Build from source requirements

If you want to build from source (APK or CLI), prepare:

- JDK 17
- Android SDK (with `adb`)
- Rust toolchain (`cargo`)
- `just`

Recommended environment:

- `ANDROID_HOME` points to your Android SDK path

## Quickstart

### 1) Deploy service on Android device

#### Option A: install a prebuilt APK

Install the APK, open the app, then do the following:

1. Enable accessibility permission for Auto Fish.
2. In Home page, turn on **Service**.
3. Note the service connection info shown in app:
   - Device IP
   - Port
   - Token

#### Option B: install from local source build

```bash
just build
just install
```

Then follow the same 3 steps above.

### 2) Install and use `af` CLI

Build from source:

```bash
cd cli
cargo build --release
```

Set environment variables (replace with your actual values):

```bash
export AF_URL="http://<DEVICE_IP>:<PORT>"
export AF_TOKEN="<TOKEN>"
export AF_DB="./af.db"
```

Run first commands:

```bash
./target/release/af health
./target/release/af observe top
./target/release/af observe screen --max-rows 80 --fields id,text,desc,resId,flags
./target/release/af act tap --x 540 --y 1200
```

## Common CLI commands

```bash
af observe screenshot --annotate --max-marks 120
af act swipe 100,1200,900,1200 --duration 300
af verify text-contains --text "Settings"
af verify node-exists --by text --value "Settings"
af recover back --times 2
```

Notes:

- `--url` is required unless `AF_URL` is set.
- `--token` is required for protected commands unless `AF_TOKEN` is set.
- Command output is JSON (single line per command).

## For developers

```bash
just check
just build
cd cli && cargo test
```

More docs:

- [CLI details](./cli/README.md)
- [Design docs](./docs/)
