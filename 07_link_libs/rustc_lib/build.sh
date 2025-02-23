#!/bin/bash

# Build command:
rustc lib_hello.rs --crate-type rlib -o libhello.rlib
rustc exe_hello.rs --extern hello_crate=./libhello.rlib -o hello.out
# run command
./hello.out