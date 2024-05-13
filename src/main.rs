use std::env::args;
#[cfg(not(target_arch = "wasm32"))]
use std::fs;

use serde::Deserialize;
use sp_format::format_string;
use sp_format::settings::Settings;

#[cfg(not(target_arch = "wasm32"))]
/// A tool to format SourcePawn code (new AND old syntaxes).
#[derive(Deserialize)]
#[allow(unused)]
pub struct Args {
    /// Number of empty lines to insert before a function declaration.
    breaks_before_function_decl: u32,

    /// Number of empty lines to insert before a function definition.
    breaks_before_function_def: u32,

    /// Number of empty lines to insert before an enum declaration.
    breaks_before_enum: u32,

    /// Number of empty lines to insert before an enum struct declaration.
    breaks_before_enum_struct: u32,

    /// Number of empty lines to insert before a methodmap declaration.
    breaks_before_methodmap: u32,

    /// Whether or not to break before a function declaration brace.
    brace_wrapping_before_function: bool,

    /// Whether or not to break before a loop statement brace.
    brace_wrapping_before_loop: bool,

    /// Whether or not to break before a condition statement brace.
    brace_wrapping_before_condition: bool,

    /// Whether or not to break before an enum struct declaration brace.
    brace_wrapping_before_enum_struct: bool,

    /// Whether or not to break before an enum declaration brace.
    brace_wrapping_before_enum: bool,

    /// Whether or not to break before a typeset declaration brace.
    brace_wrapping_before_typeset: bool,

    /// Whether or not to break before a funcenum declaration brace.
    brace_wrapping_before_funcenum: bool,

    /// Whether or not to break before a methodmap declaration brace.
    brace_wrapping_before_methodmap: bool,

    /// Whether or not to break before a methodmap property declaration brace.
    brace_wrapping_before_methodmap_property: bool,
}

#[cfg(not(target_arch = "wasm32"))]
pub fn build_settings_from_args() -> Result<Settings, anyhow::Error> {
    let str = fs::read_to_string("sp_format.toml")?;

    let toml = toml::from_str::<Settings>(&str)?;

    Ok(toml)
}

#[cfg(not(target_arch = "wasm32"))]
fn main() -> Result<(), anyhow::Error> {
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

        fs::write(&file_name, output)?;
    }

    println!("Press any key to exit...");
    std::io::stdin().read_line(&mut String::new())?;

    anyhow::Ok(())
}
