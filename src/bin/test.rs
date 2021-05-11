use anyhow::{bail, Context, Result};
use std::io::{self, Write};
use std::path::Path;
use std::{
    env,
    process::{Command, Output},
};

fn test_bash(src_file: &str) -> Result<Output> {
    let output = Command::new("bash")
        .arg(Path::new("scripts").join("tests").join("test.bash"))
        .arg(src_file)
        .output()?;
    Ok(output)
}

fn test_fish(src_file: &str) -> Result<Output> {
    let output = Command::new("echo")
        .arg(Path::new("scripts").join("tests").join("test.fish"))
        .arg(src_file)
        .output()?;
    Ok(output)
}

fn test_zsh(src_file: &str) -> Result<Output> {
    let output = Command::new("echo")
        .arg(Path::new("scripts").join("tests").join("test.zsh"))
        .arg(src_file)
        .output()?;
    Ok(output)
}

fn test_tcsh(src_file: &str) -> Result<Output> {
    let output = Command::new("echo")
        .arg(Path::new("scripts").join("tests").join("test.tcsh"))
        .arg(src_file)
        .output()?;
    Ok(output)
}

fn install() -> Result<Output> {
    let output = Command::new("sh")
        .arg(Path::new("scripts").join("tests").join("install.sh"))
        .output()?;
    Ok(output)
}

fn main() -> Result<()> {
    let output = install()?;
    match output.status.code() {
        Some(code) if code != 0 => bail!("failed to install the environment for tests"),
        Some(_) => (),
        None => bail!("install process is terminated by signal"),
    }

    let args: Vec<String> = env::args().collect();
    let src_file = args.last().context("expect the source file")?;
    if !Path::new(src_file).exists() {
        bail!("source file doesn't exist");
    }

    let tests = [test_bash, test_fish, test_zsh, test_tcsh];

    for test in &tests {
        let output = test(src_file)?;
        io::stdout().write_all(&output.stdout)?;
        io::stdout().write_all(&output.stderr)?;
        if output.status.success() {
            println!("Test passes.");
        } else {
            bail!("Test exit status: {}.", output.status);
        }
    }
    println!("All tests pass.");

    Ok(())
}
