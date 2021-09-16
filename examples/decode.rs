use structopt::StructOpt;

use dq1_password::*;

#[derive(Debug, StructOpt)]
struct Opt {
    password: String,
}

fn main() -> anyhow::Result<()> {
    let opt = Opt::from_args();

    let state = decode(&opt.password)?;
    let json = serde_json::to_string_pretty(&state)?;

    println!("{}", json);

    Ok(())
}
