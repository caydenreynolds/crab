#include <stdio.h>
#include <string.h>

int __c_format_i__(char* buf, long in) {
    return sprintf(buf, "%ld", in);
}

int __c_strlen__(char* cstr) {
    return strlen(cstr);
}

float __c_print_str__(char* str, long len) {
    return printf("%.*s", (int)len, str);
}
