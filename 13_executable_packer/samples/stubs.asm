_dl_addr:
    xor rax, rax
    ret

exit:
    xor rdi, rdi
    mov rax, 60
    syscall