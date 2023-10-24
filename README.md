# bin-cpuflags-x86

A small CLI tool to detect CPU flags (instruction sets) of X86 binaries.

### Usage

    $ bin-cpuflags-x86 [<option>...] <file>

## Options

**`-d`, `--details`**

Enable detailed report about instructions used (slower).

**`-v`, `--verbose`**

Enable more verbose output.

**`-q`, `--quiet`**

Print only the result data.

**`-h`, `--help`**

Display help message and exit.

**`--`**

Stop reading any options and treat the next argument as a file path.

## Building from the source

Install Rust compiler and run:

    $ cargo build --release
