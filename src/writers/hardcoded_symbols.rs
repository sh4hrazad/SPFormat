use std::borrow::Borrow;

use tree_sitter::Node;

use super::{preproc::insert_break, write_node, Writer};

pub fn write_hardcoded_symbol(node: &Node, writer: &mut Writer) -> anyhow::Result<()> {
    let mut cursor = node.walk();

    for child in node.children(&mut cursor) {
        let kind = child.kind();
        match kind.borrow() {
            "using __intrinsics__.Handle" => write_node(&child, writer)?,
            ";" => continue,
            _ => println!("Unexpected kind {} in write_hardcoded_symbol.", kind),
        }
    }
    writer.write(';');
    insert_break(&node, writer);

    Ok(())
}
