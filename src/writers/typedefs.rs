use std::borrow::Borrow;

use tree_sitter::Node;

use super::{
    functions::write_argument_declarations, next_sibling_kind, prev_sibling_kind,
    variables::write_type, write_comment, write_dimension, write_fixed_dimension, write_node,
    Writer,
};

pub fn write_typedef(node: &Node, writer: &mut Writer) -> anyhow::Result<()> {
    let nb_lines: usize = usize::try_from(writer.settings.r#break.after_function_decl).unwrap();
    let prev_kind = prev_sibling_kind(&node);

    if !prev_kind.starts_with("preproc_") && prev_kind != "" && prev_kind != "comment" {
        // Insert new lines automatically
        writer.write_str("\n".repeat(nb_lines).as_str());
    }

    let mut cursor = node.walk();

    for child in node.children(&mut cursor) {
        let kind = child.kind();
        match kind.borrow() {
            "typedef" => writer.write_str("typedef "),
            "identifier" => write_node(&child, writer)?,
            "=" => writer.write_str(" = "),
            "typedef_expression" => write_typedef_expression(&child, writer)?,
            ";" => continue,
            _ => {
                println!("Unexpected kind {} in write_typedef.", kind);
            }
        }
    }
    writer.write(';');
    writer.write_ln();

    Ok(())
}

pub fn write_typeset(node: &Node, writer: &mut Writer) -> anyhow::Result<()> {
    let nb_lines: usize = usize::try_from(writer.settings.r#break.after_function_decl).unwrap();
    let prev_kind = prev_sibling_kind(&node);

    if !prev_kind.starts_with("preproc_") && prev_kind != "" && prev_kind != "comment" {
        // Insert new lines automatically
        writer.write_str("\n".repeat(nb_lines).as_str());
    }

    let mut cursor = node.walk();

    for child in node.children(&mut cursor) {
        let kind = child.kind();
        match kind.borrow() {
            "typeset" => writer.write_str("typeset "),
            "identifier" => write_node(&child, writer)?,
            "{" => {
                if writer.settings.brace_wrapping.before_typeset {
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
            "typedef_expression" => {
                let next_kind = next_sibling_kind(&child);
                write_typedef_expression(&child, writer)?;
                writer.write(';');

                if next_kind != "" {
                    writer.write_ln();
                }
            }
            "comment" => write_comment(&child, writer)?,
            ";" => continue,
            _ => {
                println!("Unexpected kind {} in write_typeset.", kind);
            }
        }
    }
    writer.write(';');
    writer.write_ln();

    Ok(())
}

fn write_typedef_expression(node: &Node, writer: &mut Writer) -> anyhow::Result<()> {
    let mut cursor = node.walk();

    writer.write_indent();

    for child in node.children(&mut cursor) {
        let kind = child.kind();
        match kind.borrow() {
            "function" => writer.write_str("function "),
            "type" => write_type(&child, writer)?,
            "dimension" => write_dimension(&child, writer, false)?,
            "fixed_dimension" => write_fixed_dimension(&child, writer, false)?,
            "parameter_declarations" => write_argument_declarations(&child, writer)?,
            "(" | ")" => continue,
            _ => {
                println!("Unexpected kind {} in write_typedef_expression.", kind);
            }
        }
    }

    Ok(())
}
