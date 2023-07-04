mod giganto;
mod postgres;

use clap::Parser;
use std::process::exit;

#[derive(Parser)]
struct Args {
    #[arg(
        short,
        long,
        default_value = "postgresql://postgres:postgres@localhost/postgres"
    )]
    postgres: String,
    #[arg(short, long, default_value = "https://localhost:8444/graphql")]
    giganto: String,
    #[arg(short, long, default_value = "cert.pem")]
    ca_cert: String,
    #[arg(short, long, default_value = "output.log")]
    output: String,
}

fn main() {
    let args = Args::parse();
    let postgres = &args.postgres;
    let giganto = &args.giganto;
    let result = { postgres::run(&postgres) };
    match result {
        Err(e) => {
            eprintln!("Error: {e}");
            exit(1);
        }
        Ok(clusters) => {
            if let Err(e) =
                giganto::get_events(giganto.to_string(), &args.ca_cert, clusters, args.output)
            {
                eprintln!("fail to create client. {e}");
            }
        }
    }
}
