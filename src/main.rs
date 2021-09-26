use std::fs;

use clap::{App, Arg};

use hrpg_core::{ast, process};
use std::error::Error;

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
    let config_file = matches.value_of("config");
    let input_file = matches.value_of("INPUT");

    match process_input(input_file.unwrap(), config_file) {
        Ok(_) => (),
        Err(err) => println!("An error occurred: {}", err),
    }
}

fn process_input(input_file: &str, config_file: Option<&str>) -> Result<(), Box<dyn Error>> {
    println!("Grammar: {}", input_file);
    println!("Config: {}\n", config_file.unwrap_or("<N/A>"));

    let data = fs::read_to_string(input_file)?;
    let g = ast::parse_hrpg(&data)?;

    println!("Original AST: {:?}\n", g);

    let mut proc = process::Process::new();
    let g2 = proc.process(&g);
    println!("Transformed AST: {:?}\n", g2);

    println!("Tokens: {:?}", proc.token_names);
    println!("Errors: {:?}", proc.errors);

    Ok(())
}
