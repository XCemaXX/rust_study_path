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

int ftl_strlen(char *s) {
    int len = 0;
    while (s[len]) {
        len++;
    }
    return len;
}

void ftl_print(char *msg) {
    int len = ftl_strlen(msg);

    __asm__ (
            " \
            mov      $1, %%rdi \n\t\
            mov      %[msg], %%rsi \n\t\
            mov      %[len], %%edx \n\t\
            mov      $1, %%rax \n\t\
            syscall"
            : // no outputs
            // inputs
            : [msg] "r" (msg), [len] "r" (len)
            );
}

char *get_msg_root() {
    return "Hello, root!\n";
}

char *get_msg_user() {
    return "Hello, regular user!\n";
}

typedef char *(*get_msg_t)();

static get_msg_t resolve_get_msg() {
    int uid;

    // `getuid` syscall
    __asm__ (
            " \
            mov     $102, %%rax \n\t\
            syscall \n\t\
            mov     %%eax, %[uid]"
            : [uid] "=r" (uid)
            : // no inputs
            );

    if (uid == 0) {
        return get_msg_root;
    } else {
        return get_msg_user;
    }
}

char *get_msg() __attribute__ ((ifunc ("resolve_get_msg")));

int main() {
    ftl_print(get_msg());
    return 0;
}

void _start() {
    ftl_exit(main());
}