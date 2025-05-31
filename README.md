# Lunos

A blazingly fast JavaScript runtime written in Rust

## Prerequisites
- [Rust](https://rust-lang.org)
- Linux
  - libwebkit2gtk-4.1-dev: `sudo apt install gir1.2-webkit2-4.1 libwebkit2gtk-4.1-dev`
- macOS
  - Already has the WebKit '.framework' :)
- FreeBSD
  - webkit2gtk4: `sudo pkg install webkit2gtk4`
- Windows
  - Dont use this trash os, but if you insist: [*Yikes*](docs/WINDOWS.md)

## Install
**TPI:**
```sh
tpi install lunos
```

Shell:
```bash
set -o pipefail
git clone https://github.com/LunosRS/lunos /tmp/lunosrs; cd /tmp/lunosrs/
cargo build --release
sudo mv ./target/release/lunos /usr/local/bin
cd $HOME/; rm -rf /tmp/lunosrs
```
What does this command do?
- The first line downloads to the temp directory on your system and enters the folder
- The seccond builds the source code with the fastest config
- The third installs Lunos
- The fourth leaves the temp directory and deletes the
old source code which can reach >3gb after being build!

## License
This project uses the MIT license, see [LICENSE](LICENSE) for more details
