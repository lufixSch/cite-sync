use clap::{Command, command};

fn main() {
    // Define the command line interface using Clap.
    let cmd = command!()
        .subcommand_required(true)  // Require at least one subcommand to be present.
        .arg_required_else_help(true)  // Show help if no arguments are provided.
        .subcommand(Command::new("sync").about("Sync local Bibliography with server"));  // Define the 'sync' subcommand.

    // Parse the command line arguments.
    let matches = cmd.get_matches();

    // Match on the parsed subcommands and perform actions accordingly.
    match matches.subcommand() {
        Some(("sync", _sub_matches)) => {
            println!("Run Sync");  // Action to take when 'sync' subcommand is used.
        }
        _ => unreachable!(),  // This should never be reached because of subcommand_required(true).
    }
}
