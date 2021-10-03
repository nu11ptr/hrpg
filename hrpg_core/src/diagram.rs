use crate::ast::{Grammar, Node};
use railroad::{Diagram, RailroadNode};

pub fn draw_diagram(grammar: &Grammar) -> Diagram<Box<dyn RailroadNode>> {
    let token_rules = &grammar.token_rules;
    let parser_rules = &grammar.parser_rules;

    let mut nodes: Vec<Box<dyn RailroadNode>> = parser_rules.into_iter().map(make_node).collect();
    let token_nodes: Vec<Box<dyn RailroadNode>> = token_rules.into_iter().map(make_node).collect();
    nodes.extend(token_nodes);

    let root = Box::new(railroad::VerticalGrid::new(nodes));
    Diagram::with_default_css(root)
}

fn make_rule(name: &String, node: &Box<Node>) -> Box<dyn RailroadNode> {
    let comment = Box::new(railroad::Comment::new(name.into()));
    let start = Box::new(railroad::Start);
    let end = Box::new(railroad::End);
    Box::new(railroad::Sequence::new(vec![
        start,
        comment,
        make_node(node),
        end,
    ]))
}

fn make_node(node: &Node) -> Box<dyn RailroadNode> {
    use railroad::*;
    use Node::*;

    match node {
        Binding { name, node } => make_node(node),
        Alternatives { nodes } => Box::new(Choice::new(nodes.into_iter().map(make_node).collect())),
        MultipartBody { nodes } => {
            Box::new(Sequence::new(nodes.into_iter().map(make_node).collect()))
        }
        ZeroOrMore { node } => {
            let repeat = Box::new(Repeat::new(make_node(node), Empty));
            Box::new(Optional::new(repeat))
        }
        OneOrMore { node } => Box::new(Repeat::new(make_node(node), Empty)),
        ZeroOrOne { node, .. } => Box::new(Optional::new(make_node(node))),
        RuleRef { name } => Box::new(NonTerminal::new(name.into())),
        TokenRef { name, replaced_lit } => match replaced_lit {
            Some(literal) => Box::new(Terminal::new(literal.into())),
            None => Box::new(NonTerminal::new(name.into())),
        },
        TokenLit { literal } => Box::new(Terminal::new(literal.into())),
        ParserRule { name, node } => make_rule(name, node),
        TokenRule { name, literal } => make_rule(name, literal),
    }
}
