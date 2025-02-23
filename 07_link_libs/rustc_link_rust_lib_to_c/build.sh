#! /bin/bash

rustc some.rs --crate-type cdylib -C panic=abort
gcc main.c -l some -L . -C -Wl,-rpath,. -o add.out 
./add.out
