use std::path::PathBuf;

use structopt::StructOpt;

use dq1_password::*;

#[derive(Debug, StructOpt)]
struct Opt {
    #[structopt(parse(from_os_str))]
    path_json: PathBuf,
}

fn main() -> anyhow::Result<()> {
    let opt = Opt::from_args();

    let json = std::fs::read_to_string(opt.path_json)?;
    let state: GameState = serde_json::from_str(&json)?;

    let password = encode(&state)?;

    println!("{}", password);

    Ok(())
}
