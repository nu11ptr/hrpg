trait Comment {
    fn comment(&self) -> String;
}

pub enum Node {
    Binding { name: String, node: Box<Node> },

    // rule_body
    Alternatives { nodes: Vec<Node> },
    // rule_piece
    MultipartBody { nodes: Vec<Node> },
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
                    match node {
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

// parser_rule
struct ParserRule {
    name: String,
    node: Node,
}

impl Comment for ParserRule {
    fn comment(&self) -> String {
        format!("{}: {}", self.name, self.node.comment())
    }
}

// token_rule
struct TokenRule {
    name: String,
    literal: Node,
}

impl Comment for TokenRule {
    fn comment(&self) -> String {
        format!("{}: {}", self.name, self.literal.comment())
    }
}

// top_level
pub struct Grammar {
    parser_rules: Vec<ParserRule>,
    token_rules: Vec<TokenRule>,
}
