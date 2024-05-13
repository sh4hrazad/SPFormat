#![cfg(not(target_arch = "wasm32"))]

use std::env::args;
use std::fs;

use sp_format::format_string;
use sp_format::settings::Settings;

pub fn build_settings_from_args() -> anyhow::Result<Settings> {
    let str = fs::read_to_string("sp_format.toml")?;

    let toml = toml::from_str::<Settings>(&str)?;

    Ok(toml)
}

fn main() -> anyhow::Result<()> {
    let args = args().skip(1).collect::<Vec<String>>();

    if args.is_empty() {
        anyhow::bail!("no files given to reformat!");
    }

    let settings = build_settings_from_args()?;

    for file_name in &args {
        let source = match fs::read_to_string(&file_name) {
            Ok(src) => src,
            Err(_) => anyhow::bail!("failed to read sourcepawn file: {}", file_name),
        };

        let output = format_string(&source, &settings)?;

        if !output.is_empty() {
            fs::write(&file_name, output)?;
        } else {
            println!(
                "writer got rekt! potential syntax error in file: {}",
                file_name
            );
        }
    }

    println!("Press any key to exit...");
    std::io::stdin().read_line(&mut String::new())?;

    anyhow::Ok(())
}
