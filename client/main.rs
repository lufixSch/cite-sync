use clap::{Command, command};

fn main() {
    let cmd = command!()
        .subcommand_required(true)
        .arg_required_else_help(true)
        .subcommand(Command::new("sync").about("Sync local Bibliography with server"));

    let matches = cmd.get_matches();

    match matches.subcommand() {
        Some(("sync", _sub_matches)) => {
            println!("Run Sync");
        }
        _ => unreachable!(),
    }
}
