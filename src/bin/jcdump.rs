use std::io;

use libjcdump::parse_raw;
use libjcdump::wrap;

pub fn main() -> anyhow::Result<()> {
    let mut stdin = io::stdin().lock();
    let mut stdout = io::stdout().lock();

    let raw = parse_raw(&mut stdin)?;
    let data = wrap(&raw)?;
    serde_json::to_writer(&mut stdout, &data)?;

    Ok(())
}
