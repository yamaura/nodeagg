use clap::Parser;
use nodeagg::Nodeagg;

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[clap(version, about, long_about = None)]
struct Args {
    /// Expand Command
    #[clap(short, long)]
    expand: bool,

    #[clap(short = 'S', long, default_value = " ")]
    separator: String,

    #[clap(name = "Operations or Node names")]
    ops: Nodeagg,
}

fn main() {
    let args = Args::parse();
    if args.expand {
        println!(
            "{}",
            args.ops.iter().collect::<Vec<_>>().join(&args.separator)
        );
    } else {
        eprintln!("Please specify Command");
        std::process::exit(22);
    }
}
