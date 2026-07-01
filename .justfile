# Set shell for non-Windows OSs:
set shell := ["zsh", "-c"]

# Set shell for Windows OSs:
set windows-shell := ["powershell.exe", "-NoLogo", "-Command"]

alias b := build-release

build-release:
  cargo fmt
  cargo run -r
