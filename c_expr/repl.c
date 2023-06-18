#include "expr.h"

#include <stdio.h>

int main() {
    CExprExecutor exec = {0};

    cexpr_executor_init(&exec);

    cexpr_executor_add_functions(&exec, CEXPR_BUILTIN_FUNCTIONS, CEXPR_BUILTIN_FUNCTIONS_COUNT);
    cexpr_executor_add_constants(&exec, CEXPR_BUILTIN_CONSTANTS, CEXPR_BUILTIN_CONSTANTS_COUNT);

    double r = 2;

    cexpr_executor_add_variables(&exec, (CExprVariableBindDesc[1]){{"r", &r}}, 1);

    puts("constants:");
    for (size_t i = 0; i < exec.m_const_size; i++) {
        puts(exec.m_const[i].name);
    }

    puts("variables:");
    for (size_t i = 0; i < exec.m_var_size; i++) {
        puts(exec.m_var[i].name);
    }
    
    puts("functions:");
    for (size_t i = 0; i < exec.m_fun_size; i++) {
        puts(exec.m_fun[i].name);
    }


    CExprParsedExpression expr = cexpr_parse_expression("sqrt(2  )");

    double val = cexpr_executor_eval_expr(&exec, expr);

    printf("value: %f\n", val);

    cexpr_free_expression(expr);
    cexpr_executor_free(&exec);

    puts("end");
}