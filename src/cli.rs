use clap::Parser;
use futures_util::TryStreamExt;
use std::io::Write;
use surrealdb::engine::remote::ws::Client;
use surrealdb::Surreal;
use crate::store;

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

pub async fn run_command(args: &Args, db: &Surreal<Client>) {
    match &args.command {
        Some(s) if s == "users" => run_users_command(&args, db).await,
        Some(s) => eprintln!("Invalid command: {}", s),
        None => (),
    }
}

pub async fn run_users_command(args: &Args, db: &Surreal<Client>) {
    match &args.subcommand {
        Some(s) if s == "list" => list_users(db).await,
        Some(s) if s == "create" => create_user(args, db).await,
        Some(s) => eprintln!("Invalid `users` subcommand: {}", s),
        None => (),
    }
}

pub async fn list_users(db: &Surreal<Client>) -> () {
    let mut stream = store::auth::users_stream(db).await;
    let mut has_users = false;

    while let Ok(Some(user)) = stream.try_next().await {
        if !has_users {
            println!("{:20}  {:40}", "Name", "Email");
            println!("{}  {}", "-".repeat(20), "-".repeat(40));
        }
        println!("{:20}  {:40}", user.name, user.email);
        has_users = true;
    }

    if !has_users {
        println!("No users found.");
    }

    ()
}

pub async fn create_user(args: &Args, db: &Surreal<Client>) {
    if args.args.len() != 2 {
        eprintln!("`users create` requires 2 arguments: username and email");
        return;
    }

    let mut password0 = String::new();
    let mut password1 = String::new();

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
        args.args.get(0).unwrap(),
        args.args.get(1).unwrap(),
        &password0,
        &db
    ).await {
        Ok(_) => println!("User created."),
        Err(e) => eprintln!("Error creating user: {}", e),
    }
}
