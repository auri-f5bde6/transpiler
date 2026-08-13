use compiler::visitor::Compiler;
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
        println!("Tokens (Lexer phase)");
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
        println!("\n\nAST (Parser)");
        match parser.parse() {
            Ok(ast) => {
                //println!("{:#?}", ast);
                let mut buf = String::new();
                //println!("\nPreety Print");
                //ast.write_preety_string(&mut buf).unwrap();
                println!("{}", buf);

                let mut module = Compiler::compile(ast);
                println!("\nGenerated QBE IR");
                println!("{}", module);

                let mut qbe_child = Command::new("qbe")
                    .stdin(Stdio::piped())
                    .stdout(Stdio::piped())
                    .spawn()
                    .expect("Failed to run qbe");
                qbe_child
                    .stdin
                    .take()
                    .unwrap()
                    .write_all(format!("{}", module).as_bytes())
                    .unwrap();
                qbe_child.wait().unwrap();
                let cc_child = Command::new("gcc")
                    .arg("-o")
                    .arg("program")
                    .arg("-x")
                    .arg("assembler")
                    .arg("-")
                    .stdin(Stdio::from(qbe_child.stdout.unwrap()))
                    .spawn()
                    .unwrap()
                    .wait();
                println!("Executing compiled program");
                let program = Command::new("./program").spawn().unwrap().wait();
            }
            Err(err) => {
                println!("\x1b[91mError: {:?}\x1b[0m", err);
            }
        }
    }
}
