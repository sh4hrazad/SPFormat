use std::borrow::Borrow;

use tree_sitter::Node;

use super::{
    expressions::write_expression, prev_sibling_kind, write_comment, write_fixed_dimension,
    write_node, Writer,
};

pub fn write_enum(node: &Node, writer: &mut Writer) -> anyhow::Result<()> {
    let nb_lines: usize = usize::try_from(writer.settings.r#break.before_enum).unwrap();
    let prev_kind = prev_sibling_kind(&node);

    if !prev_kind.starts_with("preproc_") && prev_kind != "" && prev_kind != "comment" {
        // Insert new lines automatically
        writer.write_str("\n".repeat(nb_lines).as_str());
    }

    let mut cursor = node.walk();

    for child in node.children(&mut cursor) {
        let kind = child.kind();
        match kind.borrow() {
            "enum" => writer.write_str("enum "),
            "symbol" | ":" | ";" => write_node(&child, writer)?,
            "(" => writer.write_str("("),
            ")" => writer.write_str(")"),
            "enum_entries" => write_enum_entries(&child, writer)?,
            _ => {
                if writer.is_expression(&kind) {
                    write_expression(&child, writer)?;
                } else if kind.to_string().ends_with('=') {
                    write_node(&child, writer)?;
                    writer.write(' ');
                } else {
                    println!("Unexpected kind {} in write_enum.", kind);
                }
            }
        }
    }
    writer.write_ln();

    Ok(())
}

fn write_enum_entries(node: &Node, writer: &mut Writer) -> anyhow::Result<()> {
    let mut cursor = node.walk();

    for child in node.children(&mut cursor) {
        let kind = child.kind();
        match kind.borrow() {
            "{" => {
                if writer.settings.brace_wrapping.before_enum {
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
            "enum_entry" => write_enum_entry(&child, writer)?,
            "comment" => write_comment(&child, writer)?,
            "," => continue,
            _ => {
                if writer.is_expression(&kind) {
                    write_expression(&child, writer)?;
                } else if kind.to_string().ends_with('=') {
                    // Match all in place operators, write it, and add a space
                    // to respect the rest of the styling.
                    write_node(&child, writer)?;
                    writer.write(' ');
                } else {
                    println!("Unexpected kind {} in write_enum_entries.", kind);
                }
            }
        }
    }

    Ok(())
}

fn write_enum_entry(node: &Node, writer: &mut Writer) -> anyhow::Result<()> {
    let mut cursor = node.walk();

    writer.write_indent();

    for child in node.children(&mut cursor) {
        let kind = child.kind();
        match kind.borrow() {
            "builtin_type" | "symbol" | "identifier" => write_node(&child, writer)?,
            ":" => writer.write_str(": "),
            "fixed_dimension" => write_fixed_dimension(&child, writer, true)?,
            "=" => writer.write_str(" = "),
            _ => {
                if writer.is_expression(&kind) {
                    write_expression(&child, writer)?;
                } else {
                    println!("Unexpected kind {} in write_enum_entry.", kind);
                }
            }
        }
    }
    writer.write(',');
    writer.write_ln();

    Ok(())
}
