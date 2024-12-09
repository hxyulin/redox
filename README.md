# Redox

Redox is a systems programming language that can be compiled to native code, web assembly, or be interpreted.

# Implementation Details

### Lexer
The redox compiler will use the 'redox-lexer' crate to tokenize the source code.
This crate defines a simple token structure, and uses the 'logos' crate to define the lexer rules.
This module performs no validation of the source code, and only performs lexing.

### Parser
The redox compiler will use the 'redox-parser' crate to parse the source code.
This crate defines a parser, written in pure rust, that uses the 'redox-lexer' crate to tokenize the source code.
The language has no ambiguity, so the parser does not need to perform any lookahead.
The parser will return an AST, with only basic type information on literals, or typed variables.
The parser is a recursive descent one pass parser, and will not perform any backtracking. This ensures that the parser will always 
be able to parse the source code in linear time.

### Type Checker
The redox compiler will use the 'redox-type-checker' crate to type check the source code.
This crate will perform type inference on the AST, and check that the types are valid.
This crate also gives hints to the user on what type of value is expected for a given expression, and what checks for common typos, 
as well as missing type annotations or type casts.

### IR Compiler
The redox compiler will use the 'redox-ir-compiler' crate to compile the source code into the intermediate representation. Taken from the typed AST,
the IR compiler will generate an intermediate representation of the source code, known as 'RXIR'. This phase performs no additional checks or optimizations.
An optional '.rxir' file can be generated, which contains the intermediate representation of the source code as a readable format.

### Intermediate Representation
The redox compiler will generate an intermediate representation of the source code, known as 'RXIR'.
This intermediate representation is a simplified version of the AST, and is used to generate the final output, using the 'Codegen' modules, 
which can generate code for different backends (into native code, wasm using LLVM, or into C code using a custom backend).

### Optimization
The redox compiler will perform optimizations on the intermediate representation, known as 'RXIR'.
The optimization phase contains multiple passes, which can be toggled. 
Currently there are no optimizations implemented, but this is a planned feature.

### Minimization
Minimization is an optional feature that can be enabled in the compiler configuration, which is disabled by default.
This can be used to reduce the size of the compiled program, by removing unused code and data, and inlining functions, as well as renaming variables that are not explicitly declared as externally linkable. This is typically used for generating WASM output.

### Codegen
The redox compiler will use the 'redox-codegen' crate to generate code from the intermediate representation.
This crate will generate code for different backends, currently supported:
- LLVM IR (for native code on x86, x64, aarch64, and arm)
Other backends are not yet implemented.

# Roadmap

# Building

## Prerequisites
- Rust
- Cargo
These can be easily installed using the [rustup](https://rustup.rs/) tool.

## Building
To build the compiler, run the following command:
```bash
cargo build
```

To run the tests, run the following command:
```bash
cargo test
```

To run the CLI, run the following command:
```bash
cargo run -- <source file>
```

# Documentation
Currently there is no documentation for the redox compiler, but the source code is well documented.

# Roadmap

# License
Redox is licensed under the Apache License, Version 2.0. See [LICENSE](LICENSE) for more information.
