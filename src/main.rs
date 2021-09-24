use clap::{App, Arg};

mod ast;

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
    println!("Config: {}", config.unwrap_or("<N/A>"))
}
