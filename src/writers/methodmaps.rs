use std::borrow::Borrow;

use tree_sitter::Node;

use super::{
    functions::write_argument_declarations, prev_sibling_kind, statements::write_block,
    variables::write_type, write_comment, write_node, Writer,
};

pub fn write_methodmap(node: &Node, writer: &mut Writer) -> anyhow::Result<()> {
    let nb_lines: usize = usize::try_from(writer.settings.r#break.before_methodmap).unwrap();
    let prev_kind = prev_sibling_kind(&node);

    if !prev_kind.starts_with("preproc_")
        && prev_kind != ""
        && prev_kind != "comment"
        && prev_kind != "alias_declaration"
    {
        // Insert new lines automatically
        writer.write_str("\n".repeat(nb_lines).as_str());
    }

    let mut cursor = node.walk();

    for child in node.children(&mut cursor) {
        let kind = child.kind();
        match kind.borrow() {
            "methodmap" => writer.write_str("methodmap "),
            "identifier" => write_node(&child, writer)?,
            "<" => writer.write_str(" < "),
            "__nullable__" => writer.write_str(" __nullable__ "),
            "{" => {
                if writer.settings.brace_wrapping.before_methodmap {
                    writer.write_ln();
                } else {
                    writer.write(' ');
                }
                writer.write_str("{\n");
                writer.indent += 1;
            }
            "}" => {
                writer.write_str("}");
                writer.indent -= 1;
            }
            "methodmap_alias" => write_methodmap_alias(&child, writer)?,
            "methodmap_native" | "methodmap_native_destructor" | "methodmap_native_constructor" => {
                write_methodmap_native(&child, writer)?
            }
            "methodmap_method" | "methodmap_method_destructor" | "methodmap_method_constructor" => {
                write_methodmap_method(&child, writer)?
            }
            "methodmap_property" => write_methodmap_property(&child, writer)?,
            "comment" => write_comment(&child, writer)?,
            ";" => continue,
            _ => {
                println!("Unexpected kind {} in write_methodmap.", kind);
            }
        }
    }
    writer.write(';');
    writer.write_ln();

    Ok(())
}

fn write_methodmap_alias(node: &Node, writer: &mut Writer) -> anyhow::Result<()> {
    let nb_lines: usize = usize::try_from(writer.settings.r#break.before_function_def).unwrap();
    let prev_kind = prev_sibling_kind(&node);

    if !prev_kind.starts_with("preproc_") && prev_kind != "" && prev_kind != "comment" {
        // Insert two new lines automatically
        writer.write_str("\n".repeat(nb_lines).as_str());
    }

    let mut cursor = node.walk();

    writer.write_indent();

    for child in node.children(&mut cursor) {
        let kind = child.kind();
        match kind.borrow() {
            "public" => writer.write_str("public "),
            "~" | "(" | ")" | "identifier" => write_node(&child, writer)?,
            "=" => writer.write_str(" = "),
            ";" => continue,
            _ => println!("Unexpected kind {} in write_alias_declaration.", kind),
        }
    }
    writer.write(';');
    writer.write_ln();

    Ok(())
}

fn write_methodmap_native(node: &Node, writer: &mut Writer) -> anyhow::Result<()> {
    let nb_lines: usize = usize::try_from(writer.settings.r#break.before_function_def).unwrap();
    let prev_kind = prev_sibling_kind(&node);

    if !prev_kind.starts_with("preproc_") && prev_kind != "" && prev_kind != "comment" {
        // Insert two new lines automatically
        writer.write_str("\n".repeat(nb_lines).as_str());
    }

    let mut cursor = node.walk();

    writer.write_indent();

    for child in node.children(&mut cursor) {
        let kind = child.kind();
        match kind.borrow() {
            "public" => writer.write_str("public "),
            "static" | "native" => {
                write_node(&child, writer)?;
                writer.write(' ');
            }
            "type" => write_type(&child, writer)?,
            "(" | ")" | "symbol" | "~" => write_node(&child, writer)?,
            "=" => writer.write_str(" = "),
            "parameter_declarations" => write_argument_declarations(&child, writer)?,
            ";" => continue,
            _ => println!("Unexpected kind {} in write_methodmap_native.", kind),
        }
    }
    writer.write(';');
    writer.write_ln();

    Ok(())
}

fn write_methodmap_method(node: &Node, writer: &mut Writer) -> anyhow::Result<()> {
    let nb_lines: usize = usize::try_from(writer.settings.r#break.before_function_def).unwrap();
    let prev_kind = prev_sibling_kind(&node);

    if !prev_kind.starts_with("preproc_") && prev_kind != "" && prev_kind != "comment" {
        // Insert two new lines automatically
        writer.write_str("\n".repeat(nb_lines).as_str());
    }

    let mut cursor = node.walk();

    writer.write_indent();

    for child in node.children(&mut cursor) {
        let kind = child.kind();
        match kind.borrow() {
            "public" => writer.write_str("public "),
            "static" => {
                write_node(&child, writer)?;
                writer.write(' ');
            }
            "type" => write_type(&child, writer)?,
            "(" | ")" | "symbol" | "~" => write_node(&child, writer)?,
            "=" => writer.write_str(" = "),
            "parameter_declarations" => write_argument_declarations(&child, writer)?,
            "block" => {
                if writer.settings.brace_wrapping.before_function {
                    writer.write_ln();
                    write_block(&child, writer, true)?;
                } else {
                    writer.write(' ');
                    write_block(&child, writer, false)?;
                }
            }
            _ => println!("Unexpected kind {} in write_methodmap_method.", kind),
        }
    }
    writer.write_ln();

    Ok(())
}

fn write_methodmap_property(node: &Node, writer: &mut Writer) -> anyhow::Result<()> {
    let nb_lines: usize = usize::try_from(writer.settings.r#break.before_function_def).unwrap();
    let prev_kind = prev_sibling_kind(&node);

    if !prev_kind.starts_with("preproc_") && prev_kind != "" && prev_kind != "comment" {
        // Insert two new lines automatically
        writer.write_str("\n".repeat(nb_lines).as_str());
    }

    let mut cursor = node.walk();

    writer.write_indent();

    for child in node.children(&mut cursor) {
        let kind = child.kind();
        match kind.borrow() {
            "property" => {
                write_node(&child, writer)?;
                writer.write(' ');
            }
            "type" => write_type(&child, writer)?,
            "(" | ")" | "symbol" | "~" => write_node(&child, writer)?,
            "{" => {
                if writer.settings.brace_wrapping.before_methodmap_property {
                    writer.write_ln();
                } else {
                    writer.write(' ');
                }
                writer.write_str("{\n");
                writer.indent += 1;
            }
            "}" => {
                writer.write_str("}");
                writer.indent -= 1;
            }
            "=" => writer.write_str(" = "),
            "parameter_declarations" => write_argument_declarations(&child, writer)?,
            "methodmap_property_alias" => write_methodmap_property_alias(&child, writer)?,
            "methodmap_property_method" | "methodmap_property_native" => {
                write_methodmap_property_method(&child, writer)?
            }
            ";" => continue,
            _ => println!("Unexpected kind {} in write_methodmap_property.", kind),
        }
    }
    writer.write(';');
    writer.write_ln();

    Ok(())
}

fn write_methodmap_property_alias(node: &Node, writer: &mut Writer) -> anyhow::Result<()> {
    let nb_lines: usize = usize::try_from(writer.settings.r#break.before_function_def).unwrap();
    let prev_kind = prev_sibling_kind(&node);

    if !prev_kind.starts_with("preproc_") && prev_kind != "" && prev_kind != "comment" {
        // Insert two new lines automatically
        writer.write_str("\n".repeat(nb_lines).as_str());
    }

    let mut cursor = node.walk();

    writer.write_indent();

    for child in node.children(&mut cursor) {
        let kind = child.kind();
        match kind.borrow() {
            "public" => writer.write_str("public "),
            "methodmap_property_getter" => writer.write_str("get()"),
            "identifier" => write_node(&child, writer)?,
            "=" => writer.write_str(" = "),
            ";" => continue,
            _ => println!(
                "Unexpected kind {} in write_methodmap_property_alias.",
                kind
            ),
        }
    }
    writer.write(';');
    writer.write_ln();

    Ok(())
}

fn write_methodmap_property_method(node: &Node, writer: &mut Writer) -> anyhow::Result<()> {
    let nb_lines: usize = usize::try_from(writer.settings.r#break.before_function_def).unwrap();
    let prev_kind = prev_sibling_kind(&node);

    if !prev_kind.starts_with("preproc_") && prev_kind != "" && prev_kind != "comment" {
        // Insert two new lines automatically
        writer.write_str("\n".repeat(nb_lines).as_str());
    }

    let mut cursor = node.walk();

    writer.write_indent();

    for child in node.children(&mut cursor) {
        let kind = child.kind();
        match kind.borrow() {
            "public" => writer.write_str("public "),
            "native" => {
                write_node(&child, writer)?;
                writer.write(' ');
            }
            "methodmap_property_getter" => writer.write_str("get()"),
            "methodmap_property_setter" => write_methodmap_property_setter(&child, writer)?,
            "identifier" => write_node(&child, writer)?,
            "=" => writer.write_str(" = "),
            "block" => {
                if writer.settings.brace_wrapping.before_function {
                    writer.write_ln();
                    write_block(&child, writer, true)?;
                } else {
                    writer.write(' ');
                    write_block(&child, writer, false)?;
                }
            }
            ";" => writer.write(';'),
            _ => println!(
                "Unexpected kind {} in write_methodmap_property_method.",
                kind
            ),
        }
    }
    writer.write_ln();

    Ok(())
}

fn write_methodmap_property_setter(node: &Node, writer: &mut Writer) -> anyhow::Result<()> {
    let mut cursor = node.walk();

    for child in node.children(&mut cursor) {
        let kind = child.kind();
        match kind.borrow() {
            "set" => writer.write_str("set"),
            "symbol" | "(" | ")" => write_node(&child, writer)?,
            "type" => write_type(&child, writer)?,
            ";" => continue,
            _ => println!(
                "Unexpected kind {} in write_methodmap_property_setter.",
                kind
            ),
        }
    }

    Ok(())
}
