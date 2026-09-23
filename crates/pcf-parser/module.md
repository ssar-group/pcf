# pcf-parser

Parser for transforming a PCF token stream into a structured Abstract Syntax Tree (AST).

The parser consumes tokens produced by `pcf-lexer`, validates their syntax, and builds the corresponding AST representation used by the rest of the PCF toolchain.
