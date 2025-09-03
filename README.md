# Remi-Lang


<div align="center">
  <img src="./docs/mascot.gif" align="center" />
</div>
<div align="center">
  <span>Art drawn by UnknownRori</span>
</div>

> [!WARNING]
> Don't expect much on this project.

## Introduction

Remi-lang is a esoteric programming language that inspired by the charismatic vampire of the Scarlet devil Remilia Scarlet from [Touhou Project](https://en.wikipedia.org/wiki/Touhou_Project). This programming language can be compiled or intepreted although the available feature might be vary.

### About Touhou Project

Touhou Project is an indie game series (also known as "doujin" in Japanese)
known for its ensemble cast of characters and memorable soundtracks.
It is produced by and large by a single artist known as ZUN, and has a
[permissive license](https://en.touhouwiki.net/wiki/Touhou_Wiki:Copyrights#Copyright_status.2FTerms_of_Use_of_the_Touhou_Project>)

For more information on dōjin culture,
[click here](https://en.wikipedia.org/wiki/D%C5%8Djin).

### Support

|Name            | Core | FFI |
|----------------|------|-----|
|Windows x86_64  |  🔧  |  🔧 |
|Linux x86_64    |  🔧  |  ✖️  |
|JavaScript      |  🔧  |  ✖️  |
|Byte Code       |  ✖️   |  ✖️  |

- ✅ Well Supported
- 🔧 Under construction
- ⚠️  Major Bug/Issue
- ✖️  Not supported yet

- Core: Core feature of the language
- FFI: Allow you to import external shared library

## Quickstart

TODO:

## Dependency

- [Rust](https://rustup.rs/) - Compiler written in
- [Fasm](https://flatassembler.net/) - Assembler that compile assembly code into .o file that the Remi-lang compiler generated
- [gcc](https://gcc.gnu.org/) - Link the .o file into final executable

### Development

```sh
git clone https://github.com/UnknownRori/remi-lang
cd remi-lang

cargo test
```

## License

This project is licensed MIT license.
