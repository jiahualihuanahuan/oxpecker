# Oxpecker

A fast native macOS CLI that finds high-memory, low-CPU background processes you own and can optionally terminate them. Written in Rust.

macOS keeps a lot of memory cached on purpose. Oxpecker helps you surgically clean up stalled helpers, abandoned Electron processes, and other memory hogs without rebooting or installing a full cleaner app.

## Features

- Dry-run by default (never kills unless you pass `--kill`)
- Only touches processes owned by your user
- Accurate idle detection (double-samples CPU)
- Configurable memory and CPU thresholds
- Optional name exclusion list
- Excludes itself automatically

## Install

### Cargo
```bash
cargo install --git https://github.com/jiahualihuanahuan/oxpecker
```

### From source
```bash
git clone https://github.com/jiahualihuanahuan/oxpecker.git
cd oxpecker
cargo build --release
./target/release/oxpecker --help
```

## Usage

List processes using more than 50 MB and ≤ 1% CPU (default):
```bash
oxpecker
```

Higher memory threshold:
```bash
oxpecker --mem-threshold-mb 200
```

Actually kill them:
```bash
oxpecker --mem-threshold-mb 100 --kill
```

Skip specific names:
```bash
oxpecker --exclude "Google Chrome,Code"
```

## Disclaimer

Killing processes can lose unsaved data in those apps. Use `--kill` carefully. The tool only acts on processes owned by your user.

## License

MIT
