#[derive(Copy, Clone, Debug, PartialEq)]
pub enum TokenType {
    True,
    False,
    Null,
    LBracket,
    RBracket,
    LBrace,
    RBrace,
    Comma,
    Colon,
    String,
    Number,
    EOF,
    Illegal,
}

pub trait TreeNode {}

pub trait Token: TreeNode + Clone {
    fn token_type(&self) -> TokenType;
}

pub trait Tokenizer<TOK: Token> {
    fn next_token(&self) -> TOK;
}
