#!/bin/bash
cargo build --release
cargo build --target=x86_64-pc-windows-gnu --release
mkdir -p breakout_linux
mkdir -p breakout_windows
cp target/release/breakout breakout_linux/
cp target/x86_64-pc-windows-gnu/release/breakout.exe breakout_windows/
cp -r assets breakout_linux/
cp -r assets breakout_windows/