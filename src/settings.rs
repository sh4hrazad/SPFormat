use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Settings {
    pub use_space: bool,
    pub space_size: u32,
    pub r#break: BreakSettings,
    pub brace_wrapping: BraceWrappingSettings,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct BreakSettings {
    pub after_function_decl: u32,
    pub before_function_def: u32,
    pub before_enum: u32,
    pub before_enum_struct: u32,
    pub before_methodmap: u32,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct BraceWrappingSettings {
    pub before_function: bool,
    pub before_loop: bool,
    pub before_condition: bool,
    pub before_enum_struct: bool,
    pub before_enum: bool,
    pub before_typeset: bool,
    pub before_funcenum: bool,
    pub before_methodmap: bool,
    pub before_methodmap_property: bool,
    pub before_struct_ctor: bool,
}
