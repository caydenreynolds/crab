#include <stdio.h>

int __c_format_i__(char* buf, long in) {
    return sprintf(buf, "%ld", in);
}
