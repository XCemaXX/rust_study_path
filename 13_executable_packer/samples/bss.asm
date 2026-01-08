global _start
section .text

_start: ; load address of `zero`, for debugging purposes
        mov rax, zero ; extra for bss3. next lines doesn't matter
        lea rax, [rel zero] ;bss 
        mov rax, [rax] ; extra for bbs2

        ; then just exit.
        xor rdi, rdi
        mov rax, 60
        syscall

        section .bss

pad:    resq 65536
zero:   resq 16