use crate::lexer::{Lexer, LexerError};
use crate::token::{LispVal, Token, TokenType};

pub type Program = Vec<LispVal>;

#[derive(Debug)]
pub enum ParseError {
    Lexer(LexerError),
    PError(String),
}
pub struct Parser<'a> {
    lex: &'a mut Lexer,
    cur_token: Token,
    peek_token: Token,
    //errors: Vec<ParseError>,
}

impl<'a> Parser<'a> {
    pub fn init_parser(l: &'a mut Lexer) -> Result<Parser<'a>, ParseError> {
        let mut p = Parser {
            lex: l,
            cur_token: Token {
                literal: "".to_owned(),
                kind: TokenType::ILLEGAL,
            },
            peek_token: Token {
                literal: "".to_owned(),
                kind: TokenType::ILLEGAL,
            },
            //errors: Vec::<ParseError>::new(),
        };
        match p.next_token() {
            Ok(()) => (),
            Err(err) => return Err(err),
        }
        match p.next_token() {
            Ok(()) => (),
            Err(err) => return Err(err),
        }
        Ok(p)
    }

    pub fn next_token(&mut self) -> Result<(), ParseError> {
        self.cur_token = self.peek_token.to_owned();
        let tok = self.lex.next_token();
        match tok {
            Ok(t) => {
                self.peek_token = t;
                Ok(())
            }
            Err(err) => Err(ParseError::Lexer(err)),
        }
    }

    // fn cur_token_is(&self, t: TokenType) -> bool {
    //     self.cur_token.kind == t
    // }

    // fn peek_token_is(&self, t: TokenType) -> bool {
    //     self.peek_token.kind == t
    // }

    pub fn parse_program(&mut self) -> Result<Program, ParseError> {
        let mut program: Program = Program::new();
        let mut cur_t = self.cur_token.kind;
        while cur_t != TokenType::EOF && self.lex.position < self.lex.input.len() {
            let val = self.parse_lisp_val();
            match val {
                Ok(x) => program.push(x),
                Err(err) => return Err(err),
            }
            

            cur_t = self.cur_token.kind;
        }
        Ok(program)
    }

    fn parse_lisp_val(&mut self) -> Result<LispVal, ParseError> {
        match self.cur_token.kind {
            TokenType::LPAREN => {
                match self.next_token(){
                    Ok(()) => (),
                    Err(err) => return Err(err)
                }
                match self.parse_list(&mut Vec::<LispVal>::new()) {
                Ok(x) => {
                    match self.next_token(){
                        Ok(()) => (),
                        Err(err) => return Err(err)
                    }
                    Ok(x)
                }
                Err(x) => Err(x),
            }},
            TokenType::QUOTE => {
                let mut v = Vec::<LispVal>::new();
                v.push(LispVal::Atom(self.cur_token.clone()));
                match self.next_token() {
                    Ok(()) => (),
                    Err(err) => return Err(err),
                }
                match self.parse_list(&mut v) {
                    Ok(x) => {
                        match self.next_token() {
                            Ok(()) => (),
                            Err(err) => return Err(err),
                        }
                        Ok(x)
                    }
                    Err(x) => Err(x),
                }
            }
            // Should handle define, let, quote, quasiquote, if, cond... etc
            TokenType::IDENT
            | TokenType::PLUS
            | TokenType::MINUS
            | TokenType::ASTERICK
            | TokenType::SLASH
            | TokenType::AND
            | TokenType::BEGIN
            //| TokenType::DEFINE
            | TokenType::ELSE
            | TokenType::IF
            | TokenType::NOT
            | TokenType::OR
            | TokenType::LAMBDA
            //| TokenType::LET
            | TokenType::FALSE
            | TokenType::TRUE
            | TokenType::LT
            | TokenType::EQ
            | TokenType::LTEQ
            | TokenType::GT
            | TokenType::GTEQ => {
                let x = self.parse_atom();
                match self.next_token(){
                    Ok(()) => (),
                    Err(err) => return Err(err)
                }
                Ok(x)
            }
            TokenType::LET =>  match self.parse_let() {
                Ok(x) => {
                    match self.next_token(){
                        Ok(()) => (),
                        Err(err) => return Err(err)
                    }
                    Ok(x)
                }
                Err(err) => return Err(err),
            },
            //TokenType::DEFINE => {vec.push(self.parse_define().unwrap()),
            TokenType::INT => match self.parse_number() {
                Ok(x) => {
                    match self.next_token(){
                        Ok(()) => (),
                        Err(err) => return Err(err)
                    }
                     Ok(x)
                }
                Err(err) => return Err(err),
            },
            TokenType::STRING => {
                let x = self.parse_string();
                match self.next_token(){
                    Ok(()) => (),
                    Err(err) => return Err(err)
                }
                Ok(x)
            }
            _ => Err(ParseError::PError(format!(
                "Unknown Token in parse_lisp_val: Token literal: {} Token kind: {:?}",
                self.cur_token.literal, self.cur_token.kind
            ))),
        }
    }

    fn parse_list(&mut self, vec: &mut Vec<LispVal>) -> Result<LispVal, ParseError> {
        while self.cur_token.kind != TokenType::RPAREN && self.cur_token.kind != TokenType::EOF {
            match self.cur_token.kind {
                TokenType::LPAREN => {
                    let mut temp_vec = Vec::<LispVal>::new();
                    match self.next_token() {
                             Ok(()) => (),
                             Err(err) => return Err(err),
                    } 
                    match self.parse_list(&mut temp_vec) {
                        Ok(x) => vec.push(x),
                        Err(err) => return Err(err), //self.errors.push(err)
                    }
                }
                TokenType::QUOTE => {
                    let mut v = Vec::<LispVal>::new();
                    v.push(LispVal::Atom(self.cur_token.clone()));
                    match self.next_token() {
                        Ok(()) => (),
                        Err(err) => return Err(err),
                    }
                    match self.parse_list(&mut v) {
                        Ok(x) => vec.push(x),
                        Err(x) => return Err(x),
                    };
                }
                // Should handle define, let, if, cond... etc
                TokenType::IDENT
                | TokenType::PLUS
                | TokenType::MINUS
                | TokenType::ASTERICK
                | TokenType::SLASH
                | TokenType::AND
                | TokenType::BEGIN
                //| TokenType::DEFINE
                | TokenType::ELSE
                | TokenType::IF
                | TokenType::NOT
                | TokenType::OR
                //| TokenType::LET
                | TokenType::LT
                | TokenType::EQ
                | TokenType::LTEQ
                | TokenType::GT
                | TokenType::GTEQ => {
                    vec.push(self.parse_atom());
                }
                TokenType::LET => vec.push(self.parse_let().unwrap()),
                TokenType::LAMBDA => vec.push(self.parse_lambda().unwrap()),
                TokenType::DEFINE => vec.push(self.parse_define().unwrap()),
                TokenType::FALSE => vec.push(LispVal::Bool(false)),
                TokenType::TRUE => vec.push(LispVal::Bool(true)),

                TokenType::INT => match self.parse_number() {
                    Ok(x) => {
                        vec.push(x);
                    }
                    Err(err) => return Err(err),
                },
                TokenType::STRING => {
                    vec.push(self.parse_string());
                }
                //Token
                _ => {
                    return Err(ParseError::PError(format!(
                        "Unknown Token in parse_lisp_val: Token literal: {} Token kind: {:?}",
                        self.cur_token.literal, self.cur_token.kind
                    )))
                }
            }
            match self.next_token() {
                Ok(()) => (),
                Err(err) => return Err(err),
            }
        }
        Ok(LispVal::List(vec.to_vec()))
    }

    #[inline(always)]
    fn parse_atom(&self) -> LispVal {
        LispVal::Atom(self.cur_token.to_owned())
    }

    #[inline(always)]
    fn parse_number(&self) -> Result<LispVal, ParseError> {
        let int = self.cur_token.literal.parse::<i64>();
        match int {
            Ok(x) => Ok(LispVal::Number(x)),
            Err(_) => Err(ParseError::PError(format!(
                "Could not parse {} as a number",
                self.cur_token.literal
            ))),
        }
    }

    #[inline(always)]
    fn parse_string(&self) -> LispVal {
        LispVal::String(self.cur_token.literal.to_owned())
    }

    fn parse_lambda(&mut self) -> Result<LispVal, ParseError> {
        match self.next_token() {
            Ok(()) => (),
            Err(err) => return Err(err),
        }
        let x = match self.parse_lisp_val() {
            Ok(l) => l,
            Err(err) => return Err(err),
        };
        let y = match self.parse_lisp_val() {
            Ok(l) => l,
            Err(err) => return Err(err),
        };
        match self.next_token() {
            Ok(()) => (),
            Err(err) => return Err(err),
        }
        Ok(LispVal::Lamda(Box::new(x), Box::new(y)))
    }

    fn parse_define(&mut self) -> Result<LispVal, ParseError> {
        let def = self.parse_atom();
        match self.next_token() {
            Ok(()) => (),
            Err(err) => return Err(err),
        }
        println!("{:?}", self.cur_token);
        let y = match self.parse_lisp_val() {
            Ok(l) => l,
            Err(err) => return Err(err),
        };
        println!("{:?}", y);
        let z = match self.parse_lisp_val() {
            Ok(l) => l,
            Err(err) => return Err(err),
        };
        println!("{:?}", z);
        Ok(LispVal::List(vec![def, y, z]))
    }
    fn parse_let(&mut self) -> Result<LispVal, ParseError> {
        let l = self.parse_atom();
        match self.next_token() {
            Ok(()) => (),
            Err(err) => return Err(err),
        }
        //println!("{:?}", l);
        match self.next_token() {
            Ok(()) => (),
            Err(err) => return Err(err),
        }
        let mut v = Vec::<LispVal>::new();
        let pairs = self.parse_list(&mut v).unwrap();
        println!("pairs {:?}", pairs);
        let mut v2 = Vec::<LispVal>::new();
        match self.next_token() {
            Ok(()) => (),
            Err(err) => return Err(err),
        }
        let eval = self.parse_list(&mut v2).unwrap();
        println!("eval {:?}",eval);
        Ok(LispVal::List(vec![l,pairs,eval]))

    }
}


#[cfg(test)]
mod tests {
    use crate::eval::{eval_prog, Environment};

    use super::*;
    #[test]
    fn parse_let_mult_with_neg() {
        let input = "(let ((x -2) (y 3)) (* x y))";
        let mut lex = Lexer::init_lex(input.to_owned());
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
                let x = eval_prog(x, &mut env).unwrap();
                for i in x {
                    println!("{}", i.show_val());
                }
            }
            Err(e) => {
                println!("{:?}", e);
                assert!(false);
            }
        }
    }

    #[test]
    fn parse_let_sub() {
        let input = "(let ((x 2 ) (y 3)) (- x y))";
        let mut lex = Lexer::init_lex(input.to_owned());
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
                let x = eval_prog(x, &mut env).unwrap();
                for i in x {
                    println!("{}", i.show_val());
                }
            }
            Err(e) => {
                println!("{:?}", e);
                assert!(false);
            }
        }
    }

    #[test]
    fn parse_let_add() {
        let input = "(let ((x 2 ) (y 3)) (+ x y))";
        let mut lex = Lexer::init_lex(input.to_owned());
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
                let x = eval_prog(x, &mut env).unwrap();
                for i in x {
                    println!("{}", i.show_val());
                }
            }
            Err(e) => {
                println!("{:?}", e);
                assert!(false);
            }
        }
    }

    #[test]
    fn parse_let_cond_lt() {
        let input = "(let ((x 2 ) (y 3)) (<  x y))";
        let mut lex = Lexer::init_lex(input.to_owned());
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
                let x = eval_prog(x, &mut env).unwrap();
                for i in x {
                    println!("{}", i.show_val());
                }
            }
            Err(e) => {
                println!("{:?}", e);
                assert!(false);
            }
        }
    }

    #[test]
    fn parse_let_cond_gt() {
        let input = "(let ((x 2 ) (y 3)) (>  x y))";
        let mut lex = Lexer::init_lex(input.to_owned());
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
                let x = eval_prog(x, &mut env).unwrap();
                for i in x {
                    println!("{}", i.show_val())
                }
            }
            Err(e) => {
                println!("{:?}", e);
                assert!(false);
            }
        }
    }

    #[test]
    fn parse_let_and_lambda() {
        let input ="(let ((x ((lambda (x) (+ x x)) 3))) (* x x))";
        let mut lex = Lexer::init_lex(input.to_owned());
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
                let x = eval_prog(x, &mut env).unwrap();
                for i in x {
                    println!("{}", i.show_val())
                }
            }
            Err(e) => {
                println!("{:?}", e);
                assert!(false);
            }
        }
    }

    #[test]
    fn parse_more_calc() {
        let input = "(+ (* 3 5) (+ 1 2))"; // should return 18
        let mut lex = Lexer::init_lex(input.to_owned());
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
                let x = eval_prog(x, &mut env).unwrap();
                for i in x {
                    println!("{}", i.show_val())
                }
            }
            Err(e) => {
                println!("{:?}", e);
                assert!(false);
            }
        }
    }

    #[test]
    fn parse_define() {
        let input = "(define x 3) (* x 3)";
        let mut lex = Lexer::init_lex(input.to_owned());
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
                let x = eval_prog(x, &mut env).unwrap();
                for i in x {
                    println!("{}", i.show_val());
                }
            }
            Err(e) => {
                println!("{:?}", e);
                assert!(false);
            }
        }
    }

    #[test]
    fn parse_define_proc() {
        let input = "(define (p x y) (+ x y ))";
        let mut lex = Lexer::init_lex(input.to_owned());
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
                let x = eval_prog(x, &mut env).unwrap();
                for i in x {
                    println!("{}", i.show_val());
                }
            }
            Err(e) => {
                println!("{:?}", e);
                assert!(false);
            }
        }
    }

    #[test]
    fn parse_apply_proc() {
        let input = "(define (p x y z) (+ x y z)) (p 10 2 3)";
        let mut lex = Lexer::init_lex(input.to_owned());
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
                let x = eval_prog(x, &mut env).unwrap();
                for i in x {
                    println!("{}", i.show_val());
                }
            }
            Err(e) => {
                println!("{:?}", e);
                assert!(false);
            }
        }
    }

    #[test]
    fn parse_apply_proc2() {
        let input = "(define (circumference radius) (* 3 radius radius)) (circumference 5)";
        let mut lex = Lexer::init_lex(input.to_owned());
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
                let x = eval_prog(x, &mut env).unwrap();
                for i in x {
                    println!("{}", i.show_val());
                }
            }
            Err(e) => {
                println!("{:?}", e);
                assert!(false);
            }
        }
    }

    // FIXME not working at all leads to error evaluating lambda parameters
    #[test]
    fn parse_apply_proc3() {
        let input = "(define (flip fn) (lambda (a b) (fn b a))) ((flip -) 5 8)";
        let mut lex = Lexer::init_lex(input.to_owned());
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
                let x = eval_prog(x, &mut env).unwrap();
                for i in x {
                    println!("{}", i.show_val());
                }
            }
            Err(e) => {
                println!("{:?}", e);
                assert!(false);
            }
        }
    }

    #[test]
    fn parse_lambda_eval() {
        let input = "((lambda (x) (* x x)) 3)";
        let mut lex = Lexer::init_lex(input.to_owned());
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
                let x = eval_prog(x, &mut env).unwrap();
                for i in x {
                    println!("{}", i.show_val());
                }
            }
            Err(e) => {
                println!("{:?}", e);
                assert!(false);
            }
        }
    }

    #[test]
    fn parse_lambda_eval2() {
        let input = "((lambda (x y) (* x y)) 3 4)";
        let mut lex = Lexer::init_lex(input.to_owned());
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
                let x = eval_prog(x, &mut env).unwrap();
                for i in x {
                    println!("{}", i.show_val());
                }
            }
            Err(e) => {
                println!("{:?}", e);
                assert!(false);
            }
        }
    }

    #[test]
    fn parse_quote() {
        let input = "(\'(* x x))";
        let mut lex = Lexer::init_lex(input.to_owned());
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
                let x = eval_prog(x, &mut env).unwrap();
                for i in x {
                    println!("{}", i.show_val());
                }
            }
            Err(e) => {
                println!("{:?}", e);
                assert!(false);
            }
        }
    }
}
