use std::error::Error;
use std::fs;
use std::path::PathBuf;
use std::str::FromStr;

use clap::Parser;

use hrpg::ast::parse_hrpg;
use hrpg::diagram::draw_diagram;
use hrpg::transform::Transform;

/// Human Readable Parser Generator
#[derive(clap::Parser)]
#[clap(author = "Scott Meeuwsen <smeeuwsen@gmail.com>", version = "v0.1.0")]
struct Args {
    #[clap(subcommand)]
    sub_cmd: SubCommands,
}

#[derive(clap::Parser)]
enum SubCommands {
    Build(Build),
    Draw(Draw),
}

/// Build lexer/parser from a grammar
#[derive(clap::Parser)]
struct Build {
    /// Configuration file specifying overrides to the default configuration
    #[clap(short, long)]
    config_file: Option<PathBuf>,

    /// The grammar file to build
    input_file: PathBuf,
}

/// Draw railroad/syntax diagram from a grammar
#[derive(clap::Parser)]
struct Draw {
    /// The grammar file to draw
    input_file: PathBuf,
}

fn main() {
    let options = Args::parse();

    let result = match options.sub_cmd {
        SubCommands::Build(build) => process_build(&build),
        SubCommands::Draw(draw) => process_draw(&draw),
    };

    match result {
        Ok(Some(str)) => println!("{}", str),
        Ok(None) => {}
        Err(err) => eprintln!("An error occurred: {}", err),
    }
}

fn process_build(build: &Build) -> Result<Option<String>, Box<dyn Error>> {
    println!("Grammar: {:?}", &build.input_file);
    println!(
        "Config: {:?}\n",
        build
            .config_file
            .as_ref()
            .unwrap_or(&PathBuf::from_str("N/A").unwrap())
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
    eprintln!("Grammar: {:?}", &draw.input_file);
    let data = fs::read_to_string(&draw.input_file)?;
    let g = parse_hrpg(&data)?;
    let (g2, transform) = Transform::process(&g);

    if transform.errors.is_empty() {
        Ok(Some(format!("{}", draw_diagram(&g2))))
    } else {
        return Err(format!("{:?}", &transform.errors).into());
    }
}
