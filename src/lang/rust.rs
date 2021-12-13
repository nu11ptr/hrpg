use crate::parser_gen::LangConfig;
use convert_case::Case;
use std::collections::HashSet;

pub struct RustConfig;

impl LangConfig for RustConfig {
    fn var_case(&self) -> Case {
        convert_case::Case::Snake
    }

    fn class_case(&self) -> Case {
        convert_case::Case::UpperCamel
    }

    fn function_case(&self) -> Case {
        convert_case::Case::Snake
    }

    fn keywords(&self) -> HashSet<String> {
        todo!()
    }
}
