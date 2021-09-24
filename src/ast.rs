trait Comment {
    fn comment(&self) -> String;
}

pub enum Node {
    Binding { name: String, node: Box<Node> },

    // rule_body
    Alternatives { nodes: Vec<Box<Node>> },
    // rule_piece
    MultipartBody { nodes: Vec<Box<Node>> },
    // rule_part
    ZeroOrMore { node: Box<Node> },
    // rule_part
    OneOrMore { node: Box<Node> },
    // rule_part
    ZeroOrOne { node: Box<Node>, brackets: bool },

    // RULE_NAME
    RuleRef { name: String },
    // TOKEN_NAME
    TokenRef { name: String, replaced_lit: Option<String> },
    // TOKEN_LIT
    TokenLit { literal: String },
}

impl Comment for Node {
    fn comment(&self) -> String {
        match self {
            Node::Binding { name, node } => format!("{}={}", name, node.comment()),

            Node::Alternatives { nodes } => {
                let comments: Vec<String> = nodes.into_iter().map(|node| { node.comment() }).collect();
                comments.join(" | ")
            }

            Node::MultipartBody { nodes } => {
                let comments: Vec<String> = nodes.into_iter().map(|node| {
                    match node.as_ref() {
                        Node::Alternatives { .. } => format!("({})", node.comment()),
                        _ => node.comment(),
                    }
                }).collect();
                comments.join(" ")
            }

            Node::ZeroOrMore { node } => match node.as_ref() {
                Node::RuleRef { .. } | Node::TokenRef { .. } | Node::TokenLit { .. } =>
                    format!("{}*", node.comment()),
                _ => format!("({})*", node.comment())
            },

            Node::OneOrMore { node } => match node.as_ref() {
                Node::RuleRef { .. } | Node::TokenRef { .. } | Node::TokenLit { .. } =>
                    format!("{}+", node.comment()),
                _ => format!("({})+", node.comment())
            },

            Node::ZeroOrOne { node, brackets } =>
                if *brackets { format!("[{}]", node.comment()) } else {
                    match node.as_ref() {
                        Node::RuleRef { .. } | Node::TokenRef { .. } | Node::TokenLit { .. } =>
                            format!("{}?", node.comment()),
                        _ => format!("({})?", node.comment())
                    }
                },

            Node::RuleRef { name } => name.to_string(),

            Node::TokenRef { name, replaced_lit } => (match replaced_lit {
                Some(lit) => lit,
                None => name,
            }).to_string(),

            Node::TokenLit { literal } => format!("\"{}\"", literal),
        }
    }
}

pub enum Rule {
    // parser_rule
    ParserRule { name: String, node: Box<Node> },
    // token_rule
    TokenRule { name: String, literal: Box<Node> },
}

impl Comment for Rule {
    fn comment(&self) -> String {
        match self {
            Rule::ParserRule { name, node } => format!("{}: {}", name, node.comment()),
            Rule::TokenRule { name, literal } => format!("{}: {}", name, literal.comment()),
        }
    }
}

// top_level
pub struct Grammar {
    parser_rules: Vec<Box<Node>>,
    token_rules: Vec<Box<Node>>,
}
