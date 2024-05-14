use std::borrow::Borrow;

use tree_sitter::Node;

use super::expressions::write_expression;
use super::{write_comment, write_dimension, write_fixed_dimension, write_node, Writer};

pub fn write_struct_declaration(node: &Node, writer: &mut Writer) -> anyhow::Result<()> {
    let mut cursor = node.walk();

    for sub_node in node.children(&mut cursor) {
        let kind = sub_node.kind();
        match kind.borrow() {
            "public" | "identifier" => {
                write_node(&sub_node, writer)?;
                writer.write(' ');
            }
            "comment" => {
                writer.write('\t');
                write_comment(&sub_node, writer)?;
            }
            "=" => {
                writer.write_str("=");

                if writer.settings.brace_wrapping.before_struct_ctor {
                    writer.write_ln();
                } else {
                    writer.write(' ');
                }
            }
            "struct_constructor" => write_struct_constructor(&sub_node, writer)?,
            _ => {
                println!("Unexpected kind {} in write_struct_declaration.", kind);
            }
        }
    }

    Ok(())
}

fn write_struct_constructor(node: &Node, writer: &mut Writer) -> anyhow::Result<()> {
    let mut cursor = node.walk();

    for sub_node in node.children(&mut cursor) {
        let kind = sub_node.kind();
        match kind.borrow() {
            "comment" => {
                writer.write('\t');
                write_comment(&sub_node, writer)?;
            }
            "struct_field_value" => write_struct_field_value(&sub_node, writer)?,
            "{" => {
                writer.indent += 1;
                writer.write_str("{\n");
            }
            "}" => {
                writer.indent -= 1;
                writer.write('}');
            }
            ";" => writer.write(';'),
            "," => continue,
            _ => println!("Unexpected kind {} in write_struct_constructor.", kind),
        }
    }
    if !writer.output.ends_with(';') {
        writer.write_str(";\n");
    }

    Ok(())
}

fn write_struct_field_value(node: &Node, writer: &mut Writer) -> anyhow::Result<()> {
    let mut cursor = node.walk();
    let mut key = true;
    for sub_node in node.children(&mut cursor) {
        match sub_node.kind().borrow() {
            "comment" => {
                writer.write('\t');
                write_comment(&sub_node, writer)?;
            }
            "identifier" => {
                if key {
                    key = false;
                    writer
                        .output
                        .push_str(writer.indent_string.repeat(writer.indent).as_str());
                    write_node(&sub_node, writer)?;
                } else {
                    key = true;
                    write_node(&sub_node, writer)?;
                    writer.write_str(",\n");
                }
            }
            "=" => writer.write_str(" = "),
            // value
            _ => {
                write_expression(&sub_node, writer)?;
                writer.write_str(",\n")
            }
        }
    }

    Ok(())
}

pub fn write_struct(node: &Node, writer: &mut Writer) -> anyhow::Result<()> {
    let mut cursor = node.walk();
    for sub_node in node.children(&mut cursor) {
        match sub_node.kind().borrow() {
            "comment" => {
                writer.write('\t');
                write_comment(&sub_node, writer)?;
            }
            "struct" => writer.write_str("struct "),
            "identifier" => write_node(&sub_node, writer)?,
            "{" => {
                writer.indent += 1;
                writer.write_str("\n{\n");
            }
            "}" => {
                writer.indent -= 1;
                writer.write('}');
            }
            "struct_field" => write_struct_field(&sub_node, writer)?,
            _ => writer.write_str(";\n"),
        }
    }

    Ok(())
}

fn write_struct_field(node: &Node, writer: &mut Writer) -> anyhow::Result<()> {
    writer
        .output
        .push_str(writer.indent_string.repeat(writer.indent).as_str());

    let mut cursor = node.walk();
    for sub_node in node.children(&mut cursor) {
        let kind = sub_node.kind();
        match kind.borrow() {
            "public" => writer.write_str("public "),
            "const" => writer.write_str("const "),
            "type" => write_node(&sub_node, writer)?,
            "identifier" => {
                writer.write(' ');
                write_node(&sub_node, writer)?;
            }
            "fixed_dimension" => write_fixed_dimension(&sub_node, writer, true)?,
            "dimension" => write_dimension(&sub_node, writer, true)?,
            ";" => writer.write(';'),
            _ => println!("Unexpected kind {} in write_struct_field.", kind),
        }
    }

    Ok(())
}
