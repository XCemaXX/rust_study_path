#!/bin/bash

#release
cargo build --release
cargo run --release

#debug
cargo build
cargo run