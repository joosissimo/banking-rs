use std::fs::File;
use std::path::Path;

use anyhow::Result;
use clap::{Args, Parser, Subcommand};

use banking_rs::account::Account;
use banking_rs::banking_system::BankingSystem;

#[derive(Parser)]
#[command(version, about)]
#[command(propagate_version = true)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Show all accounts
    Show,
    /// Create account
    Create(SingleAccountOpArgs),
    /// Deposit amount to account
    Deposit(SingleAccountOpArgs),
    /// Withdraw amount from account
    Withdraw(SingleAccountOpArgs),
    /// Transfer amount between acounts
    Transfer(TransferOpArgs),
}

#[derive(Args)]
struct SingleAccountOpArgs {
    #[arg(short, long)]
    name: String,
    #[arg(short, long)]
    amount: String,
}

#[derive(Args)]
struct TransferOpArgs {
    #[arg(short, long)]
    from: String,
    #[arg(short, long)]
    to: String,
    #[arg(short, long)]
    amount: String,
}

const PATH: &str = "./banking_system.csv";

fn create_system() -> Result<BankingSystem> {
    let file = if !Path::new(PATH).exists() {
        File::create_new(PATH)?
    } else {
        File::open(PATH)?
    };

    let mut rdr = csv::Reader::from_reader(file);

    Ok(BankingSystem(
        rdr.deserialize::<Account>()
            .collect::<Result<Vec<_>, _>>()?,
    ))
}

fn save_system(bs: BankingSystem) -> Result<()> {
    let mut wtr = csv::Writer::from_writer(File::create(PATH)?);

    for account in bs.0 {
        wtr.serialize(account)?;
    }

    Ok(())
}

fn main() -> Result<()> {
    let mut bs = create_system()?;
    let cli = Cli::parse();

    match &cli.command {
        Commands::Show => bs.show(),
        Commands::Create(SingleAccountOpArgs { name, amount }) => bs.create(name, amount)?,
        Commands::Deposit(SingleAccountOpArgs { name, amount }) => bs.deposit(name, amount)?,
        Commands::Withdraw(SingleAccountOpArgs { name, amount }) => bs.withdraw(name, amount)?,
        Commands::Transfer(TransferOpArgs { from, to, amount }) => bs.transfer(from, to, amount)?,
    }

    save_system(bs)?;

    Ok(())
}
