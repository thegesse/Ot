use logos::Logos;
use chumsky::Parser;
use inkwell::context::Context;
use std::env;

mod lexer;
mod ast;
mod parser;
mod compiler;

use lexer::Token;

fn main() {
    let args: Vec<String> = env::args().collect();
    let filename = args.get(1).map(|s| s.as_str()).unwrap_or("ritual.ot");
    let source = std::fs::read_to_string(filename)
        .unwrap_or_else(|_| panic!("Could not find the file: {}", filename));

    let tokens: Vec<Token> = Token::lexer(&source).map(|t| t.unwrap()).collect();
    let tokens: &'static [Token] = Vec::leak(tokens);

    let result = parser::parser().parse(tokens);

    // FIX: Split the result into output and errors without moving 'result' twice
    let (ast, errors) = result.into_output_errors();

    if let Some(ast) = ast {
        let context = Context::create();
        let module = context.create_module("occams_tongue");
        let builder = context.create_builder();
        
        let compiler = compiler::Compiler {
            context: &context,
            module,
            builder,
            variables: std::cell::RefCell::new(std::collections::HashMap::new()),
        };

        compiler.compile(ast);
        
        compiler.module.print_to_file("output.ll").unwrap();
        
        // Add your Clang Command call here!
        println!("The ritual has been transcribed to LLVM IR.");
    } else {
        println!("The ritual contains errors. The universe remains silent:");
        for error in errors {
            // This will show you exactly which token caused the panic
            let span = error.span();
            println!("Error: {:?} at span {:?}", error.reason(), span);
        }
    }
}