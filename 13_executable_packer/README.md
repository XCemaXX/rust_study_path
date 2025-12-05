[Making our own executable packer](https://fasterthanli.me/series/making-our-own-executable-packer)


Useful commands to build:
```sh
nasm -f elf64 hello.asm
ld hello.o -o hello

gcc samples/entry_point.c -o samples/entry_point

nasm -f elf64 nodata.asm
ld nodata.o -o nodata

nasm -f elf64 hello-pie.asm
ld -pie hello-pie.o -o hello-pie

ld --dynamic-linker /lib64/ld-linux-x86-64.so.2 -pie samples/hello.o -o samples/hello-mov-pie

ld --dynamic-linker /lib64/ld-linux-x86-64.so.2 -pie samples/hello-dl.o samples/msg.o -o samples/hello-dl
ld -shared samples/msg.o -o samples/libmsg.so
ld --dynamic-linker /lib64/ld-linux-x86-64.so.2 -rpath '$ORIGIN' -pie samples/hello-dl.o -L samples -lmsg -o samples/hello-dl
```

Useful commands to run:
```sh
cargo b -p delf -p elk && ./target/debug/elk ./13_executable_packer/samples/nodata
ugdb ./target/debug/elk ./13_executable_packer/samples/hello-mov-pie
gdb --quiet ./13_executable_packer/samples/hello-mov-pie
```