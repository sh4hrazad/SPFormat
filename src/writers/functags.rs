use std::borrow::Borrow;

use tree_sitter::Node;

use super::{
    expressions::write_old_type, functions::write_argument_declarations, next_sibling_kind,
    prev_sibling_kind, write_comment, write_node, Writer,
};

pub fn write_functag(node: &Node, writer: &mut Writer) -> anyhow::Result<()> {
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
            "functag" => writer.write_str("functag "),
            "public" => writer.write_str("public"),
            "old_type" => write_old_type(&child, writer)?,
            "identifier" => {
                write_node(&child, writer)?;
                writer.write(' ')
            }
            "parameter_declarations" => write_argument_declarations(&child, writer)?,
            ";" => continue,
            _ => {
                println!("Unexpected kind {} in write_functag.", kind);
            }
        }
    }
    writer.write(';');
    writer.write_ln();

    Ok(())
}

pub fn write_funcenum(node: &Node, writer: &mut Writer) -> anyhow::Result<()> {
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
            "funcenum" => writer.write_str("funcenum "),
            "identifier" => write_node(&child, writer)?,
            "{" => {
                if writer.settings.brace_wrapping.before_funcenum {
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
            "funcenum_member" => {
                let next_kind = next_sibling_kind(&child);
                write_funcenum_member(&child, writer)?;
                writer.write(',');

                if next_kind != "" {
                    writer.write_ln();
                }
            }
            "comment" => write_comment(&child, writer)?,
            ";" | "," => continue,
            _ => {
                println!("Unexpected kind {} in write_funcenum.", kind);
            }
        }
    }
    writer.write(';');
    writer.write_ln();

    Ok(())
}

fn write_funcenum_member(node: &Node, writer: &mut Writer) -> anyhow::Result<()> {
    let mut cursor = node.walk();

    writer.write_indent();

    for child in node.children(&mut cursor) {
        let kind = child.kind();
        match kind.borrow() {
            "public" => writer.write_str("public "),
            "old_type" => write_old_type(&child, writer)?,
            "parameter_declarations" => write_argument_declarations(&child, writer)?,
            _ => {
                println!("Unexpected kind {} in write_funcenum_member.", kind);
            }
        }
    }

    Ok(())
}
