use std::fs;
use std::path::Path;
use std::process::Command;
use std::{ffi::OsStr, io};

use tempfile::{TempDir, tempdir};

fn javac<I: IntoIterator<Item = P>, P: AsRef<Path> + AsRef<OsStr>>(
    srcdir: P,
    files: I,
) -> anyhow::Result<TempDir> {
    let output = tempdir()?;

    Command::new("javac")
        .arg("--source-path")
        .arg(srcdir)
        .arg("-d")
        .arg(output.path())
        .args(files)
        .status()?;

    Ok(output)
}

#[test]
fn simple() -> anyhow::Result<()> {
    let srcdir = Path::new(file!()).parent().unwrap().join("./data/");
    let output = javac(srcdir.clone(), [srcdir.join("Main.java")])?;

    let mut main = fs::File::open(output.path().join("./com/example/Main.class"))?;
    let raw = jcdump::parse_raw(&mut main)?;
    let data = jcdump::wrap(&raw)?;
    serde_json::to_writer_pretty(io::stdout(), &data)?;
    println!();

    let mut module = fs::File::open(output.path().join("./module-info.class"))?;
    let raw = jcdump::parse_raw(&mut module)?;
    let data = jcdump::wrap(&raw)?;
    serde_json::to_writer_pretty(io::stdout(), &data)?;
    println!();

    Ok(())
}
