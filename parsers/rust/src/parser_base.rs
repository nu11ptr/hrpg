use crate::tokens::{Token, TokenType, Tokenizer};

struct Parser<LEX: Tokenizer<TOK>, TOK: Token> {
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
