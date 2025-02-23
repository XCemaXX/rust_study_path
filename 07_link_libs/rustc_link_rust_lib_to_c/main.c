#include "stdint.h"
#include "stdio.h"

int32_t add(int32_t x, int32_t y);

int main(int, char**) {
    int32_t x = 23;
    int32_t y = 42;
    int32_t z = add(x, y);
    printf("%d+%d=%d\n", x, y, z);
    return 0;
}