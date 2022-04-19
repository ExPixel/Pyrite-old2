Pyrite
===

GBA emulator in Rust.

## Build

**For target specific instructions:**
- [Windows](#windows)
- [MacOS](#macos)
- [Linux](#linux)

```sh
cargo build             # This will build with optimizations but will keep debug information
                        # and debug assertions.

cargo build --release   # This will build with all optimizations and will strip debug information
                        # and disable debug assertions.

cargo build --profile=profiling # This will build will full optimizations and debug information
                                # but will disable debug assertions.
```

## Run

```sh
cargo run -- --help     # To see all options available.
cargo run -- <ROM>      # To run with the specified GBA ROM file.
```

Key Bindings
---
- `Ctrl+D`: Open the debugger.
- `Ctrl+P`: Pause or unpause the emulator.
- `Ctrl+R`: Reset the GBA.

See [build](#build) section for more information on profiles (`debug`, `release`, `profiling`).


## Linux

The debugger uses [egui](https://github.com/emilk/egui) and [egui_glow](https://github.com/emilk/egui/tree/master/egui_glow)
for rendering which has a few requirements.

For Debian and its derivatives:  
```sh
sudo apt-get install libxcb-render0-dev libxcb-shape0-dev libxcb-xfixes0-dev libspeechd-dev libxkbcommon-dev libssl-dev
```

## Windows

Instructions in [Build](#build) and [Run](#run) sections should just work. If not please [file an issue](/issues).

## MacOS

Instructions in [Build](#build) and [Run](#run) sections should just work. If not please [file an issue](/issues).
