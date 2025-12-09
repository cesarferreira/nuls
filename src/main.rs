use clap::Parser;

/// rls: a tiny example CLI to show the project is wired up.
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Who to greet
    #[arg(short, long, default_value = "world")]
    name: String,

    /// How many times to repeat the greeting
    #[arg(short, long, default_value_t = 1, value_parser = clap::value_parser!(u8).range(1..=10))]
    times: u8,
}

fn main() {
    let cli = Cli::parse();

    for _ in 0..cli.times {
        println!("Hello, {}!", cli.name);
    }
}
