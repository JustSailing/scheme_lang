mod eval;
mod lexer;
mod parser;
mod token;
use std::io::{stdin, stdout, Write};

use eval::{eval_prog, Environment};
use lexer::Lexer;
use parser::Parser;
fn main() {
    println!("Scheme Interpreter");
    let _ = stdout().write(">>>".to_string().as_bytes());
    loop {
        let mut input_string = String::new();
        stdin()
            .read_line(&mut input_string)
            .ok()
            .expect("Failed to read line");
        input_string = input_string.trim().to_string();
        let _ = stdout().flush();
        let mut lex = Lexer::init_lex(input_string.to_owned());
        let par = Parser::init_parser(&mut lex);
        let mut x = match par {
            Ok(x) => x,
            Err(e) => {
                println!("{:?}", e);
                panic!("false");
            }
        };
        let prog = x.parse_program();
        match prog {
            Ok(x) => {
                let mut env = Environment::init_env();
                let y = eval_prog(x, &mut env);
                match y {
                    Ok(y) => {
                        for i in y {
                            let _ = stdout().write(format!("{}\n", i.show_val()).as_bytes());
                        }
                    }
                    Err(x) => eprint!("{}", x),
                }
            }
            Err(e) => {
                println!("{:?}", e);
                assert!(false);
            }
        }
        let _ = stdout().write(">>>".to_string().as_bytes());
    }
}
