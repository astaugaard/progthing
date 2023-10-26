use std::{iter, rc::Rc};

use pom::utf8::{seq, *, sym};

// todo switch from pom for better error messages

#[derive(Debug, Clone)]
pub struct Decleration {
    pub public: bool,
    pub value: DeclerationVal,
}

#[derive(Debug, Clone)]
pub enum DeclerationVal {
    Import(Vec<String>),
    Function(String, Vec<String>, Expr),
    Value(String, Expr),
    Adt(String, Vec<EnumVariant>),
}

#[derive(Debug, Clone)]
pub enum Statement {
    Return(Expr),
    Assign(String, Expr),
}

// binding order lambda -> Operator -> Function -> Values(GroupExpr,Number,Float,StrLiteral,Char,Array,parens)
#[derive(Debug, Clone)]
pub enum Expr {
    // when parsing the operators are stored as function calls
    // I am not going to do operator precedence in the parser because I want to be able to have the ability to make custom operators
    Function(Rc<Expr>, Vec<Rc<Expr>>),
    GroupExpr(Vec<Rc<Statement>>),
    Lambda(Vec<String>, Rc<Expr>),
    Case(Vec<(String, Vec<CaseExprs>, Expr)>),
    Number(i64),
    Float(f64),
    StrLiteral(String),
    Char(char),
    Array(Vec<Rc<Expr>>),
    Variable(String),
}

#[derive(Debug, Clone)]
pub enum CaseExprs {
    Variable(String),
    CaseSplit(String, Vec<String>),
}

#[derive(Debug, Clone)]
pub struct EnumVariant {
    pub name: String,
    pub params: Vec<String>,
}

pub type File = Vec<Decleration>;

fn spaces<'a>() -> Parser<'a, ()> {
    is_a(|a: char| a.is_whitespace()).repeat(0..).map(|_a| ())
}

fn token<'a>(token: &'a str) -> Parser<'a, ()> {
    seq(token) * spaces()
}

fn decleration<'a>() -> Parser<'a, Decleration> {
    // println!("in delc");
    let public = token("pub").opt().map(|a| a.is_some());
    let decleration_var = import() | function() | value(); // | adt();

    (public + decleration_var).map(|(public, value)| Decleration { public, value })
}

// fn adt<'a>() -> Parser<'a, DeclerationVal> {
//     println!("in adt");
//     fail()
// }

fn value<'a>() -> Parser<'a, DeclerationVal> {
    // println!("in value");
    (token("let") * variable_name() - token("=").expect("=")  + group_expr())
        .map(|(name, value)| DeclerationVal::Value(name, value))
}

fn function<'a>() -> Parser<'a, DeclerationVal> {
    // println!("in function");
    (token("fun") * variable_name().expect("variable name")
        + (token("(") * sep_by1(variable_name, token(",")) - token(")"))
        + group_expr())
    .map(|((name, args), expr)| DeclerationVal::Function(name, args, expr))
    .name("function")
}

fn statement<'a>() -> Parser<'a, Statement> {
    (token("let") * (variable_name() - token("=") + expr()).expect("variable delc") )
        .map(|(name, value)| Statement::Assign(name, value))
        | expr().map(Statement::Return)
}

fn group_expr<'a>() -> Parser<'a, Expr> {
    // println!("in assign function");
    (token("{")
        * (statement() - token(";"))
            .repeat(1..)
            .map(|exprs| Expr::GroupExpr(exprs.iter().map(|st| Rc::new(st.clone())).collect()))
        - token("}"))
    .name("assign expression")
}

fn expr<'a>() -> Parser<'a, Expr> {
    // println!("in expr");
    (
        (token("\\") * (variable_name()).repeat(1..) - token("->") + call(expr))
            .map(|(args, expr)| Expr::Lambda(args, Rc::new(expr)))
            | expr1()
        // | caseexpr()
    )
    .name("expression")
}

// fn caseexpr<'a>() -> Parser<'a, Expr> {
//     todo!()
// }

fn expr1<'a>() -> Parser<'a, Expr> {
    (expr2() + (operator() + call(expr1)).opt())
        .map(|(e1, a)| match a {
            Some((o, e2)) => {
                Expr::Function(Rc::new(Expr::Variable(o)), vec![Rc::new(e1), Rc::new(e2)])
            }
            None => e1,
        })
        .name("expression")
}

fn operator<'a>() -> Parser<'a, String> {
    let operator_token = one_of("+-&^%$@!=:<>\\/*");
    (operator_token.repeat(1..).map(|a| a.iter().collect()) - spaces()).name("operator")
}

fn expr2<'a>() -> Parser<'a, Expr> {
    // println!("in expr2");
    (expr3() + expr3().repeat(0..))
        .map(|(e1, args)| {
            if args.len() == 0 {
                e1
            } else {
                Expr::Function(
                    Rc::new(e1),
                    args.iter().map(|a| Rc::new(a.clone())).collect(),
                )
            }
        })
        .name("expression")
}

fn expr3<'a>() -> Parser<'a, Expr> {
    // println!("in expr3");
    (token("(") * call(expr) - token(")"))
        | variable_name().map(Expr::Variable)
        | array()
        | number()
        | parse_string()
        | parse_char()
}

fn parse_char<'a>() -> Parser<'a, Expr> {
    sym('\'') * char_parser(true).map(Expr::Char) - sym('\'') - spaces()
}

fn char_parser<'a>(char_literal: bool) -> Parser<'a, char> {
    not_a(move |a| if char_literal { a == '\'' } else { a == '"' }) // todo
}

fn parse_string<'a>() -> Parser<'a, Expr> {
    sym('"')
        * char_parser(false)
            .repeat(0..)
            .map(|a| Expr::StrLiteral(a.iter().collect()))
        - sym('"')
        - spaces()
}

fn number<'a>() -> Parser<'a, Expr> {
    ((sym('-').opt().map(|a| a.is_some()))
        + is_a(|a| a.is_digit(10)).repeat(1..)
        + (sym('.') * is_a(|a| a.is_digit(10)).repeat(1..)).opt())
    .map(|((neg, a), b)| match b {
        Some(b) => {
            Expr::Float(if neg { -1.0 } else { 1.0 } * chars_to_number(a) as f64 + chars_to_decimal(b))
        }
        None => Expr::Number(if neg { -1 } else { 1 } * chars_to_number(a)),
    }) - spaces()
}

fn chars_to_decimal(b: Vec<char>) -> f64 {
    let mut decimal: f64 = 1.0;
    let mut acc: f64 = 0.0;
    for d in b.iter() {
        match d.to_digit(10) {
            Some(num) => {
                decimal /= 10.0;
                acc += decimal * (num as f64);
            }
            None => {
                panic!("only numbers");
            }
        }
    }
    return acc;
}

fn chars_to_number(a: Vec<char>) -> i64 {
    let mut number: i64 = 0;
    for d in a.iter() {
        match d.to_digit(10) {
            Some(num) => {
                number = number * 10 + num as i64;
            }
            None => {
                panic!("only numbers");
            }
        }
    }
    return number;
}

fn array<'a>() -> Parser<'a, Expr> {
    // println!("in array");
    token("[")
        * sep_by(expr, token(","))
            .map(|a| Expr::Array(a.iter().map(|b| Rc::new(b.clone())).collect()))
        - token("]")
}

fn import<'a>() -> Parser<'a, DeclerationVal> {
    // println!("in import");
    (token("use") * sep_by1(variable_name, token(".")).map(|a| DeclerationVal::Import(a))
        - token(";"))
    .name("import")
}

fn sep_by1<'a, B>(variable_name: fn() -> Parser<'a, B>, token: Parser<'a, ()>) -> Parser<'a, Vec<B>>
where
    B: Clone + 'a,
{
    (call(variable_name) + (token * call(variable_name)).repeat(0..)).map(|(a, mut b)| {
        b.insert(0, a);
        b
    })
}

fn sep_by<'a, B>(variable_name: fn() -> Parser<'a, B>, token: Parser<'a, ()>) -> Parser<'a, Vec<B>>
where
    B: Clone + 'a,
{
    sep_by1(variable_name, token).opt().map(|opt| opt.unwrap_or(vec![]))
}

fn variable_name<'a>() -> Parser<'a, String> {
    let variable_token = is_a(|a| a.is_alphanumeric());
    ((is_a(|a| a.is_alphabetic()) + variable_token.repeat(0..))
        .map(|(a, b)| iter::once(&a).chain(b.iter()).collect())
        - spaces())
    .name("variable")
}

pub fn parser<'a>() -> Parser<'a, File> {
    spaces() * (decleration().expect("").repeat(0..)) - end::<char>()
}
