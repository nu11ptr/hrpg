use crate::ast::{Comment, Grammar, Node};
use std::collections::HashSet;
use std::fmt;

use convert_case::Casing;

#[derive(Debug)]
pub enum MatchKind {
    Once,
    ZeroOrOnce,
    ZeroOrMore,
    OnceOrMore,
}

#[derive(Debug)]
pub enum MatchRule {
    Token {
        kind: MatchKind,
        token_name: String,
        var_name: String,
        comment: String,
    },
    Parser {
        kind: MatchKind,
        func_name: String,
        var_name: String,
        comment: String,
    },
}

#[derive(Debug)]
pub struct Function {
    name: String,
    ret_on_match: bool,
    actions: Vec<MatchRule>,
}

#[derive(Debug)]
pub struct ParserSpec {
    ret_type: String,
    functions: Vec<Function>,
}

pub trait LangConfig {
    fn var_case(&self) -> convert_case::Case;

    fn class_case(&self) -> convert_case::Case;

    fn function_case(&self) -> convert_case::Case;

    fn keywords(&self) -> HashSet<String>;
}

pub trait LangCodeGen<W: fmt::Write> {
    fn file_start(&self, w: &mut W);

    fn class_start(&self, w: &mut W);

    fn class_end(&self, w: &mut W);

    fn func_start(&self, w: &mut W, func: &Function, spec: &ParserSpec);

    fn func_end(&self, w: &mut W, func: &Function, spec: &ParserSpec);

    fn action(&self, w: &mut W, action: &MatchRule, func: &Function) {
        match action {
            MatchRule::Token { kind, .. } => match kind {
                MatchKind::Once => self.match_token_once(w, action, func),
                MatchKind::ZeroOrOnce => self.match_token_zero_or_once(w, action, func),
                MatchKind::ZeroOrMore => self.match_token_zero_or_more(w, action, func),
                MatchKind::OnceOrMore => self.match_token_once_or_more(w, action, func),
            },
            MatchRule::Parser { kind, .. } => match kind {
                MatchKind::Once => self.match_rule_once(w, action, func),
                MatchKind::ZeroOrOnce => self.match_rule_zero_or_once(w, action, func),
                MatchKind::ZeroOrMore => self.match_rule_zero_or_more(w, action, func),
                MatchKind::OnceOrMore => self.match_rule_once_or_more(w, action, func),
            },
        }
    }

    fn match_token_once(&self, w: &mut W, action: &MatchRule, func: &Function);

    fn match_token_zero_or_once(&self, w: &mut W, action: &MatchRule, func: &Function);

    fn match_token_zero_or_more(&self, w: &mut W, action: &MatchRule, func: &Function);

    fn match_token_once_or_more(&self, w: &mut W, action: &MatchRule, func: &Function);

    fn match_rule_once(&self, w: &mut W, action: &MatchRule, func: &Function);

    fn match_rule_zero_or_once(&self, w: &mut W, action: &MatchRule, func: &Function);

    fn match_rule_zero_or_more(&self, w: &mut W, action: &MatchRule, func: &Function);

    fn match_rule_once_or_more(&self, w: &mut W, action: &MatchRule, func: &Function);
}

struct FuncName<'n> {
    base: &'n str,
    sub_name: Option<&'n str>,
    sub_num: Option<u32>,
}

impl<'n> FuncName<'n> {
    pub fn new(base: &'n str) -> Self {
        FuncName {
            base,
            sub_name: None,
            sub_num: None,
        }
    }

    pub fn to_num_sub(&self) -> Self {
        let sub_num = Some(match self.sub_num {
            Some(num) => num + 1,
            None => 1,
        });

        FuncName {
            base: self.base,
            sub_name: self.sub_name,
            sub_num,
        }
    }

    pub fn to_named_sub(&self, sub_name: &'n str) -> Self {
        FuncName {
            base: self.base,
            sub_name: Some(sub_name),
            sub_num: self.sub_num,
        }
    }

    pub fn name(&self, case: convert_case::Case) -> String {
        match (self.sub_name, self.sub_num) {
            (Some(sub_name), Some(sub_num)) => {
                format!("parse_{}_{}_sub{}", self.base, sub_name, sub_num)
            }
            (Some(sub_name), None) => {
                format!("parse_{}_{}", self.base, sub_name)
            }
            (None, Some(sub_num)) => format!("parse_{}_sub{}", self.base, sub_num),
            (None, None) => format!("parse_{}", self.base),
        }
        .to_case(case)
    }

    pub fn base_name(&self) -> &str {
        self.base
    }
}

pub struct ParserGen<L> {
    functions: Vec<Function>,
    lang_config: L,
}

impl<L: LangConfig> ParserGen<L> {
    pub fn new(config: L) -> Self {
        ParserGen {
            functions: vec![],
            lang_config: config,
        }
    }

    // Two step process:
    // 1. gen the language agnostic AST
    //   i)  send to lang gen as we go for xlate of var_names, etc. - also to get return expressions?
    // 2. Send to lang gen to output code
    pub fn generate(mut self, grammar: &Grammar) -> ParserSpec {
        self.functions.reserve(grammar.parser_rules.len());

        for rule in &grammar.parser_rules {
            log::trace!("Starting parser rule: {}", &rule.name);
            self.make_func(&FuncName::new(&rule.name), &rule.node, &rule.comment());
            log::trace!("Ending parser rule: {}", &rule.name);
        }

        ParserSpec {
            // TODO: Need this obviously - from LangConfig?
            ret_type: "<todo>".to_string(),
            functions: self.functions,
        }
    }

    fn make_func(&mut self, func_name: &FuncName, node: &Node, comment: &str) {
        // Make function name and convert to preferred case of lang
        let name = func_name.name(self.lang_config.function_case());

        let ret_on_match = !matches!(node, Node::MultipartBody { .. });

        log::trace!("Starting new function: {}", &name);
        let actions = self.process_node(node, func_name, comment, MatchKind::Once, true);
        log::trace!("Ending function: {}", &name);

        self.functions.push(Function {
            name,
            ret_on_match,
            actions,
        })
    }

    fn make_sub_func(
        &mut self,
        name: &FuncName,
        node: &Node,
        comment: &str,
        kind: MatchKind,
    ) -> Vec<MatchRule> {
        self.make_func(name, node, comment);
        self.process_rule_ref(name, comment, kind)
    }

    fn process_rule_ref(
        &self,
        func_name: &FuncName,
        comment: &str,
        kind: MatchKind,
    ) -> Vec<MatchRule> {
        let var_name = func_name.base_name().to_case(self.lang_config.var_case());

        vec![MatchRule::Parser {
            kind,
            func_name: func_name.name(self.lang_config.function_case()),
            var_name,
            comment: comment.to_string(),
        }]
    }

    fn process_node(
        &mut self,
        node: &Node,
        curr_func: &FuncName,
        comment: &str,
        kind: MatchKind,
        top_level: bool,
    ) -> Vec<MatchRule> {
        match node {
            // Binding - use the name of binding as function name, NOT `curr_func` as base like `Alternatives/MultipartBody`
            Node::Binding { name, node } => {
                self.make_sub_func(&curr_func.to_named_sub(name), node, &node.comment(), kind)
            },
            // If top level of function, we simply process each node and flatten
            // (only this an `MultipartBody` truly return more than one entry)
            Node::Alternatives { nodes } if top_level => nodes
                .iter()
                .flat_map(|node| {
                    self.process_node(node, curr_func, &node.comment(), MatchKind::ZeroOrOnce, false)
                })
                .collect(),
            // If not top level, then we need to force a sub-function to handle it
            Node::Alternatives { .. } => self.make_sub_func(&curr_func.to_num_sub(), node, comment, kind),
            // If top level of function, we simply process each node and flatten
            Node::MultipartBody { nodes } if top_level => nodes
                .iter()
                .flat_map(|node| self.process_node(node, curr_func,&node.comment(), MatchKind::Once, false))
                .collect(),
            // If not top level, then we need to force a sub-function to handle it
            Node::MultipartBody { .. } => self.make_sub_func(&curr_func.to_num_sub(), node, comment, kind),
            Node::ZeroOrMore { node } => {
                self.process_node(node, curr_func,&node.comment(), MatchKind::ZeroOrMore, false)
            }
            Node::OneOrMore { node } => {
                self.process_node(node, curr_func,&node.comment(), MatchKind::OnceOrMore, false)
            }
            Node::ZeroOrOne { node, .. } => {
                self.process_node(node, curr_func,&node.comment(), MatchKind::ZeroOrOnce, false)
            }
            Node::RuleRef { name } =>  self.process_rule_ref(&FuncName::new(name), comment, kind),
            Node::TokenRef { name,.. } => {
                let var_name = name.to_case(self.lang_config.var_case());

                vec![
                    MatchRule::Token {
                        kind,
                        // TODO: needs to conform to generated token naming
                        token_name: name.to_owned(),
                        var_name,
                        comment: comment.to_string(),
                    }
                ]
            }
            Node::TokenLit { literal } => panic!("Found token literal '{}' - this should have been replaced during AST transformation", literal),
        }
    }
}
