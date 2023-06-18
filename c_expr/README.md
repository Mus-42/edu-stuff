# Simple math expression parser & evaluator

Expression consist of ``+`` ``-`` ``*`` ``/`` operators, brackets ``()``, variables, numbers and function calls.

Example of valid expression: ``mod(sqrt(4 + (5 + 7) * 2), 2) - pi / 2``

All invalid expressions return NaN during evaluation

## TODO list

* Improve usage of ``CExprExecutor`` (make separate stages & structs for binding constatnts and actual evaluating (function & variables))
* Clear way to process errors (using new ``CExprExecutor`` + error codes?) (tell position of invalid token in source string?)
* Markers for "pure" functions in ``CExprFunctionBindDesc``
* "Optimize" expression greedly calculate all possible things and replace them with literals
* Extend max function args count up to 7 (compress all fn. call ast nodes into single one)
