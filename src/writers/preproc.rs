use std::borrow::Borrow;

use tree_sitter::Node;

use super::{next_sibling_kind, next_sibling_start, write_comment, write_node, Writer};

/// Check if there is an inline comment after the statement and don't
/// insert a line break if there is. Otherwise, insert a line break, and
/// an additional one if the following statement is there is more than one
/// empty row.
///
/// # Arguments
///
/// * `node`   - The node which was written.
/// * `writer` - The writer object.
pub fn insert_break(node: &Node, writer: &mut Writer) {
    let next_kind = next_sibling_kind(&node);
    if next_kind == "" {
        // No next sibling, add a break and return.
        writer.write_ln();
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
    writer.write_ln();

    // Check if the next sibling is right after this node.
    // If it's not, limit the amount of empty rows to 1.
    if next_row - node.end_position().row() > 1 {
        writer.write_ln();
    }
}

/// Write a preprocessor include.
///
/// # Arguments
///
/// * `node`   - The preprocessor include node to write.
/// * `writer` - The writer object.
pub fn write_preproc_include(node: &Node, writer: &mut Writer) -> anyhow::Result<()> {
    let mut cursor = node.walk();

    for child in node.children(&mut cursor) {
        let kind = child.kind();
        match kind.borrow() {
            "#include" | "#tryinclude" => {
                write_node(&child, writer)?;
                writer.write(' ')
            }
            "string_literal" | "system_lib_string" => write_node(&child, writer)?,
            _ => println!("Unexpected kind {} in write_preproc_include.", kind),
        }
    }

    insert_break(&node, writer);

    Ok(())
}

/// Write a preprocessor define or macro.
///
/// # Arguments
///
/// * `node`   - The preprocessor define/macro node to write.
/// * `writer` - The writer object.
pub fn write_preproc_define(node: &Node, writer: &mut Writer) -> anyhow::Result<()> {
    let mut cursor = node.walk();

    for child in node.children(&mut cursor) {
        let kind = child.kind();
        match kind.borrow() {
            "#define" => writer.write_str("#define "),
            "symbol" | "identifier" | "macro_param" | "(" | ")" => write_node(&child, writer)?,
            "preproc_arg" => {
                writer.write(' ');
                write_preproc_arg(&child, writer)?;
            }
            "," => writer.write_str(", "),
            _ => println!("Unexpected kind {} in write_preproc_define.", kind),
        }
    }

    insert_break(&node, writer);

    Ok(())
}

/// Write a preprocessor undef.
///
/// # Arguments
///
/// * `node`   - The preprocessor undef node to write.
/// * `writer` - The writer object.
pub fn write_preproc_undefine(node: &Node, writer: &mut Writer) -> anyhow::Result<()> {
    let mut cursor = node.walk();

    for child in node.children(&mut cursor) {
        let kind = child.kind();
        match kind.borrow() {
            "identifier" => write_node(&child, writer)?,
            "#undef" => writer.write_str("#undef "),
            _ => println!("Unexpected kind {} in write_preproc_undefine.", kind),
        }
    }

    insert_break(&node, writer);

    Ok(())
}

/// Write a preprocessor generic:
/// * `#if`
/// * `#elseif`
/// * `#error`
/// * `#warning`
/// * `#pragma`
/// * `#assert`
///
/// # Arguments
///
/// * `node`   - The preprocessor generic node to write.
/// * `writer` - The writer object.
pub fn write_preproc_generic(node: &Node, writer: &mut Writer) -> anyhow::Result<()> {
    let mut cursor = node.walk();

    for child in node.children(&mut cursor) {
        let kind = child.kind();

        match kind.borrow() {
            "#if" | "#elseif" | "#error" | "#warning" | "#pragma" | "#assert" => {
                write_node(&child, writer)?;
                writer.write(' ');
            }
            // got identifier and value together for preproc_arg here
            "preproc_arg" => {
                write_preproc_arg(&child, writer)?;
            }
            "comment" => write_comment(&child, writer)?,
            _ => println!("Unexpected kind {} in write_preproc_generic.", kind),
        }
    }

    insert_break(&node, writer);

    Ok(())
}

/// Write a preprocessor symbol:
/// * `#else`
/// * `#endif`
/// * `#endinput`
/// * `<symbol>`
///
/// # Arguments
///
/// * `node`   - The preprocessor symbol node to write.
/// * `writer` - The writer object.
pub fn write_preproc_symbol(node: &Node, writer: &mut Writer) -> anyhow::Result<()> {
    let kind = node.kind();
    match kind.borrow() {
        "preproc_endif" | "preproc_else" | "preproc_endinput" | "identifier" => {
            write_node(&node, writer)?
        }
        _ => println!("Unexpected kind {} in write_preproc_symbol.", kind),
    }

    insert_break(&node, writer);

    Ok(())
}

/// Write a preprocessor arguments node by trimming it.
///
/// # Arguments
///
/// * `node`   - The preprocessor symbol node to write.
/// * `writer` - The writer object.
fn write_preproc_arg(node: &Node, writer: &mut Writer) -> anyhow::Result<()> {
    let args = node.utf8_text(writer.source)?;
    let args = args.trim();

    if args == "semicolon 1" {
        writer.semicolon = true;
    }

    writer.write_str(args);

    Ok(())
}
