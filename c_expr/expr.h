#pragma once 

#ifndef _C_EXPR_EXPR_H_INCLUDE_
#define _C_EXPR_EXPR_H_INCLUDE_

#define CEXPR_BUILTIN_FUNCTIONS ((CExprFunctionBindDesc*)__cexpr_buitin_functions)
#define CEXPR_BUILTIN_FUNCTIONS_COUNT 20
#define CEXPR_BUILTIN_CONSTANTS ((CExprConstantBindDesc*)__cexpr_buitin_constants)
#define CEXPR_BUILTIN_CONSTANTS_COUNT 2
#define CEXPR_BUILTIN_EXECUTOR (&__cexpr_buitin_executor)

#include <stddef.h>

typedef struct CExprParsedExpression {
    void* root;
} CExprParsedExpression;


typedef struct CExprVariableBindDesc {
    const char* name;
    double* value;
} CExprVariableBindDesc;

typedef struct CExprConstantBindDesc {
    const char* name;
    double value;
} CExprConstantBindDesc;

typedef struct CExprFunctionBindDesc {
    const char* name;
    void* f_ptr;
    short args_count; // up to 3
} CExprFunctionBindDesc;


typedef struct CExprExecutor {
    CExprVariableBindDesc* m_var;
    CExprConstantBindDesc* m_const;
    CExprFunctionBindDesc* m_fun;

    size_t m_var_size;
    size_t m_const_size;
    size_t m_fun_size;

    size_t m_var_capacity;
    size_t m_const_capacity;
    size_t m_fun_capacity;
} CExprExecutor;


#ifdef __cplusplus
extern "C" {
#endif//__cplusplus

// NOTE: expr string must live long enough due to eval expression 
CExprParsedExpression cexpr_parse_expression(const char* expr_str);
CExprParsedExpression cexpr_parse_expression_len(const char* expr_str, size_t str_len);

void cexpr_free_expression(CExprParsedExpression expr);

void cexpr_executor_init(CExprExecutor* exec);
void cexpr_executor_free(CExprExecutor* exec);

void cexpr_executor_add_variables(CExprExecutor* exec, const CExprVariableBindDesc* var_desc, size_t var_desc_count);
void cexpr_executor_add_constants(CExprExecutor* exec, const CExprConstantBindDesc* const_desc, size_t const_desc_count);
void cexpr_executor_add_functions(CExprExecutor* exec, const CExprFunctionBindDesc* fun_desc, size_t fun_desc_count);

double cexpr_executor_eval_expr(const CExprExecutor* exec, CExprParsedExpression expr);

// using builtin executor:
double cexpr_eval_expr(CExprParsedExpression expr);
double cexpr_eval(const char* expr_str);

// TODO some way to check that expr is correct? (correctly parsed) (no nullptr nodes)

extern const CExprFunctionBindDesc __cexpr_buitin_functions[CEXPR_BUILTIN_FUNCTIONS_COUNT];
extern const CExprConstantBindDesc __cexpr_buitin_constants[CEXPR_BUILTIN_CONSTANTS_COUNT];
extern const CExprExecutor __cexpr_buitin_executor;

#ifdef __cplusplus
}
#endif//__cplusplus

#endif//_C_EXPR_EXPR_H_INCLUDE_