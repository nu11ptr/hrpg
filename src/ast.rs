extern crate pest;

use pest::error::Error;
use pest::iterators::Pair;
use pest::Parser;

use self::pest::iterators::Pairs;
use Node::*;

trait Comment {
    fn comment(&self) -> String;
}

#[derive(Clone, Debug)]
pub enum Node {
    Binding {
        name: String,
        node: Box<Node>,
    },

    // rule_body
    Alternatives {
        nodes: Vec<Node>,
    },
    // rule_piece
    MultipartBody {
        nodes: Vec<Node>,
    },
    // rule_part
    ZeroOrMore {
        node: Box<Node>,
    },
    // rule_part
    OneOrMore {
        node: Box<Node>,
    },
    // rule_part
    ZeroOrOne {
        node: Box<Node>,
        brackets: bool,
    },

    // RULE_NAME
    RuleRef {
        name: String,
    },
    // TOKEN_NAME
    TokenRef {
        name: String,
        replaced_lit: Option<String>,
    },
    // TOKEN_LIT
    TokenLit {
        literal: String,
    },

    // parser_rule
    ParserRule {
        name: String,
        node: Box<Node>,
    },
    // token_rule
    TokenRule {
        name: String,
        literal: Box<Node>,
    },
}

impl Comment for Node {
    fn comment(&self) -> String {
        match self {
            Binding { name, node } => format!("{}={}", name, node.comment()),

            Alternatives { nodes } => {
                let comments: Vec<String> = nodes.iter().map(|node| node.comment()).collect();
                comments.join(" | ")
            }

            MultipartBody { nodes } => {
                let comments: Vec<String> = nodes
                    .iter()
                    .map(|node| match node {
                        Alternatives { .. } => format!("({})", node.comment()),
                        _ => node.comment(),
                    })
                    .collect();
                comments.join(" ")
            }

            ZeroOrMore { node } => match node.as_ref() {
                // Regular Nodes
                RuleRef { .. } | TokenRef { .. } | TokenLit { .. } => {
                    format!("{}*", node.comment())
                }
                // Containers
                _ => format!("({})*", node.comment()),
            },

            OneOrMore { node } => match node.as_ref() {
                // Regular Nodes
                RuleRef { .. } | TokenRef { .. } | TokenLit { .. } => {
                    format!("{}+", node.comment())
                }
                // Containers
                _ => format!("({})+", node.comment()),
            },

            ZeroOrOne { node, brackets } => {
                if *brackets {
                    format!("[{}]", node.comment())
                } else {
                    match node.as_ref() {
                        // Regular Nodes
                        RuleRef { .. } | TokenRef { .. } | TokenLit { .. } => {
                            format!("{}?", node.comment())
                        }
                        // Containers
                        _ => format!("({})?", node.comment()),
                    }
                }
            }

            RuleRef { name } => name.to_owned(),

            TokenRef { name, replaced_lit } => (match replaced_lit {
                Some(lit) => lit,
                None => name,
            })
            .to_owned(),

            TokenLit { literal } => format!("\"{}\"", literal),

            ParserRule { name, node } => format!("{}: {}", name, node.comment()),

            TokenRule { name, literal } => format!("{}: {}", name, literal.comment()),
        }
    }
}

// top_level
#[derive(Debug)]
pub struct Grammar {
    pub parser_rules: Vec<Node>,
    pub token_rules: Vec<Node>,
}

#[derive(Parser)]
#[grammar = "HRPG.pest"]
struct HRPGParser;

pub fn parse_hrpg(data: &str) -> Result<Grammar, Error<Rule>> {
    let nodes: Vec<Node> = HRPGParser::parse(Rule::top_level, data)?
        .next()
        .unwrap()
        .into_inner()
        .filter(|p| p.as_rule() == Rule::entry)
        .map(parse_node)
        .collect();

    let mut parser_rules: Vec<Node> = vec![];
    let mut token_rules: Vec<Node> = vec![];

    for node in nodes {
        match node {
            ParserRule { .. } => parser_rules.push(node),
            TokenRule { .. } => token_rules.push(node),
            _ => unreachable!(),
        }
    }

    Ok(Grammar {
        parser_rules,
        token_rules,
    })
}

fn parse_node(pair: Pair<Rule>) -> Node {
    match pair.as_rule() {
        Rule::entry => parse_node(pair.into_inner().next().unwrap()),
        Rule::parse_rule => {
            let mut inner_rules = pair.into_inner();
            let rule_name = inner_rules.next().unwrap().as_str().to_owned();
            let rule_body = parse_node(inner_rules.next().unwrap());
            ParserRule {
                name: rule_name,
                node: Box::new(rule_body),
            }
        }
        Rule::rule_body => {
            let mut nodes: Vec<Node> = pair.into_inner().map(parse_node).collect();
            match nodes.len() {
                1 => nodes.remove(0),
                _ => Alternatives { nodes },
            }
        }
        Rule::rule_piece => {
            let mut inner_rules = pair.into_inner();
            // Make a copy of our iterator in case there is no binding
            let saved_inner_rules = inner_rules.clone();
            let first_inner = inner_rules.next().unwrap();

            fn process_rules(inner_rules: Pairs<Rule>) -> Node {
                let mut nodes: Vec<Node> = inner_rules.map(parse_node).collect();

                match nodes.len() {
                    1 => nodes.remove(0),
                    _ => MultipartBody { nodes },
                }
            }

            // Do we have a binding and rules or just rules?
            match first_inner.as_rule() {
                Rule::rule_name => Binding {
                    name: first_inner.as_str().to_owned(),
                    node: Box::new(process_rules(inner_rules)),
                },
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
                    },
                    Some('*') => ZeroOrMore {
                        node: Box::new(node),
                    },
                    Some('?') => ZeroOrOne {
                        node: Box::new(node),
                        brackets: false,
                    },
                    _ => node,
                },
                Rule::rule_body => ZeroOrOne {
                    node: Box::new(node),
                    brackets: true,
                },
                _ => unreachable!(),
            }
        }
        Rule::rule_elem => parse_node(pair.into_inner().next().unwrap()),
        Rule::token_rule => {
            let mut inner = pair.into_inner();
            let token_name = inner.next().unwrap().as_str().to_owned();
            let token_lit = parse_node(inner.next().unwrap());
            TokenRule {
                name: token_name,
                literal: Box::new(token_lit),
            }
        }
        Rule::rule_name => RuleRef {
            name: pair.as_str().to_owned(),
        },
        Rule::token_name => TokenRef {
            name: pair.as_str().to_owned(),
            replaced_lit: None,
        },
        Rule::token_lit => TokenLit {
            literal: pair.as_str().to_owned(),
        },
        _ => unreachable!(),
    }
}
