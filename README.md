# bin-cpuflags-x86

A small CLI tool to detect CPU flags (instruction sets) of X86 binaries.

## Usage

    $ bin-cpuflags-x86 [<option>...] <file>

| Option            | Description                                                          |
| ----------------- | -------------------------------------------------------------------- |
| `-d`, `--details` | Enable detailed report about instructions used (slower).             |
| `-v`, `--verbose` | Enable more verbose output.                                          |
| `-q`, `--quiet`   | Print only the result data.                                          |
| `-h`, `--help`    | Display help message and exit.                                       |
| `--`              | Stop reading any options and treat the next argument as a file path. |

## Download

You can download prebuilt binaries from [releases](https://github.com/HanabishiRecca/bin-cpuflags-x86/releases) page.

## Building from the source

Install Rust compiler and run:

    $ cargo build --release

## Packages

### crates.io

-   [`bin-cpuflags-x86`](https://crates.io/crates/bin-cpuflags-x86)

### AUR

-   [`bin-cpuflags-x86`](https://aur.archlinux.org/packages/bin-cpuflags-x86)
-   [`bin-cpuflags-x86-bin`](https://aur.archlinux.org/packages/bin-cpuflags-x86-bin)
