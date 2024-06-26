use std::borrow::Borrow;

use tree_sitter::Node;

use super::{
    expressions::write_expression, old_variables::write_old_variable_declaration_statement,
    variables::write_variable_declaration_statement, write_comment, write_node, Writer,
};

pub fn write_statement(
    node: &Node,
    writer: &mut Writer,
    do_indent: bool,
    do_break: bool,
) -> anyhow::Result<()> {
    let sp = node.end_position().row();
    let next_sibling = node.next_sibling();
    let kind = node.kind();

    let maybe_has_semicolon = !vec![
        "block",
        "for_statement",
        "while_loop",
        "condition_statement",
        "switch_statement",
    ]
    .contains(&kind.as_ref());

    match kind.borrow() {
        // these may not need ; at the end
        "block" => write_block(&node, writer, do_indent)?,
        "for_statement" => write_for_loop(&node, writer, do_indent)?,
        "while_loop" => write_while_loop(&node, writer, do_indent)?,
        "condition_statement" => write_condition_statement(&node, writer, do_indent)?,
        "switch_statement" => write_switch_statement(&node, writer, do_indent)?,

        // and these may need
        "variable_declaration_statement" => {
            write_variable_declaration_statement(&node, writer, do_indent)?
        }
        "old_variable_declaration_statement" => {
            write_old_variable_declaration_statement(&node, writer, do_indent)?
        }
        "do_while_loop" => write_do_while_loop(&node, writer, do_indent)?,
        "break_statement" => {
            if do_indent {
                writer.write_indent();
            }
            writer.write_str("break");
        }
        "continue_statement" => {
            if do_indent {
                writer.write_indent();
            }
            writer.write_str("continue");
        }
        "return_statement" => write_return_statement(&node, writer, do_indent)?,
        "delete_statement" => write_delete_statement(&node, writer, do_indent)?,
        "expression_statement" => {
            if do_indent {
                writer.write_indent();
            }
            write_expression_statement(&node, writer)?
        }
        _ => {
            println!("Unexpected kind {} in write_statement.", kind);
            write_node(&node, writer)?;
        }
    }

    if maybe_has_semicolon && writer.semicolon {
        writer.write_str(";");
    }

    if do_break {
        if next_sibling.is_none() {
            writer.write_ln();
            return Ok(());
        }
        let st = next_sibling.as_ref().unwrap().start_position().row();

        // Don't add a break if the next sibling is a trailing comment.
        if next_sibling.as_ref().unwrap().kind() == "comment" {
            let st = next_sibling.unwrap().start_position().row();
            if st - sp == 0 {
                return Ok(());
            }
        }

        // Add another break if the next sibling is not right below/next
        // to the current sibling.
        if st - sp > 1 {
            writer.write_ln();
        }

        writer.write_ln();
    }

    Ok(())
}

fn write_for_loop(node: &Node, writer: &mut Writer, do_indent: bool) -> anyhow::Result<()> {
    let mut end_of_conditions_reached = false;
    let mut got_condition_expr = false;
    let mut cursor = node.walk();

    for child in node.children(&mut cursor) {
        let kind = child.kind();

        match kind.borrow() {
            "for" => {
                if do_indent {
                    writer.write_indent();
                }
                write_node(&child, writer)?;
            }
            "(" => write_node(&child, writer)?,
            ")" => {
                end_of_conditions_reached = true;
                write_node(&child, writer)?;
            }
            _ => {
                if writer.is_statement(&kind) {
                    // an initializer
                    if !end_of_conditions_reached {
                        write_statement(&child, writer, false, false)?;

                        continue;
                    }

                    // something to do in for loop
                    if kind == "block" {
                        if writer.settings.brace_wrapping.before_loop {
                            writer.write_ln();
                            write_block(&child, writer, true)?;
                        } else {
                            writer.write(' ');
                            write_block(&child, writer, false)?;
                        }
                    } else {
                        writer.write_ln();
                        writer.indent += 1;
                        write_statement(&child, writer, true, false)?;
                        writer.indent -= 1;
                    }
                } else if writer.is_expression(&kind) {
                    // condition or iteration
                    if writer.output.ends_with(';') {
                        writer.write(' ');
                    }

                    write_expression(&child, writer)?;

                    if !got_condition_expr {
                        writer.write_str("; ");
                        got_condition_expr = true;
                    }
                } else {
                    // ?
                    println!("Unexpected kind {} in write_for_loop.", kind);
                    write_node(&child, writer)?;
                }
            }
        }
    }

    Ok(())
}

fn write_while_loop(node: &Node, writer: &mut Writer, do_indent: bool) -> anyhow::Result<()> {
    let mut end_condition_reached = false;
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        let kind = child.kind();
        match kind.borrow() {
            "while" => {
                if do_indent {
                    writer.write_indent();
                }
                write_node(&child, writer)?;
            }
            "(" => write_node(&child, writer)?,
            ")" => {
                end_condition_reached = true;
                write_node(&child, writer)?;
            }
            _ => {
                if writer.is_statement(&kind) {
                    if end_condition_reached {
                        if kind == "block" {
                            if writer.settings.brace_wrapping.before_loop {
                                writer.write_ln();
                                write_block(&child, writer, true)?;
                            } else {
                                writer.write(' ');
                                write_block(&child, writer, false)?;
                            }
                        } else {
                            writer.write_ln();
                            writer.indent += 1;
                            write_statement(&child, writer, true, false)?;
                            writer.indent -= 1;
                        }
                    } else {
                        write_statement(&child, writer, false, false)?;
                    }
                } else if writer.is_expression(&kind) {
                    if writer.output.ends_with(';') {
                        writer.write(' ');
                    }
                    write_expression(&child, writer)?;
                } else {
                    write_node(&child, writer)?;
                }
            }
        }
    }

    Ok(())
}

fn write_do_while_loop(node: &Node, writer: &mut Writer, do_indent: bool) -> anyhow::Result<()> {
    let mut in_condition = false;
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        let kind = child.kind();
        match kind.borrow() {
            "do" => {
                if do_indent {
                    writer.write_indent();
                }
                writer.write_str("do");
            }
            "while" => {
                in_condition = true;
                writer.write_indent();
                writer.write_str("while");
            }
            "(" => write_node(&child, writer)?,
            ")" => {
                write_node(&child, writer)?;
            }
            _ => {
                if writer.is_statement(&kind) {
                    if in_condition {
                        write_statement(&child, writer, false, false)?;
                        continue;
                    }
                    if kind == "block" {
                        if writer.settings.brace_wrapping.before_loop {
                            writer.write_ln();
                            write_block(&child, writer, true)?;
                        } else {
                            writer.write(' ');
                            write_block(&child, writer, false)?;
                        }
                        writer.write_ln();
                    } else {
                        writer.write_ln();
                        writer.indent += 1;
                        write_statement(&child, writer, true, false)?;
                        writer.indent -= 1;
                    }
                } else if writer.is_expression(&kind) {
                    if writer.output.ends_with(';') {
                        writer.write(' ');
                    }
                    write_expression(&child, writer)?;
                } else {
                    write_node(&child, writer)?;
                }
            }
        }
    }

    Ok(())
}

fn write_switch_statement(node: &Node, writer: &mut Writer, do_indent: bool) -> anyhow::Result<()> {
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        let kind = child.kind();
        match kind.borrow() {
            "switch" => {
                if do_indent {
                    writer.write_indent();
                }
                writer.write_str("switch");
            }
            "(" => write_node(&child, writer)?,
            ")" => {
                write_node(&child, writer)?;
            }
            "{" => {
                if writer.settings.brace_wrapping.before_condition {
                    writer.write_ln();
                    writer.write_indent();
                } else {
                    writer.write(' ');
                }
                writer.write('{');
                writer.write_ln();
                writer.indent += 1;
            }
            "}" => {
                writer.indent -= 1;
                writer.write_indent();
                writer.write('}');
            }
            "switch_case" => write_switch_case(&child, writer)?,
            "switch_default_case" => write_switch_default_case(&child, writer)?,
            _ => {
                if writer.is_expression(&kind) {
                    write_expression(&child, writer)?;
                } else {
                    write_node(&child, writer)?;
                }
            }
        }
    }

    Ok(())
}

fn write_switch_case(node: &Node, writer: &mut Writer) -> anyhow::Result<()> {
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        let kind = child.kind();
        match kind.borrow() {
            "case" => {
                writer.write_indent();
                write_node(&child, writer)?;
                writer.write(' ');
            }
            ":" => {
                writer.write_str(":\n");
            }
            "switch_case_values" => write_switch_case_values(&child, writer)?,
            "comment" => write_comment(&child, writer)?,
            _ => {
                if kind == "block" {
                    write_statement(&child, writer, true, true)?;
                    continue;
                }
                if writer.is_statement(&kind) {
                    writer.indent += 1;
                    write_statement(&child, writer, true, true)?;
                    writer.indent -= 1;
                } else {
                    write_node(&child, writer)?;
                }
            }
        }
    }

    Ok(())
}

fn write_switch_default_case(node: &Node, writer: &mut Writer) -> anyhow::Result<()> {
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        let kind = child.kind();
        match kind.borrow() {
            "default" => {
                writer.write_indent();
                write_node(&child, writer)?;
            }
            ":" => {
                writer.write_str(":\n");
            }
            _ => {
                if kind == "block" {
                    write_statement(&child, writer, true, true)?;
                    continue;
                }
                if writer.is_statement(&kind) {
                    writer.indent += 1;
                    write_statement(&child, writer, true, true)?;
                    writer.indent -= 1;
                } else {
                    write_node(&child, writer)?
                }
            }
        }
    }

    Ok(())
}

fn write_switch_case_values(node: &Node, writer: &mut Writer) -> anyhow::Result<()> {
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        let kind = child.kind();
        match kind.borrow() {
            "comment" => write_comment(&child, writer)?,
            "identifier" => write_node(&child, writer)?,
            "," => writer.write_str(", "),
            _ => {
                if writer.is_expression(&kind) {
                    write_expression(&child, writer)?;
                } else {
                    write_node(&child, writer)?;
                }
            }
        }
    }

    Ok(())
}

fn write_return_statement(node: &Node, writer: &mut Writer, do_indent: bool) -> anyhow::Result<()> {
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        let kind = child.kind();
        match kind.borrow() {
            "comment" => write_comment(&child, writer)?,
            "return" => {
                if do_indent {
                    writer.write_indent();
                }
                writer.write_str("return ");
            }
            ";" => writer.write(';'),
            _ => {
                if writer.is_expression(&kind) {
                    write_expression(&child, writer)?;
                } else {
                    write_node(&child, writer)?
                }
            }
        }
    }

    Ok(())
}

fn write_delete_statement(node: &Node, writer: &mut Writer, do_indent: bool) -> anyhow::Result<()> {
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        let kind = child.kind();
        match kind.borrow() {
            "comment" => write_comment(&child, writer)?,
            "delete" => {
                if do_indent {
                    writer.write_indent();
                }
                writer.write_str("delete ");
            }
            ";" => writer.write(';'),
            _ => {
                if writer.is_expression(&kind) {
                    write_expression(&child, writer)?;
                } else {
                    write_node(&child, writer)?
                }
            }
        }
    }

    Ok(())
}

fn write_expression_statement(node: &Node, writer: &mut Writer) -> anyhow::Result<()> {
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        let kind = child.kind();
        match kind.borrow() {
            "comment" => write_comment(&child, writer)?,
            _ => {
                if writer.is_expression(&kind) {
                    write_expression(&child, writer)?;
                } else {
                    write_node(&child, writer)?
                }
            }
        }
    }

    Ok(())
}

fn write_condition_statement(
    node: &Node,
    writer: &mut Writer,
    do_indent: bool,
) -> anyhow::Result<()> {
    let mut out_of_condition = false;
    let mut else_statement = false;
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        let kind = child.kind();
        match kind.borrow() {
            "if" => {
                if writer.output.ends_with("else") {
                    writer.write(' ');
                } else if do_indent {
                    writer.write_indent();
                }
                write_node(&child, writer)?;
            }
            "else" => {
                writer.write_ln();
                writer.write_indent();
                write_node(&child, writer)?;
                out_of_condition = true;
                else_statement = true;
            }
            "(" => write_node(&child, writer)?,
            ")" => {
                write_node(&child, writer)?;
                out_of_condition = true;
            }
            _ => {
                if writer.is_statement(&kind) {
                    if out_of_condition {
                        if kind == "block" {
                            if writer.settings.brace_wrapping.before_condition {
                                writer.write_ln();
                                write_block(&child, writer, true)?;
                            } else {
                                writer.write(' ');
                                write_block(&child, writer, false)?;
                            }
                        } else {
                            if else_statement && kind == "condition_statement" {
                                write_statement(&child, writer, true, false)?;
                                continue;
                            }
                            writer.write_ln();
                            writer.indent += 1;
                            write_statement(&child, writer, true, false)?;
                            writer.indent -= 1;
                        }
                    } else {
                        write_statement(&child, writer, false, false)?;
                    }
                } else if writer.is_expression(&kind) {
                    if writer.output.ends_with(';') {
                        writer.write(' ');
                    }
                    write_expression(&child, writer)?;
                } else {
                    write_node(&child, writer)?;
                }
            }
        }
    }

    Ok(())
}

pub fn write_block(node: &Node, writer: &mut Writer, do_indent: bool) -> anyhow::Result<()> {
    let mut cursor = node.walk();

    for child in node.children(&mut cursor) {
        let kind = child.kind();
        match kind.borrow() {
            "{" => {
                if do_indent {
                    writer.write_indent();
                }
                write_node(&child, writer)?;
                writer.write_ln();
                writer.indent += 1;
            }
            "}" => {
                writer.indent -= 1;
                writer.write_indent();
                write_node(&child, writer)?;
            }
            "comment" => write_comment(&child, writer)?,
            _ => {
                if writer.is_statement(&kind) {
                    write_statement(&child, writer, true, true)?
                } else {
                    write_node(&child, writer)?
                }
            }
        }
    }

    Ok(())
}
