use std::fs;

use clap::{App, Arg};

use hrpg_core::{ast, process};

fn main() {
    let app = App::new("Human Readable Parser Generator")
        .version("v0.1.0")
        .arg(Arg::with_name("config")
            .short("c")
            .long("config")
            .takes_value(true)
            .help("Configuration file specifying overrides to the default configuration"))
        .arg(Arg::with_name("INPUT")
            .required(true)
            .index(1)
            .help("The grammar file to parse"));

    let matches = app.get_matches();
    let config = matches.value_of("config");
    let grammar = matches.value_of("INPUT");

    println!("Grammar: {}", grammar.unwrap());
    println!("Config: {}", config.unwrap_or("<N/A>"));

    let data = fs::read_to_string(grammar.unwrap()).expect("File not found");
    let g = ast::parse_hrpg(&data).unwrap();
    println!("Original AST: {:?}\n", g);

    let mut proc = process::Process::new();
    let g2 = proc.process(&g);
    println!("Transformed AST: {:?}\n", g2);

    println!("Tokens: {:?}", proc.token_names);
    println!("Errors: {:?}", proc.errors);
}
