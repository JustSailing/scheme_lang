use std::collections::HashMap;

use crate::{parser::Program, token::*};

#[derive(PartialEq, Clone)]
pub struct Environment {
    store: HashMap<String, LispVal>,
    outer: Option<Box<Environment>>,
}

impl Environment {
    pub fn init_env() -> Environment {
        Environment {
            store: HashMap::<String, LispVal>::new(),
            outer: None,
        }
    }

    pub fn new_enclosed(&mut self, envir: Environment) -> Environment {
        let mut env = Environment::init_env();
        env.outer = Some(Box::new(envir));
        env
    }

    pub fn get(&self, name: String) -> Option<LispVal> {
        let p = self.store.get(&name);
        match p {
            Some(c) => return Some(c.clone()),
            None => {
                if self.outer != None {
                    let p = self.outer.as_ref()?;

                    let pp = p.get(name.clone());
                    match pp {
                        Some(c) => return Some(c),
                        None => return None,
                    }
                } else {
                    None
                }
            }
        }
    }

    pub fn set(&mut self, name: String, obj: LispVal) -> LispVal {
        self.store.insert(name, obj.clone());
        return obj;
    }
}

pub fn eval_prog(prog: Program, env: &mut Environment) -> Result<Vec<LispVal>, String> {
    let mut v = Vec::<LispVal>::new();
    // let x = prog.len();
    // println!("{}", x);
    for list in prog {
        let result = eval(list, env);
        match result {
            Ok(x) => {
                v.push(x);
            }
            Err(x) => return Err(x),
        }
    }
    Ok(v)
}

fn eval(val: LispVal, env: &mut Environment) -> Result<LispVal, String> {
    let evl = match val {
        LispVal::List(_) => eval_list(&val.clone(), env),
        LispVal::Atom(x) => eval_atom(x.literal, env),
        //LispVal::DottedList(_, _) => eval_dotted_list(&val),
        LispVal::Float(_) => Ok(val),
        LispVal::Number(_) => Ok(val),
        LispVal::String(_) => Ok(val.clone()),
        //LispVal::Fun(x) => eval_fun(&val),
        //LispVal::Lamda(_, _) => eval_lambda(&val, env),
        LispVal::Nil => Ok(val),
        LispVal::Bool(_) => Ok(val),
        _ => return Err("problem with eval".to_owned()),
    };
    evl
}

fn eval_atom(val: String, env: &mut Environment) -> Result<LispVal, String> {
    let check = env.get(val.clone());
    match check {
        Some(x) => Ok(x),
        None => Err(format!("identifier not in environment: {}", val.clone())),
    }
}
fn eval_list(val: &LispVal, env: &mut Environment) -> Result<LispVal, String> {
    let list = match val.clone() {
        LispVal::List(x) => x,
        _ => return Err(format!("not a list. LispVal: {:?}", *val)),
    };

    match &list[0] {
        LispVal::Atom(x) => match x.kind {
            TokenType::MINUS | TokenType::PLUS | TokenType::ASTERICK | TokenType::SLASH => {
                return eval_bin(&list, env)
            }
            TokenType::LET => return eval_let(&list, env),
            TokenType::EQ | TokenType::GT | TokenType::GTEQ | TokenType::LT | TokenType::LTEQ => {
                return eval_cond(&list, env)
            }
            TokenType::QUOTE => return Ok(LispVal::List(list[1..].to_vec())),
            // TokenType::IF => return eval_if(&list),
            TokenType::DEFINE => return eval_define(&list, env),
            TokenType::LAMBDA => return eval_lambda(&list, env),
            TokenType::IDENT => {
                let x = eval_atom(x.literal.clone(), env);
                let val = match x {
                    Ok(v) => v,
                    Err(err) => return Err(err),
                };
                match val {
                    LispVal::List(x) => {
                        return eval_proc(&x, &list[1..].to_vec(), env);
                    }
                    _ => return Err(format!("should of been a list for procedure")),
                }
            }
            // TokenType::LAMBDA => return eval_lam(&list),
            _ => todo!("Not fully finished"),
        },

        LispVal::List(x) => match &x[0] {
            LispVal::Atom(z) => {
                if z.kind == TokenType::LAMBDA {
                    return eval_lambda(&list.clone(), env);
                }else if z.kind == TokenType::DEFINE {
                    let y =  eval_define(&x.clone(), env);
                    if list.len() == 1 {
                        return y;
                    }
                    return eval_list(&LispVal::List(list[1..].to_vec()), env);
                }else if z.kind == TokenType::LET {
                    eval_let(&x.clone(), env)
                }
                 else {
                    return eval_list(&list[0], env);
                }
            }

            _ => return eval_list(&list[0], env),
        },
        LispVal::Float(_) => todo!(),
        LispVal::DottedList(_, _) => todo!(),
        LispVal::Number(_) => todo!(),
        LispVal::String(_) => todo!(),
        LispVal::Fun(_, _) => todo!(),
        LispVal::Lamda(_, _) => return eval_lambda(&list, env),
        LispVal::Nil => todo!(),
        LispVal::Bool(_) => todo!(),
    }
}

fn eval_let(list: &Vec<LispVal>, env: &mut Environment) -> Result<LispVal, String> {
    let vec = &list[1];
    // println!("{}", list.len());
    // println!("{:?}", vec);
    let mut envr = env.new_enclosed(env.clone());
    let _ = match vec {
        LispVal::List(x) =>{eval_let_pair(&x, &mut envr)},
        _ => return Err(format!("let eval went wrong")),
    };
    let res = eval(list[2].clone(), &mut envr);
    match res {
        Ok(re) => return Ok(re),
        Err(x) => return Err(x),
    }
}

fn eval_let_pair(list: &Vec<LispVal>, env: &mut Environment) -> Result<(), String> {
    let mut ident: String;
    for pair in list {
        match pair {
            LispVal::List(x) => {
                match &x[0] {
                    LispVal::Atom(tok) => ident = tok.literal.clone(),
                    _ => return Err(format!("first of let pair of variable binding should be an identifier. Error in: {:?}", x[0]))
                }
                let check = eval(x[1].clone(), env);
                match check {
                Ok(x) =>  {
                    _ = env.set(ident, x);
                },
                Err(x) => return Err(x)
                }
            },
            _ => return Err(format!("variable bindings in let statement should be in pairs. (let ((x 5)(y 6))...error in {}", pair.show_val()))
        }
    }
    Ok(())
}

fn eval_cond(list: &Vec<LispVal>, env: &mut Environment) -> Result<LispVal, String> {
    if list.len() != 3 {
        return Err(
            "invalid conditional operation. should have only 2 arguments for operation ex (< 2 3)"
                .to_owned(),
        );
    }
    let var: LispVal;
    match &list[0] {
        LispVal::Atom(x) => match x.kind {
            TokenType::LT => var = eval_cond_lt(list, env).unwrap(),
            TokenType::LTEQ => var = eval_cond_lteq(list, env).unwrap(),
            TokenType::EQ => var = eval_cond_eq(list, env).unwrap(),
            TokenType::GT => var = eval_cond_gt(list, env).unwrap(),
            TokenType::GTEQ => var = eval_cond_gteq(list, env).unwrap(),
            _ => {
                return Err(format!(
                    "invalid cond operation. error: {}",
                    list[0].show_val()
                ))
            }
        },
        _ => {
            return Err(format!(
                "invalid cond operation. error: {}",
                list[0].show_val()
            ))
        }
    }
    Ok(var)
}

fn eval_cond_lt(list: &Vec<LispVal>, env: &mut Environment) -> Result<LispVal, String> {
    if list.len() != 3 {
        return Err(
            "invalid conditional operation. should have 2 arguments for operation ex (< 2 3)"
                .to_owned(),
        );
    }
    let first = list[1].clone();
    let second = list[2].clone();
    let res_first = eval(first, env).unwrap();
    let res_second = eval(second, env).unwrap();
    match res_first {
        LispVal::Number(x) => match res_second {
            LispVal::Number(y) => {
                return Ok(LispVal::Bool(x < y));
            }
            _ => {
                return Err(format!(
                    "conditonal not fully implemented first: {} second: {}",
                    res_first.show_val(),
                    res_second.show_val()
                ))
            }
        },
        _ => {
            return Err(format!(
                "conditonal not fully implemented first: {} second: {}",
                res_first.show_val(),
                res_second.show_val()
            ))
        }
    }
}

fn eval_cond_lteq(list: &Vec<LispVal>, env: &mut Environment) -> Result<LispVal, String> {
    if list.len() != 3 {
        return Err(
            "invalid conditional operation. should have 2 arguments for operation ex (< 2 3)"
                .to_owned(),
        );
    }
    let first = list[1].clone();
    let second = list[2].clone();
    let res_first = eval(first, env).unwrap();
    let res_second = eval(second, env).unwrap();
    match res_first {
        LispVal::Number(x) => match res_second {
            LispVal::Number(y) => {
                return Ok(LispVal::Bool(x <= y));
            }
            _ => {
                return Err(format!(
                    "conditonal not fully implemented first: {} second: {}",
                    res_first.show_val(),
                    res_second.show_val()
                ))
            }
        },
        _ => {
            return Err(format!(
                "conditonal not fully implemented first: {} second: {}",
                res_first.show_val(),
                res_second.show_val()
            ))
        }
    }
}

fn eval_cond_eq(list: &Vec<LispVal>, env: &mut Environment) -> Result<LispVal, String> {
    if list.len() != 3 {
        return Err(
            "invalid conditional operation. should have 2 arguments for operation ex (< 2 3)"
                .to_owned(),
        );
    }
    let first = list[1].clone();
    let second = list[2].clone();
    let res_first = eval(first, env).unwrap();
    let res_second = eval(second, env).unwrap();
    match res_first {
        LispVal::Number(x) => match res_second {
            LispVal::Number(y) => {
                return Ok(LispVal::Bool(x == y));
            }
            _ => {
                return Err(format!(
                    "conditonal not fully implemented first: {} second: {}",
                    res_first.show_val(),
                    res_second.show_val()
                ))
            }
        },
        _ => {
            return Err(format!(
                "conditonal not fully implemented first: {} second: {}",
                res_first.show_val(),
                res_second.show_val()
            ))
        }
    }
}

fn eval_cond_gt(list: &Vec<LispVal>, env: &mut Environment) -> Result<LispVal, String> {
    if list.len() != 3 {
        return Err(
            "invalid conditional operation. should have 2 arguments for operation ex (< 2 3)"
                .to_owned(),
        );
    }
    let first = list[1].clone();
    let second = list[2].clone();
    let res_first = eval(first, env).unwrap();
    let res_second = eval(second, env).unwrap();
    match res_first {
        LispVal::Number(x) => match res_second {
            LispVal::Number(y) => {
                return Ok(LispVal::Bool(x > y));
            }
            _ => {
                return Err(format!(
                    "conditonal not fully implemented first: {} second: {}",
                    res_first.show_val(),
                    res_second.show_val()
                ))
            }
        },
        _ => {
            return Err(format!(
                "conditonal not fully implemented first: {} second: {}",
                res_first.show_val(),
                res_second.show_val()
            ))
        }
    }
}

fn eval_cond_gteq(list: &Vec<LispVal>, env: &mut Environment) -> Result<LispVal, String> {
    if list.len() != 3 {
        return Err(
            "invalid conditional operation. should have 2 arguments for operation ex (< 2 3)"
                .to_owned(),
        );
    }
    let first = list[1].clone();
    let second = list[2].clone();
    let res_first = eval(first, env).unwrap();
    let res_second = eval(second, env).unwrap();
    match res_first {
        LispVal::Number(x) => match res_second {
            LispVal::Number(y) => {
                return Ok(LispVal::Bool(x >= y));
            }
            _ => {
                return Err(format!(
                    "conditonal not fully implemented first: {} second: {}",
                    res_first.show_val(),
                    res_second.show_val()
                ))
            }
        },
        _ => {
            return Err(format!(
                "conditonal not fully implemented first: {} second: {}",
                res_first.show_val(),
                res_second.show_val()
            ))
        }
    }
}

fn eval_bin(list: &Vec<LispVal>, env: &mut Environment) -> Result<LispVal, String> {
    if list.len() < 3 {
        return Err(
            "invalid binary operation. should have at least 2 arguments for operation ex (+ 2 3)"
                .to_owned(),
        );
    }
    let var: LispVal;
    match &list[0] {
        LispVal::Atom(x) => match x.kind {
            TokenType::ASTERICK => var = eval_bin_mult(list, env).unwrap(),
            TokenType::MINUS => var = eval_bin_sub(list, env).unwrap(),
            TokenType::SLASH => var = eval_bin_div(list, env).unwrap(),
            TokenType::PLUS => var = eval_bin_add(list, env).unwrap(),
            _ => {
                return Err(format!(
                    "invalid binary operation. error: {}",
                    list[0].show_val()
                ))
            }
        },
        _ => {
            return Err(format!(
                "invalid binary operation. error: {}",
                list[0].show_val()
            ))
        }
    }
    Ok(var)
}

fn eval_bin_mult(list: &Vec<LispVal>, env: &mut Environment) -> Result<LispVal, String> {
    let mut result = LispVal::Number(1);
    for i in 1..list.len() {
        let res = eval(list[i].clone(), env);
        match res {
            Ok(x) => match x {
                LispVal::Number(x) => match result {
                    LispVal::Number(y) => result = LispVal::Number(x * y),
                    _ => return Err("could not multiply lispval".to_owned()),
                },
                _ => return Err("could not multipy lispval. not a number".to_owned()),
            },
            Err(err) => return Err(err),
        }
    }
    Ok(result)
}

fn eval_bin_sub(list: &Vec<LispVal>, env: &mut Environment) -> Result<LispVal, String> {
    let mut result = LispVal::Number(0);
    // if the first number in the sub (- x y). Add x to result or the answer will be -x - y instead
    // of x - y.
    for i in 1..list.len() {
        let res = eval(list[i].clone(), env);
        match res {
            Ok(x) => {
                if i == 1 {
                    match x {
                        LispVal::Number(x) => match result {
                            LispVal::Number(y) => result = LispVal::Number(y + x),
                            _ => return Err("could not sub lispval".to_owned()),
                        },
                        _ => return Err("could not sub lispval. not a number".to_owned()),
                    }
                } else {
                    match x {
                        LispVal::Number(x) => match result {
                            LispVal::Number(y) => result = LispVal::Number(y - x),
                            _ => return Err("could not sub lispval".to_owned()),
                        },
                        _ => return Err("could not sub lispval. not a number".to_owned()),
                    }
                }
            }
            Err(err) => return Err(err),
        }
    }
    Ok(result)
}

fn eval_bin_add(list: &Vec<LispVal>, env: &mut Environment) -> Result<LispVal, String> {
    let mut result = LispVal::Number(0);
    // if the first number in the sub (- x y). Add x to result or the answer will be -x - y instead
    // of x - y.
    for i in 1..list.len() {
        let res = eval(list[i].clone(), env);
        match res {
            Ok(x) => match x {
                LispVal::Number(x) => match result {
                    LispVal::Number(y) => result = LispVal::Number(y + x),
                    _ => return Err("could not sub lispval".to_owned()),
                },
                _ => return Err("could not sub lispval. not a number".to_owned()),
            },

            Err(err) => return Err(err),
        }
    }
    Ok(result)
}

fn eval_bin_div(list: &Vec<LispVal>, env: &mut Environment) -> Result<LispVal, String> {
    let mut result = LispVal::Number(0);
    for i in 1..list.len() {
        let res = eval(list[i].clone(), env);
        match res {
            Ok(x) => {
                if i == 1 {
                    match x {
                        LispVal::Number(x) => {
                            if x == 0 {
                                return Ok(LispVal::Number(0));
                            }
                            match result {
                                LispVal::Number(y) => result = LispVal::Number(y + x),
                                _ => return Err("could not divide lispval".to_owned()),
                            }
                        }
                        _ => return Err("could not divide lispval. not a number".to_owned()),
                    }
                } else {
                    match x {
                        LispVal::Number(x) => {
                            if x == 0 {
                                return Err("cannot divide by 0".to_owned());
                            }
                            match result {
                                LispVal::Number(y) => result = LispVal::Number(y / x),
                                _ => return Err("could not sub lispval".to_owned()),
                            }
                        }
                        _ => return Err("could not sub lispval. not a number".to_owned()),
                    }
                }
            }
            Err(err) => return Err(err),
        }
    }
    Ok(result)
}

fn eval_define(list: &Vec<LispVal>, env: &mut Environment) -> Result<LispVal, String> {
    // first index should be the define atom
    // second index should be either a procedure or a variable that is later assigned a value or body of function
    // the list should be a length of 3

    if list.len() != 3 {
        return Err(format!("define should have 3 parts (define (x) (3))"));
    }
    match &list[1] {
        LispVal::Atom(x) => {
            env.set(
                x.literal.clone(),
                eval(list[2].clone(), &mut env.clone()).unwrap(),
            );
            return Ok(LispVal::List(
                [
                    LispVal::Atom(x.clone()),
                    LispVal::Atom(Token {
                        literal: "<-".to_owned(),
                        kind: TokenType::ILLEGAL,
                    }),
                    eval(list[2].clone(), env).unwrap(),
                ]
                .to_vec(),
            ));
        }
        LispVal::List(x) => {
            if x.len() < 2 {
                match &x[0] {
                    LispVal::Atom(x) => {
                        env.set(
                            x.literal.clone(),
                            eval(list[2].clone(), &mut env.clone()).unwrap(),
                        );
                        return Ok(LispVal::List(
                            [
                                LispVal::Atom(x.clone()),
                                LispVal::Atom(Token {
                                    literal: "<-".to_owned(),
                                    kind: TokenType::ILLEGAL,
                                }),
                                eval(list[2].clone(), env).unwrap(),
                            ]
                            .to_vec(),
                        ));
                    }
                    _ => {
                        return Err(format!(
                            "define should be a list of identifiers or just an identifier"
                        ))
                    }
                }
            } else {
                match &x[0] {
                        LispVal::Atom(y) => {
                            env.set(
                                y.literal.clone(),
                                LispVal::List(vec![LispVal::List(x[1..].to_vec()), list[2].clone()]),
                            );
                            return Ok(LispVal::List(
                                [
                                    LispVal::String(format!("fn {}", y.literal.clone())),
                                    //LispVal::Atom(y.clone()),
                                    // // LispVal::String(format!("number of parameters {} ", x[1..].len())),
                                    //LispVal::String("parameters:".to_owned()),
                                    LispVal::List(x[1..].to_vec())
                                ]
                                .to_vec(),
                            ));
                        },
                        _ => return Err(format!(
                            "error: first part to define a procedure should be the name of the procedure"
                        ))
                    }
            }
        }

        _ => {
            return Err(format!(
                "define should be a list of identifier or just an identifier"
            ))
        }
    }
}

fn eval_proc(
    proc: &Vec<LispVal>,
    list: &Vec<LispVal>,
    env: &mut Environment,
) -> Result<LispVal, String> {
    let x = match &proc[0] {
        LispVal::List(y) => y.len(),
        _ => return Err("error getting the amount of parameters in procedure".to_owned()),
    };
    if list.len() != x {
        return Err("not enough arguments for procedure".to_owned());
    }
    let mut o_env = env.new_enclosed(env.clone());

    match &proc[0] {
        LispVal::List(x) => {
            for (index, val) in x.iter().enumerate() {
                match val {
                    LispVal::Atom(x) => {
                        o_env.set(x.literal.clone(), list[index].clone());
                    }
                    _ => return Err("should of been an atom".to_owned()),
                }
            }
        }
        _ => return Err("should of been a list".to_owned()),
    }
    eval(proc[1].clone(), &mut o_env)
}

fn eval_lambda(list: &Vec<LispVal>, env: &mut Environment) -> Result<LispVal, String> {
    let mut params: Vec<LispVal> = Vec::<LispVal>::new();
    match &list[0] {
        LispVal::Lamda(x,_) => match *x.clone() {
            LispVal::List(z) => {
                for i in z {
                    params.push(i);
                }
            }
            LispVal::Atom(x) => params.push(LispVal::Atom(x)),
            _ => return Err("error evaluating params for lambda".to_owned()),
        },
        _ => return Err("error evaluating paramaters for lambda".to_owned()),
    }
    if params.len() != list[1..].len() {
        return Err("number of params in lambda do not match the amount applied".to_owned());
    }
    let mut o_env = env.new_enclosed(env.clone());
    for (index, val) in params.iter().enumerate() {
        match val {
            LispVal::Atom(x) => {
                o_env.set(x.literal.clone(), list[index + 1].clone());
            }
            _ => return Err("should of been an atom for lambda".to_owned()),
        }
    }
    match &list[0] {
        LispVal::Lamda(_,y) => return eval(*y.clone(), &mut o_env),
        _ => return Err("error evaluating body for lambda".to_owned()),
    }
}
