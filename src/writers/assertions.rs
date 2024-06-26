use std::borrow::Borrow;

use tree_sitter::Node;

use super::{
    expressions::write_function_call_arguments, preproc::insert_break, write_comment, write_node,
    Writer,
};

pub fn write_assertion(node: &Node, writer: &mut Writer) -> anyhow::Result<()> {
    let mut cursor = node.walk();

    for child in node.children(&mut cursor) {
        let kind = child.kind();
        match kind.borrow() {
            "assert" | "static_assert" => write_node(&child, writer)?,
            "function_call_arguments" => write_function_call_arguments(&child, writer)?,
            "comment" => write_comment(&child, writer)?,
            ";" => continue,
            _ => println!("Unexpected kind {} in write_assertion.", kind),
        }
    }
    writer.write(';');
    insert_break(&node, writer);

    Ok(())
}
