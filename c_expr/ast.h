#pragma once 

#ifndef _C_EXPR_AST_H_INCLUDE_
#define _C_EXPR_AST_H_INCLUDE_


typedef struct CExprNodeCommon {
    int node_type;
} CExprNodeCommon;


typedef struct CExprOperatorNode {
    CExprNodeCommon common;
    int operator_type;
    void* l_node;
    void* r_node;
} CExprOperatorNode;


typedef struct CExprNumericLiteralNode {
    CExprNodeCommon common;
    double value;
} CExprNumericLiteralNode;

typedef struct CExprVariableOrConstantNode {
    CExprNodeCommon common;
    const char* variable_name;
    size_t name_len;
} CExprVariableOrConstantNode;


typedef struct CExprFunctionCall0Node {
    CExprNodeCommon common;
    const char* function_name;
    size_t name_len;
} CExprFunctionCall0Node;

typedef struct CExprFunctionCall1Node {
    CExprNodeCommon common;
    const char* function_name;
    size_t name_len;
    void *arg0;
} CExprFunctionCall1Node;

typedef struct CExprFunctionCall2Node {
    CExprNodeCommon common;
    const char* function_name;
    size_t name_len;
    void *arg0, *arg1;
} CExprFunctionCall2Node;

typedef struct CExprFunctionCall3Node {
    CExprNodeCommon common;
    const char* function_name;
    size_t name_len;
    void *arg0, *arg1, *arg2;
} CExprFunctionCall3Node;


enum CExprNodeType {
    CEXPR_NODE_NONE      = 0,
    CEXPR_NODE_OPERATOR  = 1,
    CEXPR_NODE_LITERAL   = 2,
    CEXPR_NODE_VARIABLE  = 3,
    
    CEXPR_NODE_FUNCTION_CALL0 = 4 | 0,
    CEXPR_NODE_FUNCTION_CALL1 = 4 | 1,
    CEXPR_NODE_FUNCTION_CALL2 = 4 | 2,
    CEXPR_NODE_FUNCTION_CALL3 = 4 | 3,

    // Helpers
    CEXPR_TOTAL_NODE_COUNT = 8
};

enum CExprOperatorType {
    // Binary
    CEXPR_OPERATOR_ADD,
    CEXPR_OPERATOR_SUB,
    CEXPR_OPERATOR_MUL,
    CEXPR_OPERATOR_DIV,
    
    // Unary
    CEXPR_OPERATOR_MINUS,
    CEXPR_OPERATOR_PLUS,
    
    // Helpers
    CEXPR_TOTAL_OPERATOR_COUNT 
};

#endif//_C_EXPR_AST_H_INCLUDE_
