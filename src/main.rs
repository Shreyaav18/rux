use std::env;
use std::fs;
use rust_compiler::*;

fn main() {
    let args: Vec<String> = env::args().collect();
    
    if args.len() < 2 {
        eprintln!("Usage: {} <source_file>", args[0]);
        std::process::exit(1);
    }
    
    let source = fs::read_to_string(&args[1])
        .expect("Failed to read source file");
    
    match compile_and_run(&source) {
        Ok(_) => println!("\nProgram executed successfully"),
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    }
}

fn compile_and_run(source: &str) -> Result<(), String> {
    println!("=== Lexical Analysis ===");
    let mut lexer = lexer::Lexer::new(source.to_string());
    let tokens = lexer.tokenize()?;
    println!("Tokens: {} generated", tokens.len());

    println!("\n=== Parsing ===");
    let mut parser = parser::Parser::new(tokens);
    let ast = parser.parse()?;
    println!("AST: {} functions parsed", ast.functions.len());

    println!("\n=== Semantic Analysis ===");
    let mut semantic = semantic::SemanticAnalyzer::new();
    semantic.analyze(&ast)?;
    println!("Type checking passed");

    println!("\n=== IR Generation ===");
    let mut ir_gen = ir::IRGenerator::new();
    let ir_program = ir_gen.generate(&ast)?;
    println!("IR: {} functions generated", ir_program.functions.len());

    println!("\n=== Bytecode Generation ===");
    let bytecode = bytecode::BytecodeGenerator::generate(&ir_program)?;
    println!("Bytecode: {} functions compiled", bytecode.functions.len());

    println!("\n=== Execution ===");
    let mut vm = vm::VM::new(bytecode);
    vm.run()?;

    Ok(())
}