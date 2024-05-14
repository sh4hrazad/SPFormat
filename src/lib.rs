mod formatter;
mod language;
mod parser;
pub mod settings;
mod writers;

use formatter::format_string_language;
use settings::Settings;

#[cfg(not(target_arch = "wasm32"))]
pub fn format_string(input: &String, settings: &Settings) -> anyhow::Result<String> {
    let language = tree_sitter_sourcepawn::language().into();
    let output = format_string_language(&input, language, settings)?;

    Ok(output)
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub async fn sp_format(input: String, val: JsValue) -> Result<String, JsValue> {
    use wasm_bindgen::prelude::*;

    tree_sitter::TreeSitter::init().await?;
    let language = language::sourcepawn().await.unwrap();
    let settings: Settings = val.into_serde().unwrap();
    let output = format_string_language(&input, language, &settings)
        .expect("An error has occured while generating the SourcePawn code.");
    Ok(output)
}
