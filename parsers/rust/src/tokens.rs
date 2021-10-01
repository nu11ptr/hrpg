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

pub trait Token: Clone {
    fn token_type(&self) -> TokenType;
}

pub trait Tokenizer<TOK: Token> {
    fn next_token(&self) -> TOK;
}
