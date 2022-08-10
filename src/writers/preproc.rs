use super::{next_sibling_kind, next_sibling_start, write_comment, write_node, Writer};
use std::{borrow::Borrow, str::Utf8Error};

use tree_sitter::Node;

/// Write a preprocessor include, and add a specified number of breaks after the
/// statement if the next statement is not preprocessor statement.
///
/// # Arguments
///
/// * `node`   - The preprocessor include node to write.
/// * `writer` - The writer object.
pub fn write_preproc_include(node: Node, writer: &mut Writer) -> Result<(), Utf8Error> {
    let mut cursor = node.walk();

    for child in node.children(&mut cursor) {
        let kind = child.kind();
        match kind.borrow() {
            "#include" | "#tryinclude" => {
                write_node(&child, writer)?;
                writer.output.push(' ')
            }
            "string_literal" | "system_lib_string" => write_node(&child, writer)?,
            _ => println!("Unexpected kind {} in write_preproc_include.", kind),
        }
    }

    break_after_statement(&node, writer);

    Ok(())
}

/// Check if there is an inline comment after the statement and don't
/// insert a line break if there is. Otherwise, insert a line break, and
/// an additional one if the following statement is there is more than one
/// empty row.
///
/// # Arguments
///
/// * `node`   - The node which was written.
/// * `writer` - The writer object.
pub fn break_after_statement(node: &Node, writer: &mut Writer) {
    let next_kind = next_sibling_kind(&node);
    if next_kind == "" {
        // No next sibling, add a break and return.
        writer.breakl();
        return;
    }

    let next_row = next_sibling_start(&node).unwrap().row();

    // If the next sibling is an inline comment, make sure it is on
    // the same line to avoid an unnecessary break.
    if next_kind == "comment" && next_row == node.end_position().row() {
        return;
    }

    // Insert a line break no matter what,
    // consecutive includes cannot be on the same line.
    writer.breakl();

    // Check if the next sibling is right after this node.
    // If it's not, limit the amount of empty rows to 1.
    if next_row - node.end_position().row() > 1 {
        writer.breakl();
    }
}

pub fn write_preproc_define(node: Node, writer: &mut Writer) -> Result<(), Utf8Error> {
    let mut cursor = node.walk();

    for sub_node in node.children(&mut cursor) {
        match sub_node.kind().borrow() {
            "symbol" => write_node(&sub_node, writer)?,
            "preproc_arg" => {
                writer.output.push(' ');
                write_preproc_arg(sub_node, writer)?;
            }
            "comment" => {
                writer.output.push('\t');
                write_comment(sub_node, writer)?;
            }
            "#define" => writer.output.push_str("#define "),
            "\n" | _ => {}
        }
    }
    if !writer.output.ends_with('\n') {
        writer.breakl();
    }

    Ok(())
}

pub fn write_preproc_undefine(node: Node, writer: &mut Writer) -> Result<(), Utf8Error> {
    let mut cursor = node.walk();

    for sub_node in node.children(&mut cursor) {
        match sub_node.kind().borrow() {
            "symbol" => write_node(&sub_node, writer)?,
            "comment" => {
                writer.output.push('\t');
                write_comment(sub_node, writer)?;
            }
            "#undef" => writer.output.push_str("#undef "),
            "\n" | _ => {}
        }
    }
    if !writer.output.ends_with('\n') {
        writer.breakl();
    }

    Ok(())
}

pub fn write_preproc_generic(node: Node, writer: &mut Writer) -> Result<(), Utf8Error> {
    let mut cursor = node.walk();

    for sub_node in node.children(&mut cursor) {
        match sub_node.kind().borrow() {
            "symbol" => write_node(&sub_node, writer)?,
            "comment" => {
                writer.output.push('\t');
                write_comment(sub_node, writer)?;
            }
            "preproc_defined_condition" => write_node(&sub_node, writer)?,
            "#if" => writer.output.push_str("#if "),
            "#endif" => writer.output.push_str("#endif"),
            "#else" => writer.output.push_str("#else"),
            "#endinput" => writer.output.push_str("#else"),
            "#pragma" => writer.output.push_str("#pragma "),
            "\n" | _ => {}
        }
    }
    if !writer.output.ends_with('\n') {
        writer.breakl();
    }

    Ok(())
}

fn write_preproc_arg(node: Node, writer: &mut Writer) -> Result<(), Utf8Error> {
    let args = node.utf8_text(writer.source)?;
    writer.output.push_str(args.trim());

    Ok(())
}
