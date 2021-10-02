use crate::ast::Grammar;
use std::any::Any;

#[derive(Clone, Debug)]
enum Type {
    String,
    Int,
    Custom(String),
}

#[derive(Clone, Debug)]
enum Expr {
    FuncCall { name: String },
    String(String),
    Int(i32),
}

#[derive(Clone, Debug)]
enum Stmt {
    Const { name: String, type_val: Box<Stmt> },
    VarBind { name: String, type_val: Box<Stmt> },
    TypeValue { type_: Type, value: Expr },
}

#[derive(Clone, Debug)]
struct Struct {
    name: String,
    comment: String,
    instance_vars: Vec<Stmt>,
    constructor: Box<Decl>,
    methods: Vec<Decl>,
}

#[derive(Clone, Debug)]
struct Func {
    name: String,
    comment: String,
    func_args: Vec<Stmt>,
    ret_type: Type,
}

#[derive(Clone, Debug)]
enum Decl {
    Struct(Struct),
    Func(Func),
}

struct ParserGen {}

impl ParserGen {
    pub fn generate(grammar: Grammar) -> Vec<Decl> {
        todo!()
    }
}
