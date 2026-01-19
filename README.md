# nuls

![Crates.io](https://img.shields.io/crates/v/nuls)
![License](https://img.shields.io/badge/license-MIT-blue)
![Tests](https://img.shields.io/badge/tests-pass-brightgreen)

<p align="center">
  <img src="public/screenshot.png" alt="nuls screenshot" width="800">
</p>


NuShell-inspired `ls` with a colorful, table-based layout: directory/file type tagging, human-readable sizes, relative “modified” times with recency-driven colors, and familiar flags.

## Features
- Box-drawn table with colored borders and headers
- Directory-first sorting by default; optional `-t/--sort-modified` (newest first) and `-r/--reverse`
- Relative modified column with recency-aware colors (seconds → years, plus future)
- Human-readable sizes (`KB`, `MB`, `GB`, `TB`)
- Hidden files toggled via `-a/--all`
- Colored help output for quick scanning
- Optional git info (`-g`) shown inline after the name, e.g., `main.rs (+15 -2)`

## Install
From crates.io:
```bash
cargo install nuls
```

Building locally:
```bash
cargo install --path . --bin nuls --force
# optional: cargo install --path . --bin nuls --force --root ~/.local
```

Nix:
```bash
# Test with nix run:
nix run github:cesarferreira/nuls
```

```nix
# Install via flake:
inputs = {
  nuls = {
    url = "github:cesarferreira/nuls";
    inputs.nixpkgs.follows = "nixpkgs";
  };
};

# Your output packages + nuls
outputs = {
#  self,
#  nixpkgs,
   nuls,
#  ...
};

# In your configuration.nix
{  inputs, ...}:{
# Your other configurations 
  environment.systemPackages = with pkgs; [
    inputs.nuls.packages.${$system}.default
  ];
}
```

## Usage
```bash
# basic listing
nuls

# include hidden files
nuls -a

# sort by modified (newest first), reverse for oldest first
nuls -t
nuls -tr

# show git status/counts inline
nuls -g
nuls -lag

# combine with hidden and long muscle-memory flag
nuls -la
```

## Flags
- `-a, --all` — show dotfiles
- `-l, --long` — accepted for familiarity (output is already long-form)
- `-t, --sort-modified` — sort by modified time (newest first)
- `-r, --reverse` — reverse sort order
- `-g, --git` — show git status inline (+added/-deleted, `(clean)` when unchanged)
- `--color=always/auto/never` — control ANSI color (default: auto; help is forced color)

## Palette
- Borders/header: teal/green highlights
- Names: dirs blue, files light gray, executables red, dotfiles amber, config/docs yellow
- Modified: green → yellow → orange → red → gray as timestamps get older; blue for future

## Notes
- Directories sort before files unless you use `-t` (modified), in which case recency wins.

## Aliases
Drop one of these in your shell config for muscle-memory:
```bash
# replace ls entirely
alias ls="nuls"

# or keep both
alias nls="nuls"

# with defaults you like
alias lst="nuls -t"
alias lsa="nuls -a"
alias lsat="nuls -at"
```
