# Rust Compiler

A complete compiler implementation in Rust that compiles a custom C-like language to bytecode and executes it on a stack-based virtual machine.

## Architecture
```
Source Code → Lexer → Parser → Semantic Analyzer → IR Generator → Bytecode Generator → VM
```

### Pipeline Stages

1. **Lexer** - Converts source text into tokens
2. **Parser** - Builds Abstract Syntax Tree (AST) from tokens
3. **Semantic Analyzer** - Performs type checking and validates scopes
4. **IR Generator** - Transforms AST into intermediate representation
5. **Bytecode Generator** - Compiles IR to compact bytecode with jump patching
6. **Virtual Machine** - Executes bytecode using stack-based architecture

## Language Features

**Types:** `int`, `bool`, `string`, `void`

**Control Flow:** `if/else`, `while`, `return`

**Operators:** 
- Arithmetic: `+`, `-`, `*`, `/`, `%`
- Comparison: `==`, `!=`, `<`, `<=`, `>`, `>=`
- Logical: `&&`, `||`, `!`

**Functions:** Parameters, return values, recursion support

**I/O:** `print()` statements

## Example Program
```
fn fibonacci(n: int) -> int {
    if (n <= 1) {
        return n;
    } else {
        return fibonacci(n - 1) + fibonacci(n - 2);
    }
}

fn main() -> int {
    print(fibonacci(10));
    return 0;
}
```

## Usage
```bash
cargo build --release
cargo run --release examples/program.lang
```

## Project Structure
```
src/
├── main.rs          # Entry point and compilation pipeline
├── token.rs         # Token types and definitions
├── lexer.rs         # Lexical analysis
├── ast.rs           # Abstract syntax tree structures
├── parser.rs        # Recursive descent parser
├── semantic.rs      # Type checking and validation
├── ir.rs            # Intermediate representation
├── bytecode.rs      # Bytecode generation
└── vm.rs            # Virtual machine execution
```

## Implementation Details

- **No external compiler libraries** - Built from scratch
- **Stack-based VM** - Simple and efficient execution model
- **No LLVM dependency** - Pure Rust implementation
- **Bytecode format** - Compact with jump offset patching
- **Call stack** - Full function call support with local variables

## Requirements

- Rust 2021 edition or later
- No external dependencies

## Tested Examples

- Factorial (recursion)
- Fibonacci sequence
- Loop summation
- Arithmetic expressions
- Conditional logic