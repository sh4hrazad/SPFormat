use std::borrow::Borrow;

use tree_sitter::Node;

use super::{write_dimension, write_dynamic_array, write_node, Writer};

pub fn write_expression(node: &Node, writer: &mut Writer) -> anyhow::Result<()> {
    match node.kind().borrow() {
        "symbol" | "null" | "this" | "int_literal " | "bool_literal" | "char_literal"
        | "float_literal" | "string_literal" => write_node(&node, writer)?,
        "binary_expression" => write_binary_expression(node, writer)?,
        "unary_expression" => write_unary_expression(node, writer)?,
        "update_expression" => write_update_expression(node, writer)?,
        "parenthesized_expression" => write_parenthesized_expression(node, writer)?,
        "comma_expression" => write_comma_expression(node, writer)?,
        "scope_access" => write_scope_access(node, writer)?,
        "view_as" => write_view_as(node, writer)?,
        "old_type_cast" => write_old_type_cast(node, writer)?,
        "ternary_expression" => write_ternary_expression(node, writer)?,
        "concatenated_string" => write_concatenated_string(node, writer)?,
        "array_indexed_access" => write_array_indexed_access(node, writer)?,
        "field_access" => write_field_access(node, writer)?,
        "new_instance" => write_new_instance(node, writer)?,
        "function_call" => write_function_call(node, writer)?,
        "assignment_expression" => write_assignment_expression(node, writer)?,
        "array_literal" => write_array_literal(node, writer)?,
        "sizeof_expression" => write_sizeof_expression(node, writer)?,
        _ => write_node(&node, writer)?,
    };

    Ok(())
}

fn write_binary_expression(node: &Node, writer: &mut Writer) -> anyhow::Result<()> {
    write_expression(&node.child_by_field_name("left").unwrap(), writer)?;
    writer.write(' ');
    write_node(&node.child_by_field_name("operator").unwrap(), writer)?;
    writer.write(' ');
    write_expression(&node.child_by_field_name("right").unwrap(), writer)?;

    Ok(())
}

fn write_old_type_cast(node: &Node, writer: &mut Writer) -> anyhow::Result<()> {
    write_old_type(&node.child_by_field_name("type").unwrap(), writer)?;
    write_expression(&node.child_by_field_name("value").unwrap(), writer)?;

    Ok(())
}

pub fn write_old_type(node: &Node, writer: &mut Writer) -> anyhow::Result<()> {
    writer.write_str(node.utf8_text(writer.source)?.borrow());
    writer.write(' ');

    Ok(())
}

fn write_assignment_expression(node: &Node, writer: &mut Writer) -> anyhow::Result<()> {
    write_expression(&node.child_by_field_name("left").unwrap(), writer)?;
    writer.write(' ');
    write_node(&node.child_by_field_name("operator").unwrap(), writer)?;
    writer.write(' ');
    let right_node = node.child_by_field_name("right").unwrap();
    match right_node.kind().borrow() {
        "dynamic_array" => write_dynamic_array(&right_node, writer)?,
        _ => write_expression(&right_node, writer)?,
    }

    Ok(())
}

fn write_array_indexed_access(node: &Node, writer: &mut Writer) -> anyhow::Result<()> {
    let array_node = node.child_by_field_name("array").unwrap();
    match array_node.kind().borrow() {
        "array_indexed_access" => write_array_indexed_access(&array_node, writer)?,
        // TODO: Handle "field_access" here.
        _ => write_node(&array_node, writer)?,
    }
    writer.write('[');
    write_expression(&node.child_by_field_name("index").unwrap(), writer)?;
    writer.write(']');

    Ok(())
}

fn write_field_access(node: &Node, writer: &mut Writer) -> anyhow::Result<()> {
    write_expression(&node.child_by_field_name("target").unwrap(), writer)?;
    writer.write('.');
    write_node(&node.child_by_field_name("field").unwrap(), writer)?;

    Ok(())
}

fn write_new_instance(node: &Node, writer: &mut Writer) -> anyhow::Result<()> {
    writer.write_str("new ");
    write_node(&node.child_by_field_name("class").unwrap(), writer)?;
    write_function_call_arguments(&node.child_by_field_name("arguments").unwrap(), writer)?;

    Ok(())
}

fn write_function_call(node: &Node, writer: &mut Writer) -> anyhow::Result<()> {
    let function_node = node.child_by_field_name("function").unwrap();
    write_expression(&function_node, writer)?;
    write_function_call_arguments(&node.child_by_field_name("arguments").unwrap(), writer)?;

    Ok(())
}

fn write_unary_expression(node: &Node, writer: &mut Writer) -> anyhow::Result<()> {
    write_node(&node.child_by_field_name("operator").unwrap(), writer)?;
    write_expression(&node.child_by_field_name("argument").unwrap(), writer)?;

    Ok(())
}

fn write_parenthesized_expression(node: &Node, writer: &mut Writer) -> anyhow::Result<()> {
    // TODO: Check for literals/symbols to remove unneeded parenthesis.
    writer.write('(');
    let expression_node = node.child_by_field_name("expression").unwrap();
    match expression_node.kind().borrow() {
        "comma_expression" => write_comma_expression(&expression_node, writer)?,
        _ => write_expression(&expression_node, writer)?,
    }
    writer.write(')');

    Ok(())
}

fn write_comma_expression(node: &Node, writer: &mut Writer) -> anyhow::Result<()> {
    write_expression(&node.child_by_field_name("left").unwrap(), writer)?;
    writer.write_str(", ");
    write_expression(&node.child_by_field_name("right").unwrap(), writer)?;

    Ok(())
}

fn write_concatenated_string(node: &Node, writer: &mut Writer) -> anyhow::Result<()> {
    write_node(&node.child_by_field_name("left").unwrap(), writer)?;
    writer.write_str(" ... ");
    let right_node = node.child_by_field_name("right").unwrap();
    match right_node.kind().borrow() {
        "concatenated_string" => write_concatenated_string(&right_node, writer)?,
        _ => write_node(&right_node, writer)?,
    }

    Ok(())
}

fn write_update_expression(node: &Node, writer: &mut Writer) -> anyhow::Result<()> {
    let argument_node = node.child_by_field_name("argument").unwrap();
    let operator_node = node.child_by_field_name("operator").unwrap();
    if operator_node.end_position() <= argument_node.start_position() {
        write_node(&operator_node, writer)?;
        write_expression(&argument_node, writer)?;
    } else {
        write_expression(&argument_node, writer)?;
        write_node(&operator_node, writer)?;
    }

    Ok(())
}

fn write_ternary_expression(node: &Node, writer: &mut Writer) -> anyhow::Result<()> {
    write_expression(&node.child_by_field_name("condition").unwrap(), writer)?;
    writer.write_str(" ? ");
    write_expression(&node.child_by_field_name("consequence").unwrap(), writer)?;
    writer.write_str(" : ");
    write_expression(&node.child_by_field_name("alternative").unwrap(), writer)?;

    Ok(())
}

fn write_scope_access(node: &Node, writer: &mut Writer) -> anyhow::Result<()> {
    write_expression(&node.child_by_field_name("scope").unwrap(), writer)?;
    writer.write_str("::");
    write_expression(&node.child_by_field_name("field").unwrap(), writer)?;

    Ok(())
}

fn write_view_as(node: &Node, writer: &mut Writer) -> anyhow::Result<()> {
    writer.write_str("view_as<");
    write_node(&node.child_by_field_name("type").unwrap(), writer)?;
    writer.write_str(">(");
    write_expression(&node.child_by_field_name("value").unwrap(), writer)?;
    writer.write(')');

    Ok(())
}

fn write_array_literal(node: &Node, writer: &mut Writer) -> anyhow::Result<()> {
    let mut cursor = node.walk();
    writer.write_str("{ ");
    for child in node.children(&mut cursor) {
        match child.kind().borrow() {
            "{" | "}" => continue,
            "," => writer.write_str(", "),
            _ => write_expression(&child, writer)?,
        }
    }
    writer.write_str(" }");

    Ok(())
}

fn write_sizeof_expression(node: &Node, writer: &mut Writer) -> anyhow::Result<()> {
    let mut cursor = node.walk();
    writer.write_str("sizeof ");
    for child in node.children_by_field_name("type", &mut cursor) {
        match child.kind().borrow() {
            "dimension" => write_dimension(&child, writer, true)?,
            _ => write_expression(&child, writer)?,
        }
    }

    Ok(())
}

pub fn write_function_call_arguments(node: &Node, writer: &mut Writer) -> anyhow::Result<()> {
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        match child.kind().borrow() {
            "(" => writer.write('('),
            ")" => {
                if writer.output.ends_with(", ") {
                    writer.output.pop();
                    writer.output.pop();
                }
                writer.write(')')
            }
            "," => writer.write_str(", "),
            "symbol" | "ignore_argument" => write_node(&child, writer)?,
            "named_arg" => write_named_arg(&child, writer)?,
            _ => {
                let kind = child.kind();
                if writer.is_expression(&kind) {
                    write_expression(&child, writer)?
                } else {
                    write_node(&child, writer)?;
                }
            }
        }
    }

    Ok(())
}

fn write_named_arg(node: &Node, writer: &mut Writer) -> anyhow::Result<()> {
    writer.write('.');
    write_node(&node.child_by_field_name("name").unwrap(), writer)?;
    writer.write_str(" = ");
    // FIXME: Always write_node.
    write_node(&node.child_by_field_name("value").unwrap(), writer)?;

    Ok(())
}
