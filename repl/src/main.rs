use compiler::compiler::Compiler;
use lexer::lexer::Lexer;
use lexer::token::TokenType;
use parser::parser::Parser;
use qbe::Module;
use std::{
    io::{Read, Write, stdin},
    process::{Command, Stdio},
};

fn main() {
    println!("Press ctrl-d to send eof");
    let mut buf = Vec::new();
    loop {
        println!(">>> ");
        buf.clear();
        stdin().lock().read_to_end(&mut buf).unwrap();
        println!();
        let string = String::from_utf8_lossy(&buf);
        let mut lexer = Lexer::from(&*string);
        let mut parser = Parser::from(&*string);
        println!("Tokens (Lexer)");
        loop {
            match lexer.bump() {
                Ok(token) => {
                    if token.token_type == TokenType::Eof {
                        print!("{:?}", token.token_type);
                        break;
                    }
                    print!("{:?}, ", token.token_type);
                }
                Err(err) => {
                    println!("\x1b[91mError: {:?}\x1b[0m", err);
                }
            }
        }
        println!("\n");
        //println!("\nAST (Parser)");
        match parser.parse() {
            Ok(ast) => {
                println!("Pretty Printed (Parser)");
                let mut buf = String::new();
                ast.write_preety_string(&mut buf).unwrap();
                println!("{}", buf);
                println!("LMC (compiler)");
                let mut result = Compiler::compile(ast);
                result.optimise();
                println!("{}", result.get_program())
            }
            Err(err) => {
                println!("\x1b[91mError: {:?}\x1b[0m", err);
            }
        }
    }
}
