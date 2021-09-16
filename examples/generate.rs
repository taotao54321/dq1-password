use structopt::StructOpt;

use dq1_password::*;

#[derive(Debug, StructOpt)]
struct Opt {
    pattern: String,

    #[structopt(default_value = "10")]
    n_max: usize,
}

fn main() -> anyhow::Result<()> {
    let opt = Opt::from_args();

    let passwords = generate(&opt.pattern, opt.n_max)?;
    assert!(passwords.len() <= opt.n_max);
    assert!(passwords.iter().all(|p| decode(p).is_ok()));

    for password in passwords {
        println!("{}", password);
    }

    Ok(())
}
