use std::{
    fs,
    io::{self, Write},
    path::Path,
};

pub fn set_env(key: &str, value: &str, path: &str) -> io::Result<()> {
    let path = Path::new(path);
    let mut lines = if path.exists() {
        fs::read_to_string(path)?
            .lines()
            .map(|l| l.to_string())
            .collect::<Vec<String>>()
    } else {
        vec![]
    };

    let mut found = false;
    for line in &mut lines {
        if line.starts_with(&format!("{} = ", key)) {
            *line = format!("{} = {}", key, value);
            found = true;
            break;
        }
    }

    if !found {
        lines.push(format!("{} = {}", key, value));
    }

    let mut file = fs::File::create(path)?;
    for line in lines {
        writeln!(file, "{}", line)?;
    }

    Ok(())
}
