extern crate pest;

use pest::Parser;

trait Comment {
    fn comment(&self) -> String;
}

// *** Rules ***

enum RuleType {
    Parser(ParserRule),
    Token(TokenRule),
}

impl From<ParserRule> for RuleType {
    fn from(rule: ParserRule) -> Self {
        RuleType::Parser(rule)
    }
}

impl From<TokenRule> for RuleType {
    fn from(rule: TokenRule) -> Self {
        RuleType::Token(rule)
    }
}

// parser_rule
#[derive(Clone, Debug)]
pub struct ParserRule {
    pub name: String,
    pub node: Node,
}

// token_rule
#[derive(Clone, Debug)]
pub struct TokenRule {
    pub name: String,
    pub literal: Node,
}

// *** Nodes ***

// rule_body
#[derive(Clone, Debug)]
pub struct Alternatives {
    pub nodes: Vec<Node>,
}

impl Comment for Alternatives {
    #[inline]
    fn comment(&self) -> String {
        let comments: Vec<String> = self.nodes.iter().map(|node| node.comment()).collect();
        comments.join(" | ")
    }
}

// rule_piece
#[derive(Clone, Debug)]
pub struct MultipartBody {
    pub nodes: Vec<Node>,
}

impl Comment for MultipartBody {
    fn comment(&self) -> String {
        let comments: Vec<String> = self
            .nodes
            .iter()
            .map(|node| match node {
                Node::Alt(_) => format!("({})", node.comment()),
                _ => node.comment(),
            })
            .collect();
        comments.join(" ")
    }
}

// binding
#[derive(Clone, Debug)]
pub struct Binding {
    pub name: String,
    pub node: Box<Node>,
}

impl Comment for Binding {
    #[inline]
    fn comment(&self) -> String {
        format!("{}={}", self.name, self.node.comment())
    }
}

// rule_part
#[derive(Clone, Debug)]
pub struct ZeroOrMore {
    pub node: Box<Node>,
}

impl Comment for ZeroOrMore {
    fn comment(&self) -> String {
        match self.node.as_ref() {
            // Regular Nodes
            Node::RulRef(_) | Node::TokRef(_) | Node::TokLit(_) => {
                format!("{}*", self.node.comment())
            }
            // Containers
            _ => format!("({})*", self.node.comment()),
        }
    }
}

// rule_part
#[derive(Clone, Debug)]
pub struct OneOrMore {
    pub node: Box<Node>,
}

impl Comment for OneOrMore {
    fn comment(&self) -> String {
        match self.node.as_ref() {
            // Regular Nodes
            Node::RulRef(_) | Node::TokRef(_) | Node::TokLit(_) => {
                format!("{}+", self.node.comment())
            }
            // Containers
            _ => format!("({})+", self.node.comment()),
        }
    }
}

// rule_part
#[derive(Clone, Debug)]
pub struct ZeroOrOne {
    pub node: Box<Node>,
    pub brackets: bool,
}

impl Comment for ZeroOrOne {
    fn comment(&self) -> String {
        if self.brackets {
            format!("[{}]", self.node.comment())
        } else {
            match self.node.as_ref() {
                // Regular Nodes
                Node::RulRef(_) | Node::TokRef(_) | Node::TokLit(_) => {
                    format!("{}?", self.node.comment())
                }
                // Containers
                _ => format!("({})?", self.node.comment()),
            }
        }
    }
}

// RULE_NAME
#[derive(Clone, Debug)]
pub struct RuleRef {
    pub name: String,
}

impl Comment for RuleRef {
    #[inline]
    fn comment(&self) -> String {
        self.name.to_owned()
    }
}

// TOKEN_NAME
#[derive(Clone, Debug)]
pub struct TokenRef {
    pub name: String,
    pub replaced_lit: Option<String>,
}

impl Comment for TokenRef {
    fn comment(&self) -> String {
        (match &self.replaced_lit {
            Some(lit) => lit,
            None => &self.name,
        })
        .to_owned()
    }
}

// TOKEN_LIT
#[derive(Clone, Debug)]
pub struct TokenLit {
    pub literal: String,
}

impl Comment for TokenLit {
    #[inline]
    fn comment(&self) -> String {
        format!("\"{}\"", self.literal)
    }
}

#[derive(Clone, Debug)]
pub enum Node {
    Alt(Alternatives),
    Multi(MultipartBody),

    Bind(Binding),

    ZoM(ZeroOrMore),
    OoM(OneOrMore),
    ZoO(ZeroOrOne),

    RulRef(RuleRef),
    TokRef(TokenRef),
    TokLit(TokenLit),
}

impl Comment for Node {
    fn comment(&self) -> String {
        use Node::*;

        match self {
            Alt(a) => a.comment(),
            Multi(m) => m.comment(),

            Bind(b) => b.comment(),

            ZoM(z) => z.comment(),
            OoM(o) => o.comment(),
            ZoO(z) => z.comment(),

            RulRef(r) => r.comment(),
            TokRef(t) => t.comment(),

            TokLit(t) => t.comment(),
        }
    }
}

impl From<Alternatives> for Node {
    fn from(alt: Alternatives) -> Self {
        Node::Alt(alt)
    }
}

impl From<MultipartBody> for Node {
    fn from(mult: MultipartBody) -> Self {
        Node::Multi(mult)
    }
}

impl From<Binding> for Node {
    fn from(bind: Binding) -> Self {
        Node::Bind(bind)
    }
}

impl From<ZeroOrMore> for Node {
    fn from(zero: ZeroOrMore) -> Self {
        Node::ZoM(zero)
    }
}

impl From<OneOrMore> for Node {
    fn from(one: OneOrMore) -> Self {
        Node::OoM(one)
    }
}

impl From<ZeroOrOne> for Node {
    fn from(zero: ZeroOrOne) -> Self {
        Node::ZoO(zero)
    }
}

impl From<RuleRef> for Node {
    fn from(rr: RuleRef) -> Self {
        Node::RulRef(rr)
    }
}

impl From<TokenRef> for Node {
    fn from(tr: TokenRef) -> Self {
        Node::TokRef(tr)
    }
}

impl From<TokenLit> for Node {
    fn from(tok: TokenLit) -> Self {
        Node::TokLit(tok)
    }
}

// top_level
#[derive(Debug)]
pub struct Grammar {
    pub parser_rules: Vec<ParserRule>,
    pub token_rules: Vec<TokenRule>,
}

#[derive(pest_derive::Parser)]
#[grammar = "HRPG.pest"]
struct HRPGParser;

pub fn parse_hrpg(data: &str) -> Result<Grammar, pest::error::Error<Rule>> {
    let nodes: Vec<RuleType> = HRPGParser::parse(Rule::top_level, data)?
        .next()
        .unwrap()
        .into_inner()
        .filter(|p| p.as_rule() == Rule::entry)
        .map(parse_rule_type)
        .collect();

    let mut parser_rules: Vec<ParserRule> = vec![];
    let mut token_rules: Vec<TokenRule> = vec![];

    for node in nodes {
        match node {
            RuleType::Parser(rule) => parser_rules.push(rule),
            RuleType::Token(rule) => token_rules.push(rule),
        }
    }

    Ok(Grammar {
        parser_rules,
        token_rules,
    })
}

fn parse_rule_type(pair: pest::iterators::Pair<Rule>) -> RuleType {
    match pair.as_rule() {
        Rule::entry => parse_rule_type(pair.into_inner().next().unwrap()),
        Rule::parse_rule => {
            let mut inner_rules = pair.into_inner();
            let rule_name = inner_rules.next().unwrap().as_str().to_owned();
            let rule_body = parse_node(inner_rules.next().unwrap());
            ParserRule {
                name: rule_name,
                node: rule_body,
            }
            .into()
        }

        Rule::token_rule => {
            let mut inner = pair.into_inner();
            let token_name = inner.next().unwrap().as_str().to_owned();
            let token_lit = parse_node(inner.next().unwrap());
            TokenRule {
                name: token_name,
                literal: token_lit,
            }
            .into()
        }

        _ => unreachable!(),
    }
}

fn parse_node(pair: pest::iterators::Pair<Rule>) -> Node {
    match pair.as_rule() {
        Rule::rule_body => {
            let mut nodes: Vec<Node> = pair.into_inner().map(parse_node).collect();
            match nodes.len() {
                1 => nodes.remove(0),
                _ => Alternatives { nodes }.into(),
            }
        }

        Rule::rule_piece => {
            let mut inner_rules = pair.into_inner();
            // Make a copy of our iterator in case there is no binding
            let saved_inner_rules = inner_rules.clone();
            let first_inner = inner_rules.next().unwrap();

            fn process_rules(inner_rules: pest::iterators::Pairs<Rule>) -> Node {
                let mut nodes: Vec<Node> = inner_rules.map(parse_node).collect();

                match nodes.len() {
                    1 => nodes.remove(0),
                    _ => MultipartBody { nodes }.into(),
                }
            }

            // Do we have a binding and rules or just rules?
            match first_inner.as_rule() {
                Rule::rule_name => Binding {
                    name: first_inner.as_str().to_owned(),
                    node: Box::new(process_rules(inner_rules)),
                }
                .into(),
                _ => process_rules(saved_inner_rules),
            }
        }

        Rule::rule_part => {
            let mut inner_rules = pair.clone().into_inner();
            let first_inner = inner_rules.next().unwrap();
            let node = parse_node(first_inner.clone());

            match first_inner.as_rule() {
                Rule::rule_elem => match pair.as_str().chars().last() {
                    Some('+') => OneOrMore {
                        node: Box::new(node),
                    }
                    .into(),
                    Some('*') => ZeroOrMore {
                        node: Box::new(node),
                    }
                    .into(),
                    Some('?') => ZeroOrOne {
                        node: Box::new(node),
                        brackets: false,
                    }
                    .into(),
                    _ => node,
                },
                Rule::rule_body => ZeroOrOne {
                    node: Box::new(node),
                    brackets: true,
                }
                .into(),
                _ => unreachable!(),
            }
        }

        Rule::rule_elem => parse_node(pair.into_inner().next().unwrap()),

        Rule::rule_name => RuleRef {
            name: pair.as_str().to_owned(),
        }
        .into(),

        Rule::token_name => TokenRef {
            name: pair.as_str().to_owned(),
            replaced_lit: None,
        }
        .into(),

        Rule::token_lit => TokenLit {
            literal: pair.as_str().to_owned(),
        }
        .into(),

        _ => unreachable!(),
    }
}
