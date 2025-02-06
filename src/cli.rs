use clap::Parser;

#[derive(Parser, Debug)]
#[command(version)]
pub struct Args {
    #[arg(short, long, default_value_t = String::from("./config/app.toml"))]
    pub config_file: String,
}

pub fn parse() -> Args {
    Args::parse()
}
