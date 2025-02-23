# lcheck

**lcheck** is a cli license checker for your project dependencies. It autodetects your language, extract dependencies, identifies missing license information, and detects potential license conflicts.

## Features

- **Automatic Dependency Detection** – Scans dependencies from `Cargo.toml`, `pyproject.toml`, and other package files.
- **License Verification** – Retrieves license information from public sources.
- **Conflict Detection** – Highlights incompatible licenses between dependencies.
- **Modular Architecture** – Designed with separate, well-structured modules for each language, ensuring maintainability and easy extensibility..

## Installation

To install **lcheck**, ensure you have Rust installed, then build and install it using:

```sh
git clone https://github.com/Roshan-R/lcheck.git
cd lcheck
cargo build --release
```

To use lcheck globally:

```sh
cargo install --path .
```

### Usage

Run lcheck in your project directory:

```sh
lcheck
```

#### Options

```
cli tool to check license compatibility across your project dependencies

Usage: lcheck [OPTIONS]

Options:
  -v, --verbose  Show verbose output
  -h, --help     Print help
  -V, --version  Print version
```

