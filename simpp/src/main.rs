use clap::Parser;
use cpparser::{display::CustomDisplay, EntityParser};
use std::io::Read;

/// Simplify a C++ entity name into a human-readable form
///
/// The entity can be provided from either directly on the command line (with
/// suitable quoting and escaping) or by piping a file into standard input.
///
#[derive(Parser, Debug)]
#[clap(author, version, about)]
struct Args {
    /// Number of terminal columns available to display the entity
    #[clap(short = 'c', long = "cols")]
    max_cols: u16,

    /// C++ entity name to be simplified (can also be specified via stdin)
    entity: Option<String>,
}

fn main() {
    // Parse CLI arguments
    let mut args = Args::parse();

    // Get C++ entity name from either CLI or stdin
    let mut stdin = String::new();
    let mut entity = args
        .entity
        .take()
        .or_else(|| {
            if atty::isnt(atty::Stream::Stdin) {
                std::io::stdin()
                    .read_to_string(&mut stdin)
                    .expect("Failed to read entity from standard input");
                Some(stdin)
            } else {
                None
            }
        })
        .expect("Please provide a C++ entity name via either CLI or stdin");

    // Pre-process entity name to get rid of line feeds from pretty-printing or
    // other kinds of preprocessing.
    let entity_lines = entity.lines().collect::<Vec<_>>();
    entity = entity_lines.join(" ");

    // Parse C++ entity name
    let mut entities = EntityParser::new();
    let entity = entities
        .parse_entity(&entity)
        .expect("Failed to parse entity name");
    let entity = entities.entity(entity);

    // Display it
    println!("{}", entity.bounded_display(args.max_cols));
}
