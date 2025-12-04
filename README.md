# bin-cpuflags-x86

A small CLI tool to detect CPU flags (instruction sets) of X86 binaries.

## Usage

    $ bin-cpuflags-x86 [<option>...] <file>

| Option             | Description                                                                          |
| ------------------ | ------------------------------------------------------------------------------------ |
| `--mode <mode>`    | Select mode of operation: `detect`, `stats` or `details`. Default value is `detect`. |
| `-s`, `--stats`    | Alias for `--mode stats`. Count instructions used in every feature set.              |
| `-d`, `--details`  | Alias for `--mode details`. List instructions used in every feature set.             |
| `--output <level>` | Select output level: `normal`, `quiet` or `verbose`. Default value is `normal`.      |
| `-q`, `--quiet`    | Alias for `--output quiet`. Print only the result data.                              |
| `-v`, `--verbose`  | Alias for `--output verbose`. Enable more verbose output.                            |
| `-h`, `--help`     | Display help message and exit.                                                       |
| `--`               | Stop reading any options and treat the next argument as a file path.                 |

## Download

You can download prebuilt binaries from the [releases](https://github.com/HanabishiRecca/bin-cpuflags-x86/releases) page.

## Building from the source

**Rust 1.85 or up is required.**

Install the Rust compiler and run:

    $ cargo build --release

## Packages

### crates.io

- [`bin-cpuflags-x86`](https://crates.io/crates/bin-cpuflags-x86)

### AUR

- [`bin-cpuflags-x86`](https://aur.archlinux.org/packages/bin-cpuflags-x86)
- [`bin-cpuflags-x86-bin`](https://aur.archlinux.org/packages/bin-cpuflags-x86-bin)

### CachyOS

- [`bin-cpuflags-x86`](https://packages.cachyos.org/package/cachyos/x86_64/bin-cpuflags-x86)
