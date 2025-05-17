use std::fs::File;
use std::io::{BufRead, BufReader, Write};

pub fn write_env(key: &str, value: &str, env_path: &str) -> std::io::Result<()> {
    let mut lines = Vec::new();
    if let Ok(file) = std::fs::File::open(env_path) {
        for line in BufReader::new(file).lines() {
            let l = line?;
            if !l.trim_start().starts_with(&format!("{}=", key)) {
                lines.push(l);
            }
        }
    }
    lines.push(format!("{} = {}", key, value));
    let mut file = File::create(env_path)?;
    for l in lines {
        writeln!(file, "{}", l)?;
    }
    Ok(())
}
