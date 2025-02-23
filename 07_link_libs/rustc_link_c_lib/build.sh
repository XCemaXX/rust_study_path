#! /bin/bash

gcc -fPIC -shared -o libsome.so some.c
rustc -l some -L . -C link-args=-Wl,-rpath,. main.rs -o add.out
./add.out

# or without changing rpath
# rustc -l some -L . main.rs -o add.out
# LD_LIBRARY_PATH="$LD_LIBRARY_PATH:." ./add.out
