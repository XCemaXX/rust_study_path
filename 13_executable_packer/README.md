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
Env: "Ubuntu 22.04.5 LTS"
Stage1: In nom 8.0 there is no context anymore, but it is fine. It will be fixed on stage5.
Stage3: There is no /lib64/ld-2.30.so. On ubuntu I used /lib/x86_64-linux-gnu/ld-linux-x86-64.so.2.
Stage5: use RUNPATH instead of RPATH
Stage9: In glibc 2.35 I got a link to ".plt.sec" instead of ".plt". But it is fine. I saw more output in nolibc-ifunc | xxd, but got ".UH..H..M" in the end.
Stage11: Different behavior of dynamic symbol resolution. Unlike the classic lazy-binding flow, _dl_runtime_resolve_xsavec is not involved. The PLT entry contains endbr64 followed by a direct bnd jmp through the GOT. The GOT entry is already populated, so the symbol is resolved immediately rather than via the runtime resolver.
stage13: /bin/ls requires DTPMOD64(code: 16) relocation - can be skipped.

stage14: there is no `break _dl_addr`
```
b ptmalloc_init
p &_rtld_global
$1 = (struct rtld_global *) 0x7ffff7ffd040 <_rtld_global>
```
So running `ls` in elk:
```
(gdb) r
(gdb) autosym
(gdb) bt
(gdb) x/5i $rip-10
0x7ffff77e1e40 <__libc_start_main_impl+128>: mov    0x1f0159(%rip),%r15        # 0x7ffff79d1fa0 - global struct
   0x7ffff77e1e47 <__libc_start_main_impl+135>: mov    (%r15),%r14
=> 0x7ffff77e1e4a <__libc_start_main_impl+138>: mov    0xa0(%r14),%rcx
(gdb) x/gx $r15
0x7ffff7689040: 0x0000000000000000
(gdb) x/4gx $r14
0x0:    Cannot access memory at address 0x0
```
Runing just `ls`:
```
(gdb) set stop-on-solib-events 1 # break after libc load
(gdb) run
(gdb) c
(gdb) break *(__libc_start_main_impl+138)
(gdb) c
(gdb) x/gx $r15
0x7ffff7ffd040 <_rtld_global>:  0x00007ffff7ffe2e0 # correct _rtld_global
(gdb) x/4gx $r14
0x7ffff7ffe2e0: 0x0000555555554000      0x00007ffff7ffe888
0x7ffff7ffe2f0: 0x0000555555575a58      0x00007ffff7ffe890
(gdb) dig 0x00007ffff7ffe2e0
Mapped rw-p from File("/usr/lib/x86_64-linux-gnu/ld-linux-x86-64.so.2")
(Map range: 00007ffff7ffd000..00007ffff7fff000, 8 KiB total)
```
In the end of a guide I tried to run `/bin/ls` or `nano`. In gdb after segfault I have r14=0, so lets hack:
``` 
(gdb) run
(gdb) autosym
(gdb) p/x  $r14
$1 = 0x0
(gdb) set $r14 = &_rtld_global
(gdb) continue
(gdb) p/x $rdi
$3 = 0x7ffff7f48d10
(gdb) x/s $rdi
0x7fffffffb8e0: "# Locale name alias data base.\n"
(gdb) bt
#4  __GI_setlocale (category=<optimized out>, locale=<optimized out>) at ./locale/setlocale.c:217
(gdb) x/8i $rip-16
   0x7ffff77f5706 <read_alias_file+278>:        mov    %fs:(%rax),%rsi
   0x7ffff77f570a <read_alias_file+282>:        mov    %rdx,%rax
=> 0x7ffff77f570d <read_alias_file+285>:        testb  $0x20,0x1(%rsi,%rdx,2)
```
stage14.22: `ls`, `nano --help` work but not `nano`. Tested on Ubuntu22 with GLIBC 2.35
Need to mock `_rtld_global` and `setlocale`.
stage14.24: `ls`, `nano --help` work on Ubuntu24 with GLIBC 2.39
Need to add compressed Relr

stage15: need to use "build-std". `rlibc`, `compiler_builtins` and `#![feature(lang_items)]` complain about `cmp`, `strlen`, `bcmp` etc. symbols.

stage17: possible to use `samples/what.c` instead of new hello-pie.c:
```sh
gcc -static-pie -g what.c -o what-pie
```