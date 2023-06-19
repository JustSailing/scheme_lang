#[derive(Clone, Debug, PartialEq)]
pub struct Token {
    pub literal: String,
    pub kind: TokenType,
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum TokenType {
    ILLEGAL,
    EOF,
    IDENT,
    STRING,
    INT,
    DOT,
    POUND,
    // ASSIGN,
    PLUS,
    MINUS,
    COMMENT, // Single line comment: --comment
    COMMA,
    // BANG,
    ASTERICK,
    SLASH,
    AND,
    OR,
    NOT,
    EQ,
    // NEQ, not sure if this exists in scheme
    LT,
    LTEQ,
    GT,
    GTEQ,
    TRUE,  // #t
    FALSE, // #f
    IF,
    ELSE, // might be implicit in Scheme (it exists in scheme)
    // Beginning of list
    LPAREN,
    RPAREN,
    AT,
    // Used for multiline comments: {- Comment in here -}
    LBRACK,
    RBRACK,
    BEGIN, // evaluates a series of one or more S-Expressions in order
    DEFINE,
    LET, // Takes two arguments first is paired list of variables and values which are
    // scoped to evaluation of the second argument
    LAMBDA, // Anonymous function
    QUOTE,  // Delays the evaluation of its arguments
}

// AST for SCHEME
#[derive(Clone, PartialEq, Debug)]
pub enum LispVal {
    Atom(Token),
    List(Vec<LispVal>),
    DottedList(Vec<LispVal>, Box<LispVal>),
    Number(i64),
    Float(f64),
    String(String),
    Fun(Vec<LispVal>, Vec<LispVal>),
    Lamda(Box<LispVal>, Box<LispVal>),
    Nil,
    Bool(bool),
}

impl LispVal {
    pub fn show_val(&self) -> String {
        match self {
            LispVal::Atom(x) => format!("{} ", x.literal),
            LispVal::List(x) => {
                let mut st: String = String::from("(");
                for str in x {
                    st.push_str(&str.show_val());
                    // st.push_str(" ");
                }
                let _ = st.pop();
                st.push_str(") ");
                st
            }
            LispVal::DottedList(x, y) => {
                let mut st: String = String::from("(");
                for str in x {
                    st.push_str(&str.show_val());
                    // st.push_str(" ");
                }
                st.push_str(y.show_val().as_str());
                st.push_str(")");
                st
            }
            LispVal::Number(x) => {
                let mut st = x.to_string();
                st.push_str(" ");
                st
            }
            LispVal::Float(x) => {
                let mut st = x.to_string();
                st.push_str(" ");
                st
            }
            LispVal::String(x) => {
                let mut st: String = String::from("");
                st.push_str(x.as_str());
                st.push_str(" ");
                st
            }
            LispVal::Fun(_, _) => "(internal function)".to_owned(),
            LispVal::Lamda(_, _) => "(lambda function)".to_owned(),
            LispVal::Nil => "Nil".to_owned(),
            LispVal::Bool(x) => {
                if *x == false {
                    "#f ".to_owned()
                } else {
                    "#t ".to_owned()
                }
            }
        }
    }
}
