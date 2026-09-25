use std::fmt::format;
use wasm_bindgen::prelude::wasm_bindgen;
use compiler::compiler::Compiler;
use compiler::optimiser::Optimiser;
use parser::parser::Parser;
use parser::PrettyPrint;
use type_checker::TypeChecker;

#[wasm_bindgen]
pub fn transpile(name: &str) -> String {
    let mut parser = Parser::from(&*name);
    match parser.parse() {
        Ok(ast) => {
            let errors = TypeChecker::new().check(&ast);
            if let Some(errors) = errors {
                format!("{:?}", errors)
            } else {
                let mut result = Compiler::compile(ast);
                result = Optimiser::new(result).optimise();
                format!("{}", result.get_program())
            }
        }
        Err(err) => {
            format!("Error: {:?}", err)
        }
    }
}