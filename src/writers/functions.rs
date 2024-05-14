use std::borrow::Borrow;

use tree_sitter::Node;

use super::{
    expressions::{write_expression, write_old_type},
    next_sibling_kind, prev_sibling_kind,
    statements::{write_block, write_statement},
    variables::write_type,
    write_dimension, write_fixed_dimension, write_node, Writer,
};

pub fn write_function_declaration(node: &Node, writer: &mut Writer) -> anyhow::Result<()> {
    let nb_lines: usize = usize::try_from(writer.settings.r#break.after_function_decl)?;
    let prev_kind = prev_sibling_kind(&node);

    let mut cursor = node.walk();

    for child in node.children(&mut cursor) {
        let kind = child.kind();

        match kind.borrow() {
            "visibility" | "function_declaration_kind" => {
                write_function_visibility(&child, writer)?;
            }
            "type" => {
                write_type(&child, writer)?;
            }
            "dimension" => write_dimension(&child, writer, true)?,
            "parameter_declarations" => write_argument_declarations(&child, writer)?,
            "identifier" => write_node(&child, writer)?,
            "block" => {
                if writer.settings.brace_wrapping.before_function {
                    writer.breakl();
                    write_block(&child, writer, true)?;
                } else {
                    writer.output.push(' ');
                    write_block(&child, writer, false)?;
                }
            }
            _ => {
                write_statement(&child, writer, false, false)?;
            }
        }
    }

    writer.breakl();

    if !prev_kind.starts_with("preproc_") && prev_kind != "" && prev_kind != "comment" {
        for _ in 0..nb_lines {
            writer.breakl();
        }
    }

    Ok(())
}

pub fn write_argument_declarations(node: &Node, writer: &mut Writer) -> anyhow::Result<()> {
    let mut cursor = node.walk();

    for child in node.children(&mut cursor) {
        match child.kind().borrow() {
            "(" | ")" => write_node(&child, writer)?,
            "rest_argument" => {
                let mut sub_cursor = child.walk();
                for sub_child in child.children(&mut sub_cursor) {
                    match sub_child.kind().borrow() {
                        "type" => write_node(&sub_child, writer)?,
                        "old_type" => write_old_type(&sub_child, writer)?,
                        _ => write_node(&sub_child, writer)?,
                    }
                }
            }
            "argument_declaration" => write_argument_declaration(&child, writer)?,
            "," => writer.output.push_str(", "),
            _ => write_node(&child, writer)?,
        }
    }

    Ok(())
}

fn write_argument_declaration(node: &Node, writer: &mut Writer) -> anyhow::Result<()> {
    let mut cursor = node.walk();

    for child in node.children(&mut cursor) {
        match child.kind().borrow() {
            "const" => writer.output.push_str("const "),
            "argument_type" => write_argument_type(&child, writer)?,
            "identifier" => write_node(&child, writer)?,
            "dimension" => write_dimension(&child, writer, true)?,
            "fixed_dimension" => {
                let next_kind = next_sibling_kind(&child);
                write_fixed_dimension(&child, writer, true)?;
                if next_kind != "dimension" || next_kind != "fixed_dimension" {
                    writer.output.push(' ')
                };
            }
            "=" => writer.output.push_str(" = "),
            _ => write_expression(&child, writer)?,
        }
    }

    Ok(())
}

fn write_argument_type(node: &Node, writer: &mut Writer) -> anyhow::Result<()> {
    let mut cursor = node.walk();

    for child in node.children(&mut cursor) {
        match child.kind().borrow() {
            "&" => {
                let next_kind = next_sibling_kind(&child);
                writer.output.push('&');
                if next_kind != "old_type" && next_kind != "" {
                    writer.output.push(' ')
                };
            }
            "type" => write_type(&child, writer)?,
            "dimension" => write_dimension(&child, writer, true)?,
            _ => write_node(&child, writer)?,
        }
    }

    Ok(())
}

pub fn write_function_visibility(node: &Node, writer: &mut Writer) -> anyhow::Result<()> {
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        write_node(&child, writer)?;
        writer.output.push(' ');
    }

    Ok(())
}
