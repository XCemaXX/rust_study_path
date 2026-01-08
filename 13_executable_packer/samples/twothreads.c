#include <unistd.h>
#include <pthread.h>
// #include <errno.h>
#include <stdio.h>

void *in_thread(void *unused) {
    while (1) {
        sleep(1);
    }
}

extern __thread int errno;

int main() {
    printf("errno = %d\n", errno);
    pthread_t t1, t2;
    pthread_create(&t1, NULL, in_thread, NULL);
    pthread_create(&t2, NULL, in_thread, NULL);
    pthread_join(t1, NULL);
    pthread_join(t2, NULL);
}
// gcc -g  samples/twothreads.c -pthread -o samples/twothreads