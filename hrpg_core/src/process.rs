use std::collections::{HashMap, HashSet};

use crate::ast::{Grammar, Node};

const EOF: &str = "EOF";
const ILLEGAL: &str = "ILLEGAL";

pub struct Process {
    pub token_names: HashSet<String>,
    literals: HashMap<String, (String, Option<Node>)>,
    pub errors: Vec<String>,
}

fn strip_quotes(str: &str) -> String {
    let mut chars = str.chars();
    chars.next();
    chars.next_back();
    chars.as_str().to_string()
}

impl Process {
    pub fn new() -> Process {
        let mut token_names = HashSet::new();
        token_names.insert(EOF.to_string());
        token_names.insert(ILLEGAL.to_string());

        Process { token_names, literals: HashMap::new(), errors: vec![] }
    }

    pub fn process(&mut self, grammar: &Grammar) -> Grammar {
        let token_rules = &grammar.token_rules;
        let parser_rules = &grammar.parser_rules;

        let token_rules: Vec<Node> = token_rules
            .iter()
            .map(|node| { self.process_token_rule(node) })
            .collect();
        let parser_rules: Vec<Node> = parser_rules
            .iter()
            .map(|node| { self.process_parser_rule(node) })
            .collect();
        Grammar { parser_rules: token_rules, token_rules: parser_rules }
    }

    fn log_error(&mut self, msg: &str) {
        self.errors.push(format!("ERROR: {}", msg));
    }

    fn process_token_rule(&mut self, node: &Node) -> Node {
        match node {
            Node::TokenRule { name, literal } => {
                let lit = match literal.as_ref() {
                    Node::TokenLit { literal } => strip_quotes(literal),
                    _ => unreachable!()
                };
                self.literals.insert(lit, (name.to_string(), None));
                self.token_names.insert(name.to_string());

                node.clone()
            }
            _ => unreachable!()
        }
    }

    fn process_parser_rule(&mut self, node: &Node) -> Node {
        match node {
            Node::ParserRule { name, node } => Node::ParserRule {
                name: name.to_string(),
                node: Box::new(self.process_node(node)),
            },
            _ => unreachable!()
        }
    }

    fn process_node(&mut self, node: &Node) -> Node {
        match node {
            Node::Binding { name, node } => Node::Binding {
                name: name.to_string(),
                node: Box::new(self.process_node(node)),
            },
            Node::Alternatives { nodes } => Node::Alternatives {
                nodes: nodes.iter().map(|node| { self.process_node(node) }).collect()
            },
            Node::MultipartBody { nodes } => Node::MultipartBody {
                nodes: nodes.iter().map(|node| { self.process_node(node) }).collect()
            },
            Node::ZeroOrMore { node } => Node::ZeroOrMore {
                node: Box::new(self.process_node(node))
            },
            Node::OneOrMore { node } => Node::OneOrMore {
                node: Box::new(self.process_node(node))
            },
            Node::ZeroOrOne { node, brackets } => Node::ZeroOrOne {
                node: Box::new(self.process_node(node)),
                brackets: *brackets,
            },
            Node::RuleRef { .. } => node.clone(),
            Node::TokenRef { name, replaced_lit: _replaced_lit } => {
                self.token_names.insert(name.to_string());
                node.clone()
            }
            Node::TokenLit { literal } => {
                // Strip quotes and use as lookup key
                let lit = strip_quotes(literal);

                // Try and find the literal to ensure it has a corresponding rule
                match self.literals.get(&lit).cloned() {
                    Some((_name, Some(token_ref))) => token_ref.clone(),
                    Some((name, None)) => {
                        let token_ref = Node::TokenRef {
                            name: name.to_string(),
                            replaced_lit: Some(literal.to_string()),
                        };
                        let ref_copy = token_ref.clone();
                        self.literals.insert(lit, (name.to_string(), Some(token_ref)));
                        ref_copy
                    }
                    None => {
                        self.log_error(&format!("Literal {} does not have corresponding rule", literal));
                        node.clone()
                    }
                }
            }
            _ => unreachable!()
        }
    }
}
