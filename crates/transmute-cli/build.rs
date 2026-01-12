use clap::CommandFactory;
use clap_complete::{generate_to, shells::*};
use std::env;
use std::io::Error;

// Import the CLI definition from the library
include!("src/cli.rs");

fn main() -> Result<(), Error> {
    let outdir = match env::var_os("OUT_DIR") {
        None => return Ok(()),
        Some(outdir) => outdir,
    };

    let mut cmd = Cli::command();

    // Generate Unix shells only on Unix platforms
    #[cfg(unix)]
    {
        generate_to(Bash, &mut cmd, "transmute", &outdir)?;
        generate_to(Fish, &mut cmd, "transmute", &outdir)?;
        generate_to(Zsh, &mut cmd, "transmute", &outdir)?;
    }

    // PowerShell works on all platforms
    generate_to(PowerShell, &mut cmd, "transmute", &outdir)?;

    println!("cargo:warning=Shell completions generated in {:?}", outdir);

    Ok(())
}
