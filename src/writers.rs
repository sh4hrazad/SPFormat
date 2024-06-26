use std::{
    borrow::{Borrow, Cow},
    collections::HashSet,
};

use tree_sitter::{Node, Point};

use crate::settings::Settings;

use self::{expressions::write_expression, preproc::insert_break};

pub mod alias;
pub mod assertions;
pub mod enum_structs;
pub mod enums;
pub mod expressions;
pub mod functags;
pub mod functions;
pub mod hardcoded_symbols;
pub mod methodmaps;
pub mod old_variables;
pub mod preproc;
pub mod source_file;
pub mod statements;
pub mod structs;
pub mod typedefs;
pub mod variables;

pub struct Writer<'a> {
    pub output: String,
    pub source: &'a [u8],
    // pub language: &'a Language,
    pub indent: usize,
    pub indent_string: String,
    pub skip: u8,
    pub semicolon: bool,
    pub settings: &'a Settings,
    pub statement_kinds: HashSet<&'static str>,
    pub expression_kinds: HashSet<&'static str>,
    pub literal_kinds: HashSet<&'static str>,
}

impl Writer<'_> {
    fn write_str(&mut self, s: &str) {
        self.output.push_str(s);
    }

    fn write(&mut self, ch: char) {
        self.output.push(ch);
    }

    fn write_indent(&mut self) {
        self.write_str(self.indent_string.repeat(self.indent).as_str());
    }

    fn write_ln(&mut self) {
        self.write('\n');
    }

    fn is_statement(&self, kind: &Cow<str>) -> bool {
        return self.statement_kinds.contains(&kind.as_ref());
    }

    fn is_expression(&self, kind: &Cow<str>) -> bool {
        return self.expression_kinds.contains(&kind.as_ref()) || self.is_literal(kind);
    }

    fn is_literal(&self, kind: &Cow<str>) -> bool {
        return self.literal_kinds.contains(&kind.as_ref());
    }
}

pub fn write_comment(node: &Node, writer: &mut Writer) -> anyhow::Result<()> {
    let prev_node = node.prev_named_sibling();

    if !prev_node.is_none() {
        let prev_node = prev_node.unwrap();

        if node.start_position().row() == prev_node.end_position().row() {
            // Previous node is on the same line, simply add a tab.
            let indent = writer.indent_string.as_str().to_string();

            writer.write_str(&indent);
        } else {
            // Previous node is on a different line, indent the comment.
            writer.write_indent();
        }
    }

    writer.write_str(node.utf8_text(writer.source)?.trim());

    insert_break(&node, writer);

    Ok(())
}

fn write_dynamic_array(node: &Node, writer: &mut Writer) -> anyhow::Result<()> {
    writer.write_str("new ");

    let mut cursor = node.walk();

    for child in node.children(&mut cursor) {
        match child.kind().borrow() {
            "type" => write_node(&child, writer)?,
            // TODO: Handle different cases here.
            _ => write_node(&child, writer)?,
        }
    }

    Ok(())
}

fn write_dimension(node: &Node, writer: &mut Writer, insert_space: bool) -> anyhow::Result<()> {
    let next_kind = next_sibling_kind(&node);

    writer.write_str("[]");

    if insert_space && next_kind != "dimension" && next_kind != "fixed_dimension" {
        writer.write(' ')
    };

    Ok(())
}

fn write_fixed_dimension(
    node: &Node,
    writer: &mut Writer,
    insert_space: bool,
) -> anyhow::Result<()> {
    let next_kind = next_sibling_kind(&node);

    let mut cursor = node.walk();

    writer.write('[');

    for child in node.children(&mut cursor) {
        match child.kind().borrow() {
            "[" | "]" => continue,
            _ => write_expression(&child, writer)?,
        }
    }

    writer.write(']');

    if insert_space && next_kind != "dimension" && next_kind != "fixed_dimension" {
        writer.write(' ')
    };

    Ok(())
}

fn write_node(node: &Node, writer: &mut Writer) -> anyhow::Result<()> {
    writer.write_str(node.utf8_text(writer.source)?.as_ref());

    Ok(())
}

fn next_sibling_kind(node: &Node) -> String {
    let next_node = node.next_sibling();

    if next_node.is_none() {
        return String::from("");
    }

    return String::from(next_node.unwrap().kind());
}

fn prev_sibling_kind(node: &Node) -> String {
    let prev_node = node.prev_sibling();

    if prev_node.is_none() {
        return String::from("");
    }

    return String::from(prev_node.unwrap().kind());
}

fn next_sibling_start(node: &Node) -> Option<Point> {
    let next_node = node.next_sibling();

    if next_node.is_none() {
        return None;
    }

    return Some(next_node.unwrap().start_position());
}

#[allow(dead_code)]
fn prev_sibling_end(node: &Node) -> Option<Point> {
    let prev_node = node.prev_sibling();

    if prev_node.is_none() {
        return None;
    }

    return Some(prev_node.unwrap().end_position());
}

/// Returns the length of a node.
///
/// # Arguments
///
/// * `node`   - The node to compute the length of.
pub fn node_len(node: &Node) -> usize {
    usize::try_from(node.end_byte() - node.start_byte()).unwrap()
}
