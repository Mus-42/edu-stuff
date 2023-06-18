#include "expr.h"

#include <stdio.h>

int main() {
    char expr_buf[1024];
    printf(">>> ");
    while (1) {
        fgets(expr_buf, 1024, stdin);
        printf("%lf\n>>> ", cexpr_eval(expr_buf));
    }
}