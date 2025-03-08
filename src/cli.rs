use crate::config::Config;
use crate::{services, store};
use clap::Parser;
use std::io::Write;
use surrealdb::engine::any::Any;
use surrealdb::Surreal;

#[derive(Parser, Debug)]
#[command(version)]
pub struct Args {
    #[arg(short, long, default_value_t = String::from("./config/app.toml"))]
    pub config_file: String,
    pub command: Option<String>,
    pub subcommand: Option<String>,
    pub args: Vec<String>,
}

pub fn parse() -> Args {
    Args::parse()
}

pub async fn run_command(args: &Args, config: &Config, db: &Surreal<Any>) {
    match &args.command {
        Some(s) if s == "users" => run_users_command(&args, config, db).await,
        Some(s) => eprintln!("Invalid command: {}", s),
        None => (),
    }
}

pub async fn run_users_command(args: &Args, config: &Config, db: &Surreal<Any>) {
    match &args.subcommand {
        Some(s) if s == "list" => list_users(db).await,
        Some(s) if s == "create" => create_user(args, config, db).await,
        Some(s) => eprintln!("Invalid `users` subcommand: {}", s),
        None => (),
    }
}

pub async fn list_users(db: &Surreal<Any>) -> () {
    let users = store::auth::users_stream(db).await.unwrap();
    let mut has_users = false;

    users.iter().for_each(|user| {
        if !has_users {
            println!("{:20}  {:40}", "Name", "Email");
            println!("{}  {}", "-".repeat(20), "-".repeat(40));
        }
        println!("{:20}  {:40}", user.name, user.email);
        has_users = true;
    });

    if !has_users {
        println!("No users found.");
    }

    ()
}

pub async fn create_user(args: &Args, config: &Config, db: &Surreal<Any>) {
    if args.args.len() != 2 {
        eprintln!("`users create` requires 2 arguments: username and email");
        std::process::exit(1);
    }

    let username = args.args.get(0).unwrap().trim().to_lowercase();
    let email = args.args.get(1).unwrap().trim().to_lowercase();

    if let Err(msg) = services::auth::validate_username(&username, config) {
        eprintln!("User creation failed: {}", msg);
        std::process::exit(2);
    }

    let mut password0 = String::new();
    let mut password1 = String::new();

    // TODO: Turn off echo.
    print!("Enter password: ");
    std::io::stdout().flush().unwrap();
    std::io::stdin().read_line(&mut password0).expect("Failed to read input.");

    print!("Enter again   : ");
    std::io::stdout().flush().unwrap();
    std::io::stdin().read_line(&mut password1).expect("Failed to read input.");

    password0 = password0.trim().to_string();
    password1 = password1.trim().to_string();
    if password0 != password1 {
        println!("Passwords do not match.");
        return;
    }

    match store::auth::create_user(
        &username,
        &email,
        &password0,
        &db
    ).await {
        Ok(_) => println!("User created."),
        Err(e) => eprintln!("Error creating user: {}", e),
    }
}
