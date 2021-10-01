use crate::tokens::{Token, TokenType, Tokenizer, TreeNode};
use parser::memoize;
use std::collections::HashMap;

pub struct Parser<LEX: Tokenizer<TOK>, TOK: Token> {
    tokenizer: LEX,
    pos: usize,
    tokens: Vec<TOK>,
}

impl<LEX: Tokenizer<TOK>, TOK: Token> Parser<LEX, TOK> {
    pub fn new(tokenizer: LEX) -> Parser<LEX, TOK> {
        let mut parser = Parser {
            tokenizer,
            pos: 0,
            tokens: vec![],
        };
        parser.next_token();
        parser
    }

    #[inline]
    fn curr_token(&self) -> TOK {
        self.tokens[self.pos].clone()
    }

    fn next_token(&mut self) -> TOK {
        self.pos += 1;

        if self.pos < self.tokens.len() {
            self.tokens[self.pos].clone()
        } else {
            let tok = self.tokenizer.next_token();
            self.tokens.push(tok.clone());
            tok
        }
    }

    fn match_token_or_rollback(&mut self, tt: TokenType, old_pos: usize) -> Option<TOK> {
        let tok = self.curr_token();

        if tok.token_type() == tt {
            self.next_token();
            Some(tok)
        } else {
            self.pos = old_pos;
            None
        }
    }

    fn match_tokens_or_rollback(&mut self, tt: TokenType, old_pos: usize) -> Vec<TOK> {
        match self.match_token_or_rollback(tt, old_pos) {
            Some(tok) => {
                let mut tokens = self.try_match_tokens(tt);
                tokens.insert(0, tok);
                tokens
            }
            None => {
                self.pos = old_pos;
                vec![]
            }
        }
    }

    fn try_match_token(&mut self, tt: TokenType) -> Option<TOK> {
        let tok = self.curr_token();

        if tok.token_type() == tt {
            self.next_token();
            Some(tok)
        } else {
            None
        }
    }

    fn try_match_tokens(&mut self, tt: TokenType) -> Vec<TOK> {
        let mut tokens = vec![];

        loop {
            match self.try_match_token(tt) {
                Some(tok) => tokens.push(tok),
                None => break,
            }
        }

        tokens
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
enum Func {
    ParseValue,
    ParseDict,
    ParseList,
}

#[derive(Clone)]
pub enum TokenOrNode<TOK: Token, NODE: TreeNode> {
    Token(TOK),
    Node(NODE),
    None,
}

impl<TOK: Token, NODE: TreeNode> TokenOrNode<TOK, NODE> {
    pub fn into_option(self) -> Option<TokenOrNode<TOK, NODE>> {
        match self {
            TokenOrNode::Token(_) => Some(self),
            TokenOrNode::Node(_) => Some(self),
            TokenOrNode::None => None,
        }
    }
}

pub struct JSONParser<LEX: Tokenizer<TOK>, TOK: Token, NODE: TreeNode> {
    parser: Parser<LEX, TOK>,
    memos: HashMap<(Func, usize), (TokenOrNode<TOK, NODE>, usize)>,
}

impl<LEX: Tokenizer<TOK>, TOK: Token, NODE: TreeNode> JSONParser<LEX, TOK, NODE> {
    // value: dict | list | STRING | NUMBER | 'true' | 'false' | 'null'
    #[memoize]
    pub fn parse_value(&mut self) -> TokenOrNode<TOK, NODE> {
        let old_pos = self.parser.pos;

        // dict
        if let Some(dict) = self.parse_dict().into_option() {
            dict
        // list
        } else if let Some(list) = self.parse_list().into_option() {
            list
        // STRING
        } else if let Some(string) = self.parser.try_match_token(TokenType::String) {
            TokenOrNode::Token(string)
        // NUMBER
        } else if let Some(number) = self.parser.try_match_token(TokenType::Number) {
            TokenOrNode::Token(number)
        // 'true'
        } else if let Some(true_) = self.parser.try_match_token(TokenType::True) {
            TokenOrNode::Token(true_)
        // 'false'
        } else if let Some(false_) = self.parser.try_match_token(TokenType::False) {
            TokenOrNode::Token(false_)
        // 'null'
        } else if let Some(null) = self.parser.try_match_token(TokenType::Null) {
            TokenOrNode::Token(null)
        } else {
            self.parser.pos = old_pos;
            TokenOrNode::None
        }
    }

    // dict: '{' [pair (',' pair)*] '}'
    #[memoize]
    pub fn parse_dict(&self) -> TokenOrNode<TOK, NODE> {
        TokenOrNode::None
    }

    // list: '[' [value (',' value)*] ']'
    #[memoize]
    pub fn parse_list(&self) -> TokenOrNode<TOK, NODE> {
        TokenOrNode::None
    }
}
