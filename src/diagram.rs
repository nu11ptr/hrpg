use crate::ast::{Grammar, Node, ParserRule, TokenRule};
use railroad::{Diagram, RailroadNode};

pub fn draw_diagram(grammar: &Grammar) -> Diagram<Box<dyn RailroadNode>> {
    let token_rules = &grammar.token_rules;
    let parser_rules = &grammar.parser_rules;

    let mut nodes: Vec<Box<dyn RailroadNode>> = parser_rules.iter().map(make_parser_rule).collect();
    let token_nodes: Vec<Box<dyn RailroadNode>> = token_rules.iter().map(make_token_rule).collect();
    nodes.extend(token_nodes);

    let root = Box::new(railroad::VerticalGrid::new(nodes));
    Diagram::with_default_css(root)
}

#[inline]
fn make_parser_rule(rule: &ParserRule) -> Box<dyn RailroadNode> {
    make_rule(&rule.name, &rule.node)
}

#[inline]
fn make_token_rule(rule: &TokenRule) -> Box<dyn RailroadNode> {
    make_rule(&rule.name, &rule.literal)
}

fn make_rule(name: &str, node: &Node) -> Box<dyn RailroadNode> {
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
        Binding { name: _, node } => make_node(node),
        Alternatives { nodes } => Box::new(Choice::new(nodes.iter().map(make_node).collect())),
        MultipartBody { nodes } => Box::new(Sequence::new(nodes.iter().map(make_node).collect())),
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
    }
}
