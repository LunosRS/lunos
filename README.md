# Lunos

A blazingly fast JavaScript runtime written in Rust

## Prerequisites
- [Rust](https://rust-lang.org)
- Linux
  - libwebkit2gtk-4.1-dev: `sudo apt install gir1.2-webkit2-4.0 libwebkit2gtk-4.1-dev`
- macOS
  - Already has the WebKit '.framework' :)
- FreeBSD
  - webkit2gtk4: `sudo pkg install webkit2gtk4`
- Windows
  - Dont use this trash os, but if you insist: [*Yikes*](.guides/WINDOWS.md)

## Install
```bash
git clone https://github.com/LunosRS/lunos; cd lunos
cargo build --release
sudo mv ./target/release/lunos /usr/local/bin
```

**Note:** This method leverages WSL to run WebKit in a Linux-like environment on Windows. While WebKit does not natively support Windows, this workaround allows for full WebKit development capabilities.

## License
This project uses the MIT license, see [LICENSE](LICENSE) for more details
