void ftl_exit(int code) {
    __asm__ (
            " \
            mov     %[code], %%edi \n\t\
            mov     $60, %%rax \n\t\
            syscall"
            : // no outputs
            : [code] "r" (code)
            );
}

extern int number;
extern void change_number(void);

void _start(void) {
    change_number();
    change_number();
    change_number();
    ftl_exit(number);
}