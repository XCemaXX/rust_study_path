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

Useful commands to run2:
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
Env: "Ubuntu 22.04.5 LTS". Rust 1.92
Stage1: In nom 8.0 the parsing context is gone. This is acceptable here and is fixed later at stage5.
Stage3: There is no /lib64/ld-2.30.so. On ubuntu I used /lib/x86_64-linux-gnu/ld-linux-x86-64.so.2.
Stage5: Use `RUNPATH` instead of `RPATH`.
Stage9: With glibc 2.35 the binary links against `.plt.sec` instead of `.plt`. This difference is harmless. `nolibc-ifunc | xxd` also produces slightly different output, ending with `.UH..H..M`.
Stage11: Dynamic symbol resolution differs from the classic lazy-binding flow. `_dl_runtime_resolve_xsavec` is not involved. The PLT entry contains `endbr64` followed by a direct bnd jmp via the GOT. The GOT entry is already populated, so symbols are resolved immediately.
stage13: `/bin/ls` requires the DTPMOD64 relocation (type 16). This relocation can be skipped.

stage14: There is no `break _dl_addr`
```
b ptmalloc_init
p &_rtld_global
$1 = (struct rtld_global *) 0x7ffff7ffd040 <_rtld_global>
```
When running `ls` inside `elk`, `_rtld_global` is not initialized correctly and ends up as NULL, leading to a crash inside `__libc_start_main_impl`.
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
Running the same binary normally initializes `_rtld_global` as expected. This difference explains the observed segfaults.
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
At the end of the guide, running `/bin/ls` or `nano` results in a segfault with `r14 = 0`. Manually patching `r14` to point to `_rtld_global` allows execution to continue and reveals a failure later in `setlocale`.
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
stage14.22: `ls` and `nano --help` work, but `nano` itself does not. Tested on Ubuntu 22.04 with glibc 2.35. `_rtld_global` and `setlocale` need to be mocked.
stage14.24: `ls`, `nano --help` work on Ubuntu 24.04 with glibc 2.39. Compressed RELR relocations must be supported.

stage15: `build-std` is required. `rlibc`, `compiler_builtins`, and `#![feature(lang_items)]` complain about missing symbols such as `cmp`, `strlen`, and `bcmp`.

stage17: `samples/what.c` can be used instead of writing a new `hello-pie.c`.
```sh
gcc -static-pie -g what.c -o what-pie
```
stage18: I do not repack PT_LOAD file offsets. Modern toolchains often place the first PT_LOAD at file offset 0 and include critical data (.dynsym, .rela.dyn, .rodata). To preserve the ELF invariant `p_offset % p_align == p_vaddr % p_align`, I keep original p_offset values and only shift vaddr/paddr, copying segments while skipping the prefix occupied by the new ELF header/PHDR. Older binaries typically did not rely on a PT_LOAD@0 with essential data, so the simpler repacking approach happened to work, but it breaks with modern glibc/toolchains.
```sh
wget https://github.com/gohugoio/hugo/releases/download/v0.154.2/hugo_extended_0.154.2_linux-amd64.tar.gz
```