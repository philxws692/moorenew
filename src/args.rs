use clap::{
    Args,
    Parser,
    Subcommand
};


#[derive(Debug, Parser)]
#[clap(author, version, about)]
pub struct MooRenewArgs {
    #[clap(subcommand)]
    pub action: Option<Action>,
}

#[derive(Debug, Subcommand)]
pub enum Action {
    /// Generate an SSH Keypair which MooRenew uses to fetch the certificates
    Keygen(GenerateKeyCommand),
    /// Runs the updating process
    Run(RunCommand),
}

#[derive(Debug, Args)]
pub struct GenerateKeyCommand {
    /// RSA-4096, or ED25519
    pub algorithm: Option<String>,
    /// The comment to add to the ssh key generation
    pub comment: Option<String>,
    /// The name of the file to store the key in
    pub filename: Option<String>,
}

#[derive(Debug, Args)]
pub struct RunCommand {
    /// Checks if the cert has changed since the last time
    #[arg(short, long, default_value_t = false)]
    pub dry_run: bool,
}