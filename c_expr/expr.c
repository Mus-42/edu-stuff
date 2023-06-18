#include "expr.h"
#include "ast.h"

#include <string.h>
#include <math.h>
#include <memory.h>
#include <ctype.h>
#include <assert.h>
#include <stdbool.h>
#include <stdlib.h>

static void cexpr_free_expression_impl(void* expr);
static double cexpr_eval_expression_impl(CExprExecutor* exec, void* expr);
static void* cexpr_parce_impl(const char* str, size_t len);

const CExprFunctionBindDesc __cexpr_buitin_functions[CEXPR_BUILTIN_FUNCTIONS_COUNT] = {
    {"sqrt", (void*)&sqrt, 1}
};
const CExprConstantBindDesc __cexpr_buitin_constants[CEXPR_BUILTIN_CONSTANTS_COUNT] = {
    {"pi", 3.14159265358979323846}
};
const CExprExecutor __cexpr_buitin_executor = {
    0, CEXPR_BUILTIN_CONSTANTS, CEXPR_BUILTIN_FUNCTIONS,
    0, CEXPR_BUILTIN_CONSTANTS_COUNT, CEXPR_BUILTIN_FUNCTIONS_COUNT,
    0, CEXPR_BUILTIN_CONSTANTS_COUNT, CEXPR_BUILTIN_FUNCTIONS_COUNT
};

CExprParsedExpression cexpr_parse_expression(const char* expr_str) {
    return cexpr_parse_expression_len(expr_str, strlen(expr_str));
}
CExprParsedExpression cexpr_parse_expression_len(const char* expr_str, size_t str_len) {
    assert(expr_str != 0);
    
    void* expr = cexpr_parce_impl(expr_str, str_len);

    CExprParsedExpression ret = {expr};
    return ret;
}

void cexpr_free_expression(CExprParsedExpression expr) {
    cexpr_free_expression_impl(expr.root);
}

void cexpr_executor_init(CExprExecutor* exec) {
    memset(exec, 0, sizeof(CExprExecutor));
}
void cexpr_executor_free(CExprExecutor* exec) {
    if (exec->m_var) 
        free(exec->m_var);
    if (exec->m_const) 
        free(exec->m_const);
    if (exec->m_fun) 
        free(exec->m_fun);

    memset(exec, 0, sizeof(CExprExecutor));
}

//TODO add checks for add_* functions?

void cexpr_executor_add_variables(CExprExecutor* exec, const CExprVariableBindDesc* var_desc, size_t var_desc_count) {
    if (!var_desc_count || !var_desc) return;

    if (exec->m_var_capacity < exec->m_var_size + var_desc_count || !exec->m_var) {
        // resize
        size_t new_size = exec->m_var_capacity * 2;
        if (new_size < 64)  
            new_size = 64;
        if (new_size < exec->m_var_size + var_desc_count)  
            new_size = exec->m_var_size + var_desc_count;
        
        CExprVariableBindDesc* new_vars = (CExprVariableBindDesc*)malloc(new_size * sizeof(CExprVariableBindDesc));
        if (exec->m_var) {
            memcpy(new_vars, exec->m_var, exec->m_var_size * sizeof(CExprVariableBindDesc));
            free(exec->m_var);
        }
        exec->m_var = new_vars;
    }

    memcpy(exec->m_var + exec->m_var_size, var_desc, var_desc_count * sizeof(CExprVariableBindDesc));
    exec->m_var_size += var_desc_count;
}
void cexpr_executor_add_constants(CExprExecutor* exec, const CExprConstantBindDesc* const_desc, size_t const_desc_count) {
    if (!const_desc_count || !const_desc) return;

    if (exec->m_const_capacity < exec->m_const_size + const_desc_count || !exec->m_const) {
        // resize
        size_t new_size = exec->m_const_capacity * 2;
        if (new_size < 64)  
            new_size = 64;
        if (new_size < exec->m_const_size + const_desc_count)  
            new_size = exec->m_const_size + const_desc_count;
        
        CExprConstantBindDesc* new_consts = (CExprConstantBindDesc*)malloc(new_size * sizeof(CExprConstantBindDesc));
        if (exec->m_const) {
            memcpy(new_consts, exec->m_const, exec->m_const_size * sizeof(CExprConstantBindDesc));
            free(exec->m_const);
        }
        exec->m_const = new_consts;
        exec->m_const_capacity = new_size;
    }

    memcpy(exec->m_const + exec->m_const_size, const_desc, const_desc_count * sizeof(CExprConstantBindDesc));
    exec->m_const_size += const_desc_count;
}
void cexpr_executor_add_functions(CExprExecutor* exec, const CExprFunctionBindDesc* fun_desc, size_t fun_desc_count) {
    if (!fun_desc_count || !fun_desc) return;

    if (exec->m_fun_capacity < exec->m_fun_size + fun_desc_count || !exec->m_fun) {
        // resize
        size_t new_size = exec->m_fun_capacity * 2;
        if (new_size < 64)  
            new_size = 64;
        if (new_size < exec->m_fun_size + fun_desc_count)  
            new_size = exec->m_fun_size + fun_desc_count;
        
        CExprFunctionBindDesc* new_funs = (CExprFunctionBindDesc*)malloc(new_size * sizeof(CExprFunctionBindDesc));
        if (exec->m_fun) {
            memcpy(new_funs, exec->m_fun, exec->m_fun_size * sizeof(CExprFunctionBindDesc));
            free(exec->m_fun);
        }
        exec->m_fun = new_funs;
    }

    memcpy(exec->m_fun + exec->m_fun_size, fun_desc, fun_desc_count * sizeof(CExprFunctionBindDesc));
    exec->m_fun_size += fun_desc_count;
}

double cexpr_executor_eval_expr(const CExprExecutor* exec, CExprParsedExpression expr) {
    return cexpr_eval_expression_impl(exec, expr.root);
}

double cexpr_eval_expr(CExprParsedExpression expr) {
    return cexpr_executor_eval_expr(CEXPR_BUILTIN_EXECUTOR, expr);
}
double cexpr_eval(const char* expr_str) {
    CExprParsedExpression expr = cexpr_parse_expression(expr_str);
    double ret = cexpr_eval_expr(expr);
    cexpr_free_expression(expr);
    return ret;
}


// parser & evaluator implementation

static void cexpr_free_expression_impl(void* expr) {
    if (!expr) return;
    switch (((CExprNodeCommon*)expr)->node_type) {
    case CEXPR_NODE_NONE: 
        break;
    case CEXPR_NODE_OPERATOR: 
        cexpr_free_expression_impl(((CExprOperatorNode*)expr)->l_node);
        cexpr_free_expression_impl(((CExprOperatorNode*)expr)->r_node);
        break;
    case CEXPR_NODE_LITERAL:  
        break;
    case CEXPR_NODE_VARIABLE: 
        break;

    case CEXPR_NODE_FUNCTION_CALL0:
        break;
    case CEXPR_NODE_FUNCTION_CALL1:
        cexpr_free_expression_impl(((CExprFunctionCall1Node*)expr)->arg0);
        break;
    case CEXPR_NODE_FUNCTION_CALL2:
        cexpr_free_expression_impl(((CExprFunctionCall2Node*)expr)->arg0);
        cexpr_free_expression_impl(((CExprFunctionCall2Node*)expr)->arg1);
        break;
    case CEXPR_NODE_FUNCTION_CALL3:
        cexpr_free_expression_impl(((CExprFunctionCall3Node*)expr)->arg0);
        cexpr_free_expression_impl(((CExprFunctionCall3Node*)expr)->arg1);
        cexpr_free_expression_impl(((CExprFunctionCall3Node*)expr)->arg2);
        break;
    
    default: 
        break;
    }  
    free(expr);
}

static void* cexpr_find_function_by_name(CExprExecutor* exec, size_t args_count, const char* fn_name, size_t name_len) {
    if (!exec || !fn_name || !name_len) return 0;
    for (size_t i = 0; i < exec->m_fun_size; i++) {
        if (args_count != exec->m_fun[i].args_count) continue;
        bool is_ok = true;
        const char* cur_fn = exec->m_fun[i].name;
        for (size_t j = 0; j < name_len; j++)
            if (!cur_fn[j] || cur_fn[j] != fn_name[j]) {
                is_ok = false;
                break;
            }
        if (is_ok && !cur_fn[name_len]) return exec->m_fun[i].f_ptr;
    }
    return 0;
}

static double* cexpr_find_variable_by_name(CExprExecutor* exec, const char* var_name, size_t name_len) {
    if (!exec || !var_name || !name_len) return 0;
    for (size_t i = 0; i < exec->m_var_size; i++) {
        bool is_ok = true;
        const char* cur_var = exec->m_var[i].name;
        for (size_t j = 0; j < name_len; j++)
            if (!cur_var[j] || cur_var[j] != var_name[j]) {
                is_ok = false;
                break;
            }
        if (is_ok && !cur_var[name_len]) return exec->m_var[i].value;
    }
    return 0;
}
static double cexpr_find_counstant_by_name(CExprExecutor* exec, const char* const_name, size_t name_len) {
    if (!exec || !const_name || !name_len) return NAN;
    for (size_t i = 0; i < exec->m_const_size; i++) {
        bool is_ok = true;
        const char* cur_const = exec->m_const[i].name;
        for (size_t j = 0; j < name_len; j++)
            if (!cur_const[j] || cur_const[j] != const_name[j]) {
                is_ok = false;
                break;
            }
        if (is_ok && !cur_const[name_len]) return exec->m_const[i].value;
    }
    return NAN;
}

static double cexpr_eval_expression_impl(CExprExecutor* exec, void* expr) {
    if (!expr) return NAN;
    switch (((CExprNodeCommon*)expr)->node_type) {
    case CEXPR_NODE_NONE: 
        break;
    case CEXPR_NODE_OPERATOR: 
        double l = cexpr_eval_expression_impl(exec, ((CExprOperatorNode*)expr)->l_node);
        double r = cexpr_eval_expression_impl(exec, ((CExprOperatorNode*)expr)->r_node);
        switch (((CExprOperatorNode*)expr)->operator_type) {
        case CEXPR_OPERATOR_ADD:
            return l+r;
        case CEXPR_OPERATOR_SUB:
            return l-r;
        case CEXPR_OPERATOR_MUL:
            return l*r;
        case CEXPR_OPERATOR_DIV:
            return l/r;
        case CEXPR_OPERATOR_PLUS:
            return +l;
        case CEXPR_OPERATOR_MINUS:
            return -l;
        default:
            return NAN;
        }
    case CEXPR_NODE_LITERAL:  
        return ((CExprNumericLiteralNode*)expr)->value;
    case CEXPR_NODE_VARIABLE: 
    {
        const char* name = ((CExprVariableOrConstantNode*)expr)->variable_name;
        size_t len = ((CExprVariableOrConstantNode*)expr)->name_len;
        double value = NAN;
        double* var_ptr = cexpr_find_variable_by_name(exec, name, len);
        if (var_ptr) value = *var_ptr;
        else value = cexpr_find_counstant_by_name(exec, name, len);
        
        //printf("var: %.*s, val = %lf\n", (int)len, name, value);
        return value;
    }

    case CEXPR_NODE_FUNCTION_CALL0:
    {
        CExprFunctionCall0Node* cur_node = (CExprFunctionCall0Node*)expr;
        void* cur_fn = cexpr_find_function_by_name(exec, 0, cur_node->function_name, cur_node->name_len);
        if (!cur_fn) return NAN;
        return ((double(*)())cur_fn)();
    }
    case CEXPR_NODE_FUNCTION_CALL1:
    {
        CExprFunctionCall0Node* cur_node = (CExprFunctionCall0Node*)expr;
        void* cur_fn = cexpr_find_function_by_name(exec, 1, cur_node->function_name, cur_node->name_len);
        if (!cur_fn) return NAN;
        double arg0 = cexpr_eval_expression_impl(exec, ((CExprFunctionCall1Node*)expr)->arg0);
        //printf("f(%lf) = %lf\n", arg0, ((double(*)(double))cur_fn)(arg0));
        return ((double(*)(double))cur_fn)(arg0);
    }
    case CEXPR_NODE_FUNCTION_CALL2: 
    {
        CExprFunctionCall0Node* cur_node = (CExprFunctionCall0Node*)expr;
        void* cur_fn = cexpr_find_function_by_name(exec, 2, cur_node->function_name, cur_node->name_len);
        if (!cur_fn) return NAN;
        double arg0 = cexpr_eval_expression_impl(exec, ((CExprFunctionCall2Node*)expr)->arg0);
        double arg1 = cexpr_eval_expression_impl(exec, ((CExprFunctionCall2Node*)expr)->arg1);
        return ((double(*)(double, double))cur_fn)(arg0, arg1);
    }
    case CEXPR_NODE_FUNCTION_CALL3: 
    {
        CExprFunctionCall0Node* cur_node = (CExprFunctionCall0Node*)expr;
        void* cur_fn = cexpr_find_function_by_name(exec, 3, cur_node->function_name, cur_node->name_len);
        if (!cur_fn) return NAN;
        double arg0 = cexpr_eval_expression_impl(exec, ((CExprFunctionCall3Node*)expr)->arg0);
        double arg1 = cexpr_eval_expression_impl(exec, ((CExprFunctionCall3Node*)expr)->arg1);
        double arg2 = cexpr_eval_expression_impl(exec, ((CExprFunctionCall3Node*)expr)->arg2);
        return ((double(*)(double, double, double))cur_fn)(arg0, arg1, arg2);
    }
    
    default: 
        return NAN;
    } 
}

typedef struct CExprTokenizer {
    const char* str_beg;
    const char* str_cur;
    const char* str_end;

    int cur_token_type;
    char char_token_value;
    bool is_valid;

    size_t id_token_length;
    const char* id_token_begin;

    double number_token_value;
} CExprTokenizer;

enum CExprTokenType {
    TK_INVALID = 0,

    TK_CHAR, // all single char tokens
    TK_NUMBER,
    TK_ID,

    TK_EOF
};

static void cexpr_parse_next_token(CExprTokenizer* tk) {
    while (tk->str_cur < tk->str_end && isspace(*tk->str_cur)) tk->str_cur++;
    if (tk->str_cur == tk->str_end) {
        tk->cur_token_type = TK_EOF;
        return;
    }
    // non space ch -> token begin or invalid
    switch (*tk->str_cur) {
    case '+': case '-': case '*': case '/': case ',': case '(': case ')':
        tk->cur_token_type = TK_CHAR;
        tk->char_token_value = *tk->str_cur;
        tk->str_cur++;
        break;

    case '.':
    case '0': case '1': case '2': case '3': case '4': 
    case '5': case '6': case '7': case '8': case '9': 
        tk->cur_token_type = TK_NUMBER;
        // TODO string can be non-null terminated. replace with custom function
        tk->number_token_value = strtod(tk->str_cur, &tk->str_cur); 
        break;
    
    default:
        if (isalpha(*tk->str_cur) || *tk->str_cur == '_') {
            tk->id_token_begin = tk->str_cur;
            tk->cur_token_type = TK_ID;
            tk->str_cur++;
            while (tk->str_cur < tk->str_end && (isalnum(*tk->str_cur) || *tk->str_cur == '_')) tk->str_cur++;
            tk->id_token_length = (size_t)(tk->str_cur - tk->id_token_begin);
            break;
        }
            tk->cur_token_type = TK_INVALID;
        break;
    }
}

static void* cexpr_parce_3_level(CExprTokenizer* tk);

// number | brackets | variable | function call 
static void* cexpr_parce_0_level(CExprTokenizer* tk) {
    if (tk->cur_token_type == TK_NUMBER) {
        CExprNumericLiteralNode* lit_node = malloc(sizeof(CExprNumericLiteralNode));
        lit_node->common.node_type = CEXPR_NODE_LITERAL;
        lit_node->value = tk->number_token_value;

        cexpr_parse_next_token(tk);
        return lit_node;
    }
    if (tk->cur_token_type == TK_CHAR && tk->char_token_value == '(') {
        cexpr_parse_next_token(tk);
        void* node = cexpr_parce_3_level(tk);
        if (tk->cur_token_type == TK_CHAR && tk->char_token_value == ')') {
            cexpr_parse_next_token(tk);
        } else {
            tk->is_valid = false;
        }
        return node;
    }
    if (tk->cur_token_type == TK_ID) {
        const char* name = tk->id_token_begin;
        const char* len = tk->id_token_length;
        cexpr_parse_next_token(tk);
        // function
        if (tk->cur_token_type == TK_CHAR && tk->char_token_value == '(') {
            void* args[3];
            size_t args_count = 0;
            cexpr_parse_next_token(tk);

            while (true) {
                args[args_count++] = cexpr_parce_3_level(tk);

                if (tk->cur_token_type == TK_CHAR && tk->char_token_value == ',') {
                    if (args_count == 3) {
                        tk->is_valid = false;
                        break;
                    } 
                    cexpr_parse_next_token(tk);
                    continue;
                }
                if (tk->cur_token_type == TK_CHAR && tk->char_token_value == ')') {
                    cexpr_parse_next_token(tk);
                    break;
                }
                tk->is_valid = false;
                break;
            }

            
            switch (args_count) {
            case 0: 
            {
                CExprFunctionCall0Node* fn_node = malloc(sizeof(CExprFunctionCall0Node));
                fn_node->common.node_type = CEXPR_NODE_FUNCTION_CALL0;
                fn_node->function_name = name;
                fn_node->name_len = len;
                return fn_node;
            }
            case 1: 
            {
                CExprFunctionCall1Node* fn_node = malloc(sizeof(CExprFunctionCall1Node));
                fn_node->common.node_type = CEXPR_NODE_FUNCTION_CALL1;
                fn_node->function_name = name;
                fn_node->name_len = len;
                fn_node->arg0 = args[0];
                return fn_node;
            }
            case 2: 
            {
                CExprFunctionCall2Node* fn_node = malloc(sizeof(CExprFunctionCall2Node));
                fn_node->common.node_type = CEXPR_NODE_FUNCTION_CALL2;
                fn_node->function_name = name;
                fn_node->name_len = len;
                fn_node->arg0 = args[0];
                fn_node->arg1 = args[1];
                return fn_node;
            }
            case 3: 
            {
                CExprFunctionCall3Node* fn_node = malloc(sizeof(CExprFunctionCall3Node));
                fn_node->common.node_type = CEXPR_NODE_FUNCTION_CALL3;
                fn_node->function_name = name;
                fn_node->name_len = len;
                fn_node->arg0 = args[0];
                fn_node->arg1 = args[1];
                fn_node->arg2 = args[2];
                return fn_node;
            }
            }
        } else {
            CExprVariableOrConstantNode* var_node = malloc(sizeof(CExprVariableOrConstantNode));
            var_node->common.node_type = CEXPR_NODE_VARIABLE;
            var_node->variable_name = name;
            var_node->name_len = len;
            return var_node;
        }
    }

    tk->is_valid = false;
    return 0;
}

// unary operators + | - 
static void* cexpr_parce_1_level(CExprTokenizer* tk) {
    if (tk->cur_token_type == TK_CHAR && (tk->char_token_value == '+' || tk->char_token_value == '-')) {
        int op_type = tk->char_token_value == '+' ? CEXPR_OPERATOR_PLUS : CEXPR_OPERATOR_MINUS;
        cexpr_parse_next_token(tk);
        CExprOperatorNode* ret = malloc(sizeof(CExprOperatorNode));
        ret->common.node_type = CEXPR_NODE_OPERATOR;
        ret->operator_type = op_type;
        ret->l_node = cexpr_parce_0_level(tk);
        ret->r_node = 0;
        return ret;
    } else {
        return cexpr_parce_0_level(tk);
    }
}

// binary * | / 
static void* cexpr_parce_2_level(CExprTokenizer* tk) {
    void* cur_node = cexpr_parce_1_level(tk);
    while (tk->cur_token_type == TK_CHAR && (tk->char_token_value == '*' || tk->char_token_value == '/')) {
        int op_type = tk->char_token_value == '*' ? CEXPR_OPERATOR_MUL : CEXPR_OPERATOR_DIV;
        cexpr_parse_next_token(tk);
        void* r_node = cexpr_parce_1_level(tk);
        CExprOperatorNode* new_node = malloc(sizeof(CExprOperatorNode));
        new_node->common.node_type = CEXPR_NODE_OPERATOR;
        new_node->operator_type = op_type;
        new_node->l_node = cur_node;
        new_node->r_node = r_node;
        cur_node = new_node;
    }
    return cur_node;
}

// binary + | - 
static void* cexpr_parce_3_level(CExprTokenizer* tk) {
    void* cur_node = cexpr_parce_2_level(tk);
    while (tk->cur_token_type == TK_CHAR && (tk->char_token_value == '+' || tk->char_token_value == '-')) {
        int op_type = tk->char_token_value == '+' ? CEXPR_OPERATOR_ADD : CEXPR_OPERATOR_SUB;
        cexpr_parse_next_token(tk);
        void* r_node = cexpr_parce_2_level(tk);
        CExprOperatorNode* new_node = malloc(sizeof(CExprOperatorNode));
        new_node->common.node_type = CEXPR_NODE_OPERATOR;
        new_node->operator_type = op_type;
        new_node->l_node = cur_node;
        new_node->r_node = r_node;
        cur_node = new_node;
    }
    return cur_node;
}

static void* cexpr_parce_impl(const char* str, size_t len) {
    if (!str || !len) return 0; 
    CExprTokenizer tk = {0};
    tk.str_beg = tk.str_cur = str;
    tk.str_end = str + len;
    tk.is_valid = true;
    cexpr_parse_next_token(&tk);
    void* ret = cexpr_parce_3_level(&tk);

    if (!tk.is_valid) {
        cexpr_free_expression_impl(ret);
        return 0;
    }

    return ret;
}
