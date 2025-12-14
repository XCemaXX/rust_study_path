[Making our own executable packer](https://fasterthanli.me/series/making-our-own-executable-packer)


Useful commands to build:
```sh
nasm -f elf64 hello.asm
ld hello.o -o hello

gcc samples/entry_point.c -o samples/entry_point
gcc -nostartfiles -nodefaultlibs samples/nolibc.c -o samples/nolibc

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

cd 13_executable_packer/elk
cargo install --force --path .
```

How to add elk to gdb:
```sh
echo "source /path/to/13_executable_packer/elk/gdb-elk.py > ~/.gdbinit
```

Notes:
Stage1: In nom 8.0 there is no context anymore, but it is fine. It will be fixed on stage5.
Stage3: There is no /lib64/ld-2.30.so. On ubuntu I used /lib/x86_64-linux-gnu/ld-linux-x86-64.so.2.
Stage5: use RUNPATH instead of RPATH
Stage9: In glibc 2.35 I got a link to ".plt.sec" instead of ".plt". But it is fine. I saw more output in nolibc-ifunc | xxd, but got ".UH..H..M" in the end.