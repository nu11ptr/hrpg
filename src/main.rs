use std::error::Error;
use std::fs;

use clap::{AppSettings, Clap};

use hrpg_core::ast::parse_hrpg;
use hrpg_core::diagram::draw_diagram;
use hrpg_core::transform::Transform;

/// Human Readable Parser Generator
#[derive(Clap)]
#[clap(version = "v0.1.0")]
#[clap(setting = AppSettings::ColoredHelp)]
struct Options {
    #[clap(subcommand)]
    sub_cmd: SubCommands,
}

#[derive(Clap)]
enum SubCommands {
    Build(Build),
    Draw(Draw),
}

/// Build lexer/parser from a grammar
#[derive(Clap)]
struct Build {
    /// Configuration file specifying overrides to the default configuration
    #[clap(short, long)]
    config_file: Option<String>,

    /// The grammar file to build
    input_file: String,
}

/// Draw railroad/syntax diagram from a grammar
#[derive(Clap)]
struct Draw {
    /// The grammar file to draw
    input_file: String,
}

fn main() {
    let options = Options::parse();

    let result = match options.sub_cmd {
        SubCommands::Build(build) => process_build(&build),
        SubCommands::Draw(draw) => process_draw(&draw),
    };
    match result {
        Ok(Some(str)) => println!("{}", str),
        Ok(None) => (),
        Err(err) => println!("An error occurred: {}", err),
    }
}

fn process_build(build: &Build) -> Result<Option<String>, Box<dyn Error>> {
    println!("Grammar: {}", &build.input_file);
    println!(
        "Config: {}\n",
        build.config_file.as_ref().unwrap_or(&"<N/A>".to_owned())
    );

    let data = fs::read_to_string(&build.input_file)?;
    let g = parse_hrpg(&data)?;

    println!("Original AST: {:?}\n", g);

    let (g2, transform) = Transform::process(&g);
    println!("Transformed AST: {:?}\n", g2);

    println!("Tokens: {:?}", &transform.token_names);
    println!("Errors: {:?}", &transform.errors);

    Ok(None)
}

fn process_draw(draw: &Draw) -> Result<Option<String>, Box<dyn Error>> {
    eprintln!("Grammar: {}", &draw.input_file);
    let data = fs::read_to_string(&draw.input_file)?;
    let g = parse_hrpg(&data)?;
    let (g2, transform) = Transform::process(&g);

    if *&transform.errors.is_empty() {
        Ok(Some(format!("{}", draw_diagram(&g2))))
    } else {
        Err(format!("{:?}", &transform.errors))?
    }
}
