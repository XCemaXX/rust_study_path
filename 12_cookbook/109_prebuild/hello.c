
#include <stdio.h>
#include "stdint.h"

int32_t add(int32_t x, int32_t y) {
    return x + y;
}

void hello(const char* name) {
    printf("Hello, %s!\n", name);
}