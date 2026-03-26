//! COBALT COBOL Parser
//!
//! Parses a COBALT-dialect COBOL source file into the intermediate
//! representation defined in `cobalt_ir`.

use cobalt_ir::*;
use pest::Parser;
use pest_derive::Parser;
use std::collections::HashMap;

#[derive(Parser)]
#[grammar = "grammar.pest"]
struct CobaltParser;

#[derive(Debug, thiserror::Error)]
pub enum ParseError {
    #[error("Parse error: {0}")]
    Pest(#[from] pest::error::Error<Rule>),
    #[error("Invalid level number: {0}")]
    InvalidLevel(String),
    #[error("Invalid PIC clause at '{0}'")]
    InvalidPic(String),
}

/// Parse a COBALT COBOL source string into a `CobaltApp`.
pub fn parse(source: &str) -> Result<CobaltApp, ParseError> {
    // Normalize: strip leading sequence-number columns (cols 1-6) if present,
    // and uppercase everything (COBOL is case-insensitive).
    let normalized = normalize_source(source);
    let pairs = CobaltParser::parse(Rule::program, &normalized)?;

    let mut state: StateMap = HashMap::new();
    let mut screens: Vec<Screen> = Vec::new();
    let mut handlers: Vec<Handler> = Vec::new();
    let mut paragraphs: HashMap<String, Paragraph> = HashMap::new();

    for pair in pairs {
        if pair.as_rule() == Rule::program {
            for inner in pair.into_inner() {
                match inner.as_rule() {
                    Rule::data_division => {
                        parse_data_division(inner, &mut state, &mut screens);
                    }
                    Rule::procedure_division => {
                        parse_procedure_division(inner, &mut handlers, &mut paragraphs);
                    }
                    _ => {}
                }
            }
        }
    }

    Ok(CobaltApp {
        screens,
        state,
        handlers,
        paragraphs,
    })
}

/// Normalize COBOL source: uppercase, strip sequence numbers, collapse whitespace.
fn normalize_source(source: &str) -> String {
    source
        .lines()
        .map(|line| {
            let trimmed = line.trim();
            // Skip comment lines (starting with *)
            if trimmed.starts_with('*') && !trimmed.starts_with("*>") {
                return String::new();
            }
            trimmed.to_uppercase()
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// Intermediate flat entry before tree construction.
#[derive(Debug)]
struct FlatEntry {
    level: u8,
    name: String,
    pic: Option<PicClause>,
    value: Option<String>,
    binding: Option<String>,
    action: Option<String>,
    navigate: Option<String>,
    fg_color: Option<u8>,
    bg_color: Option<u8>,
    conditions: Vec<(String, String)>,
}

fn parse_data_division(
    pair: pest::iterators::Pair<Rule>,
    state: &mut StateMap,
    screens: &mut Vec<Screen>,
) {
    for inner in pair.into_inner() {
        match inner.as_rule() {
            Rule::working_storage => {
                parse_working_storage(inner, state);
            }
            Rule::screen_section => {
                parse_screen_section(inner, screens);
            }
            _ => {}
        }
    }
}

fn parse_working_storage(pair: pest::iterators::Pair<Rule>, state: &mut StateMap) {
    let mut entries: Vec<FlatEntry> = Vec::new();

    for inner in pair.into_inner() {
        match inner.as_rule() {
            Rule::working_storage_entry => {
                for entry_inner in inner.into_inner() {
                    match entry_inner.as_rule() {
                        Rule::data_entry => {
                            let entry = parse_data_entry(entry_inner);
                            entries.push(entry);
                        }
                        Rule::level_88 => {
                            let (cond_name, cond_value) = parse_level_88(entry_inner);
                            // Attach to the most recent non-88 entry
                            if let Some(last) = entries.last_mut() {
                                last.conditions.push((cond_name, cond_value));
                            }
                        }
                        _ => {}
                    }
                }
            }
            _ => {}
        }
    }

    // Convert flat entries to state fields
    for entry in entries {
        if entry.level >= 5 {
            if let Some(pic) = entry.pic {
                state.insert(
                    entry.name.clone(),
                    StateField {
                        name: entry.name,
                        pic,
                        default_value: entry.value,
                        conditions: entry.conditions,
                    },
                );
            }
        }
    }
}

fn parse_screen_section(pair: pest::iterators::Pair<Rule>, screens: &mut Vec<Screen>) {
    let mut flat_entries: Vec<FlatEntry> = Vec::new();

    for inner in pair.into_inner() {
        if inner.as_rule() == Rule::screen_entry {
            for entry_inner in inner.into_inner() {
                if entry_inner.as_rule() == Rule::data_entry {
                    flat_entries.push(parse_data_entry(entry_inner));
                }
            }
        }
    }

    // Build tree from flat entries using level numbers
    let nodes = build_tree(&flat_entries);
    for node in nodes {
        if let Node::Container { ref name, .. } = node {
            screens.push(Screen {
                name: name.clone(),
                root: node,
            });
        }
    }
}

fn parse_data_entry(pair: pest::iterators::Pair<Rule>) -> FlatEntry {
    let mut level = 0u8;
    let mut name = String::new();
    let mut pic = None;
    let mut value = None;
    let mut binding = None;
    let mut action = None;
    let mut navigate = None;
    let mut fg_color = None;
    let mut bg_color = None;

    for inner in pair.into_inner() {
        match inner.as_rule() {
            Rule::level_number => {
                level = inner.as_str().parse().unwrap_or(0);
            }
            Rule::ident => {
                name = inner.as_str().to_string();
            }
            Rule::clause => {
                for clause in inner.into_inner() {
                    match clause.as_rule() {
                        Rule::pic_clause => {
                            pic = Some(parse_pic_clause(clause));
                        }
                        Rule::value_clause => {
                            value = Some(parse_value_clause(clause));
                        }
                        Rule::using_clause => {
                            for c in clause.into_inner() {
                                if c.as_rule() == Rule::ident {
                                    binding = Some(c.as_str().to_string());
                                }
                            }
                        }
                        Rule::on_action_clause => {
                            for c in clause.into_inner() {
                                if c.as_rule() == Rule::ident {
                                    action = Some(c.as_str().to_string());
                                }
                            }
                        }
                        Rule::go_to_screen_clause => {
                            for c in clause.into_inner() {
                                if c.as_rule() == Rule::ident {
                                    navigate = Some(c.as_str().to_string());
                                }
                            }
                        }
                        Rule::bg_color_clause => {
                            for c in clause.into_inner() {
                                if c.as_rule() == Rule::digits {
                                    bg_color = c.as_str().parse().ok();
                                }
                            }
                        }
                        Rule::fg_color_clause => {
                            for c in clause.into_inner() {
                                if c.as_rule() == Rule::digits {
                                    fg_color = c.as_str().parse().ok();
                                }
                            }
                        }
                        _ => {}
                    }
                }
            }
            _ => {}
        }
    }

    FlatEntry {
        level,
        name,
        pic,
        value,
        binding,
        action,
        navigate,
        fg_color,
        bg_color,
        conditions: Vec::new(),
    }
}

fn parse_pic_clause(pair: pest::iterators::Pair<Rule>) -> PicClause {
    let mut kind = PicKind::Alphanumeric;
    let mut width = 1usize;
    let mut decimals = None;

    for inner in pair.into_inner() {
        match inner.as_rule() {
            Rule::pic_kind => {
                kind = match inner.as_str() {
                    "9" => PicKind::Numeric,
                    "A" => PicKind::Alphabetic,
                    _ => PicKind::Alphanumeric,
                };
            }
            Rule::pic_width => {
                for d in inner.into_inner() {
                    if d.as_rule() == Rule::digits {
                        width = d.as_str().parse().unwrap_or(1);
                    }
                }
            }
            Rule::pic_decimal => {
                for d in inner.into_inner() {
                    if d.as_rule() == Rule::pic_width {
                        for dd in d.into_inner() {
                            if dd.as_rule() == Rule::digits {
                                decimals = dd.as_str().parse().ok();
                            }
                        }
                    }
                }
            }
            _ => {}
        }
    }

    PicClause {
        kind,
        width,
        decimals,
    }
}

fn parse_value_clause(pair: pest::iterators::Pair<Rule>) -> String {
    for inner in pair.into_inner() {
        match inner.as_rule() {
            Rule::quoted_string => {
                let s = inner.as_str();
                // Strip surrounding quotes
                return s[1..s.len() - 1].to_string();
            }
            Rule::digits => {
                return inner.as_str().to_string();
            }
            _ => {}
        }
    }
    String::new()
}

fn parse_level_88(pair: pest::iterators::Pair<Rule>) -> (String, String) {
    let mut name = String::new();
    let mut value = String::new();

    for inner in pair.into_inner() {
        match inner.as_rule() {
            Rule::ident => {
                name = inner.as_str().to_string();
            }
            Rule::quoted_string => {
                let s = inner.as_str();
                value = s[1..s.len() - 1].to_string();
            }
            Rule::digits => {
                value = inner.as_str().to_string();
            }
            _ => {}
        }
    }

    (name, value)
}

/// Build a tree of `Node` from flat level-numbered entries.
/// Uses a stack-based algorithm: when we see a level that is deeper than
/// the current, we push onto the stack. When we see a level that is the
/// same or shallower, we pop back up.
fn build_tree(entries: &[FlatEntry]) -> Vec<Node> {
    if entries.is_empty() {
        return Vec::new();
    }

    // Stack: (level, node, children_so_far)
    let mut stack: Vec<(u8, FlatEntry, Vec<Node>)> = Vec::new();
    let mut result: Vec<Node> = Vec::new();

    // We'll process entries and build the tree
    // First pass: identify which entries are containers (have children)
    let container_levels = find_container_levels(entries);

    for entry in entries {
        // Pop entries from stack that are at same or deeper level
        while let Some((top_level, _, _)) = stack.last() {
            if *top_level >= entry.level {
                let (_, popped_entry, children) = stack.pop().unwrap();
                let node = make_node(&popped_entry, children, container_levels.contains(&popped_entry.level));
                if let Some((_, _, parent_children)) = stack.last_mut() {
                    parent_children.push(node);
                } else {
                    result.push(node);
                }
            } else {
                break;
            }
        }

        // Push current entry
        stack.push((
            entry.level,
            FlatEntry {
                level: entry.level,
                name: entry.name.clone(),
                pic: entry.pic.clone(),
                value: entry.value.clone(),
                binding: entry.binding.clone(),
                action: entry.action.clone(),
                navigate: entry.navigate.clone(),
                fg_color: entry.fg_color,
                bg_color: entry.bg_color,
                conditions: entry.conditions.clone(),
            },
            Vec::new(),
        ));
    }

    // Drain remaining stack
    while let Some((_, entry, children)) = stack.pop() {
        let is_container = find_container_levels(entries).contains(&entry.level);
        let node = make_node(&entry, children, is_container);
        if let Some((_, _, parent_children)) = stack.last_mut() {
            parent_children.push(node);
        } else {
            result.push(node);
        }
    }

    result
}

/// Determine which levels act as containers (i.e., have children at deeper levels).
fn find_container_levels(entries: &[FlatEntry]) -> Vec<u8> {
    let mut containers = Vec::new();
    for (i, entry) in entries.iter().enumerate() {
        if i + 1 < entries.len() && entries[i + 1].level > entry.level {
            if !containers.contains(&entry.level) {
                containers.push(entry.level);
            }
        }
    }
    containers
}

/// Convert a FlatEntry into a Node.
fn make_node(entry: &FlatEntry, children: Vec<Node>, is_container: bool) -> Node {
    let style = Style {
        fg_color: entry.fg_color,
        bg_color: entry.bg_color,
    };

    // If it has children or is explicitly a container level, make a Container
    if !children.is_empty() || (is_container && entry.pic.is_none()) {
        return Node::Container {
            name: entry.name.clone(),
            children,
            style,
        };
    }

    // If it has an action or navigation target but no PIC, it's a button
    if entry.action.is_some() || entry.navigate.is_some() {
        return Node::Button {
            name: entry.name.clone(),
            label: entry.value.clone().unwrap_or_else(|| entry.name.clone()),
            action: entry.action.clone(),
            navigate: entry.navigate.clone(),
            style,
        };
    }

    // If it has a VALUE but no PIC, treat as button/label
    if entry.pic.is_none() {
        if entry.value.is_some() {
            return Node::Button {
                name: entry.name.clone(),
                label: entry.value.clone().unwrap_or_default(),
                action: entry.action.clone(),
                navigate: entry.navigate.clone(),
                style,
            };
        }
        // Fallback: empty container
        return Node::Container {
            name: entry.name.clone(),
            children,
            style,
        };
    }

    let pic = entry.pic.clone().unwrap();
    match pic.kind {
        PicKind::Numeric => Node::Numeric {
            name: entry.name.clone(),
            pic,
            value: entry.value.clone(),
            binding: entry.binding.clone(),
            style,
        },
        _ => Node::Text {
            name: entry.name.clone(),
            pic,
            value: entry.value.clone(),
            binding: entry.binding.clone(),
            style,
        },
    }
}

fn parse_procedure_division(
    pair: pest::iterators::Pair<Rule>,
    handlers: &mut Vec<Handler>,
    paragraphs: &mut HashMap<String, Paragraph>,
) {
    for inner in pair.into_inner() {
        if inner.as_rule() == Rule::paragraph {
            let mut para_name = String::new();
            let mut statements = Vec::new();

            for p in inner.into_inner() {
                match p.as_rule() {
                    Rule::paragraph_name => {
                        for name in p.into_inner() {
                            if name.as_rule() == Rule::ident {
                                para_name = name.as_str().to_string();
                            }
                        }
                    }
                    Rule::statement => {
                        if let Some(stmt) = parse_statement(p) {
                            statements.push(stmt);
                        }
                    }
                    _ => {}
                }
            }

            if !para_name.is_empty() {
                handlers.push(Handler {
                    name: para_name.clone(),
                    paragraph_name: para_name.clone(),
                });
                paragraphs.insert(
                    para_name.clone(),
                    Paragraph {
                        name: para_name,
                        statements,
                    },
                );
            }
        }
    }
}

fn parse_statement(pair: pest::iterators::Pair<Rule>) -> Option<Statement> {
    for inner in pair.into_inner() {
        match inner.as_rule() {
            Rule::move_stmt => return Some(parse_move_stmt(inner)),
            Rule::add_stmt => return Some(parse_add_stmt(inner)),
            Rule::subtract_stmt => return Some(parse_subtract_stmt(inner)),
            Rule::multiply_stmt => return Some(parse_multiply_stmt(inner)),
            Rule::divide_stmt => return Some(parse_divide_stmt(inner)),
            Rule::display_stmt => return Some(parse_display_stmt(inner)),
            Rule::perform_stmt => return Some(parse_perform_stmt(inner)),
            Rule::stop_stmt => return Some(Statement::StopRun),
            Rule::if_stmt => return Some(parse_if_stmt(inner)),
            Rule::string_stmt => return Some(parse_string_stmt(inner)),
            _ => {}
        }
    }
    None
}

fn parse_expr(pair: pest::iterators::Pair<Rule>) -> Expr {
    for inner in pair.into_inner() {
        match inner.as_rule() {
            Rule::quoted_string => {
                let s = inner.as_str();
                return Expr::Literal(s[1..s.len() - 1].to_string());
            }
            Rule::digits => {
                if let Ok(n) = inner.as_str().parse::<f64>() {
                    return Expr::NumericLiteral(n);
                }
                return Expr::Literal(inner.as_str().to_string());
            }
            Rule::ident => {
                return Expr::Variable(inner.as_str().to_string());
            }
            _ => {}
        }
    }
    Expr::Literal(String::new())
}

fn parse_move_stmt(pair: pest::iterators::Pair<Rule>) -> Statement {
    let mut exprs = Vec::new();
    let mut target = String::new();
    for inner in pair.into_inner() {
        match inner.as_rule() {
            Rule::expr => exprs.push(parse_expr(inner)),
            Rule::ident => target = inner.as_str().to_string(),
            _ => {}
        }
    }
    Statement::Move {
        source: exprs.into_iter().next().unwrap_or(Expr::Literal(String::new())),
        target,
    }
}

fn parse_add_stmt(pair: pest::iterators::Pair<Rule>) -> Statement {
    let mut exprs = Vec::new();
    let mut target = String::new();
    for inner in pair.into_inner() {
        match inner.as_rule() {
            Rule::expr => exprs.push(parse_expr(inner)),
            Rule::ident => target = inner.as_str().to_string(),
            _ => {}
        }
    }
    Statement::Add {
        source: exprs.into_iter().next().unwrap_or(Expr::NumericLiteral(0.0)),
        target,
    }
}

fn parse_subtract_stmt(pair: pest::iterators::Pair<Rule>) -> Statement {
    let mut exprs = Vec::new();
    let mut target = String::new();
    for inner in pair.into_inner() {
        match inner.as_rule() {
            Rule::expr => exprs.push(parse_expr(inner)),
            Rule::ident => target = inner.as_str().to_string(),
            _ => {}
        }
    }
    Statement::Subtract {
        source: exprs.into_iter().next().unwrap_or(Expr::NumericLiteral(0.0)),
        target,
    }
}

fn parse_multiply_stmt(pair: pest::iterators::Pair<Rule>) -> Statement {
    let mut exprs = Vec::new();
    let mut target = String::new();
    for inner in pair.into_inner() {
        match inner.as_rule() {
            Rule::expr => exprs.push(parse_expr(inner)),
            Rule::ident => target = inner.as_str().to_string(),
            _ => {}
        }
    }
    Statement::Multiply {
        source: exprs.into_iter().next().unwrap_or(Expr::NumericLiteral(1.0)),
        target,
    }
}

fn parse_divide_stmt(pair: pest::iterators::Pair<Rule>) -> Statement {
    let mut exprs = Vec::new();
    let mut target = String::new();
    for inner in pair.into_inner() {
        match inner.as_rule() {
            Rule::expr => exprs.push(parse_expr(inner)),
            Rule::ident => target = inner.as_str().to_string(),
            _ => {}
        }
    }
    Statement::Divide {
        source: exprs.into_iter().next().unwrap_or(Expr::NumericLiteral(1.0)),
        target,
    }
}

fn parse_display_stmt(pair: pest::iterators::Pair<Rule>) -> Statement {
    let mut values = Vec::new();
    for inner in pair.into_inner() {
        if inner.as_rule() == Rule::expr {
            values.push(parse_expr(inner));
        }
    }
    Statement::Display { values }
}

fn parse_perform_stmt(pair: pest::iterators::Pair<Rule>) -> Statement {
    let mut paragraph = String::new();
    for inner in pair.into_inner() {
        if inner.as_rule() == Rule::ident {
            paragraph = inner.as_str().to_string();
        }
    }
    Statement::Perform { paragraph }
}

fn parse_if_stmt(pair: pest::iterators::Pair<Rule>) -> Statement {
    let mut condition = Condition::ConditionName(String::new());
    let mut then_body = Vec::new();
    let mut else_body = Vec::new();
    let mut in_else = false;

    for inner in pair.into_inner() {
        match inner.as_rule() {
            Rule::condition => {
                condition = parse_condition(inner);
            }
            Rule::statement => {
                if let Some(stmt) = parse_statement(inner) {
                    if in_else {
                        else_body.push(stmt);
                    } else {
                        then_body.push(stmt);
                    }
                }
            }
            Rule::else_clause => {
                in_else = true;
                for else_inner in inner.into_inner() {
                    if else_inner.as_rule() == Rule::statement {
                        if let Some(stmt) = parse_statement(else_inner) {
                            else_body.push(stmt);
                        }
                    }
                }
            }
            _ => {}
        }
    }

    Statement::If {
        condition,
        then_body,
        else_body,
    }
}

fn parse_condition(pair: pest::iterators::Pair<Rule>) -> Condition {
    let mut left = Expr::Literal(String::new());
    let mut op = CompareOp::Equal;
    let mut right = Expr::Literal(String::new());
    let mut idx = 0;

    for inner in pair.into_inner() {
        match inner.as_rule() {
            Rule::expr => {
                if idx == 0 {
                    left = parse_expr(inner);
                } else {
                    right = parse_expr(inner);
                }
                idx += 1;
            }
            Rule::compare_op => {
                let op_str = inner.as_str().trim();
                op = match op_str {
                    "=" => CompareOp::Equal,
                    ">" => CompareOp::GreaterThan,
                    "<" => CompareOp::LessThan,
                    ">=" => CompareOp::GreaterOrEqual,
                    "<=" => CompareOp::LessOrEqual,
                    _ if op_str.contains("NOT") => CompareOp::NotEqual,
                    _ => CompareOp::Equal,
                };
            }
            _ => {}
        }
    }

    Condition::Compare { left, op, right }
}

fn parse_string_stmt(pair: pest::iterators::Pair<Rule>) -> Statement {
    let mut sources = Vec::new();
    let mut into = String::new();

    for inner in pair.into_inner() {
        match inner.as_rule() {
            Rule::string_source => {
                let mut src_expr = Expr::Literal(String::new());
                let mut delim_expr = Expr::Literal("SIZE".to_string());
                for s in inner.into_inner() {
                    match s.as_rule() {
                        Rule::expr => src_expr = parse_expr(s),
                        Rule::string_delim => {
                            let delim_text = s.as_str().trim();
                            delim_expr = match delim_text {
                                "SIZE" => Expr::Literal("SIZE".to_string()),
                                "SPACE" | "SPACES" => Expr::Literal("SPACE".to_string()),
                                _ => {
                                    // Quoted string delimiter
                                    if delim_text.starts_with('"') {
                                        Expr::Literal(
                                            delim_text[1..delim_text.len() - 1].to_string(),
                                        )
                                    } else {
                                        Expr::Literal(delim_text.to_string())
                                    }
                                }
                            };
                        }
                        _ => {}
                    }
                }
                sources.push((src_expr, delim_expr));
            }
            Rule::ident => {
                into = inner.as_str().to_string();
            }
            _ => {}
        }
    }

    Statement::StringConcat { sources, into }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_minimal_screen() {
        let source = r#"
IDENTIFICATION DIVISION.
PROGRAM-ID. HELLO.

DATA DIVISION.
SCREEN SECTION.
01 MAIN-SCREEN.
   05 HEADER.
      10 TITLE PIC X(20) VALUE "Hello World".
"#;

        let app = parse(source).expect("should parse");
        assert_eq!(app.screens.len(), 1);
        assert_eq!(app.screens[0].name, "MAIN-SCREEN");

        // Root should be a container with one child (HEADER)
        if let Node::Container { ref children, .. } = app.screens[0].root {
            assert_eq!(children.len(), 1);
            if let Node::Container {
                ref name,
                ref children,
                ..
            } = children[0]
            {
                assert_eq!(name, "HEADER");
                assert_eq!(children.len(), 1);
            } else {
                panic!("expected HEADER container");
            }
        } else {
            panic!("expected root container");
        }
    }

    #[test]
    fn test_parse_working_storage() {
        let source = r#"
IDENTIFICATION DIVISION.
PROGRAM-ID. TEST-APP.

DATA DIVISION.
WORKING-STORAGE SECTION.
01 APP-STATE.
   05 USER-NAME PIC X(40) VALUE "ANON".
   05 COUNTER PIC 9(4) VALUE 0.
"#;

        let app = parse(source).expect("should parse");
        assert!(app.state.contains_key("USER-NAME"));
        assert!(app.state.contains_key("COUNTER"));

        let counter = &app.state["COUNTER"];
        assert_eq!(counter.pic.kind, PicKind::Numeric);
        assert_eq!(counter.pic.width, 4);
    }

    #[test]
    fn test_parse_go_to_screen() {
        let source = r#"
DATA DIVISION.
SCREEN SECTION.
01 MAIN-SCREEN.
   05 CONTROLS.
      10 NEXT-BTN VALUE "Next" GO-TO-SCREEN SETTINGS-SCREEN.
01 SETTINGS-SCREEN.
   05 CONTROLS.
      10 BACK-BTN VALUE "Back" GO-TO-SCREEN MAIN-SCREEN.
"#;
        let app = parse(source).expect("should parse");
        assert_eq!(app.screens.len(), 2);
        assert_eq!(app.screens[0].name, "MAIN-SCREEN");
        assert_eq!(app.screens[1].name, "SETTINGS-SCREEN");

        // Check navigation target on first screen's button
        if let Node::Container { ref children, .. } = app.screens[0].root {
            if let Node::Container { ref children, .. } = children[0] {
                if let Node::Button {
                    ref navigate,
                    ref label,
                    ..
                } = children[0]
                {
                    assert_eq!(label, "NEXT");
                    assert_eq!(navigate.as_deref(), Some("SETTINGS-SCREEN"));
                } else {
                    panic!("expected button node");
                }
            }
        }

        // Check navigation target on second screen's button
        if let Node::Container { ref children, .. } = app.screens[1].root {
            if let Node::Container { ref children, .. } = children[0] {
                if let Node::Button {
                    ref navigate,
                    ref label,
                    ..
                } = children[0]
                {
                    assert_eq!(label, "BACK");
                    assert_eq!(navigate.as_deref(), Some("MAIN-SCREEN"));
                } else {
                    panic!("expected button node");
                }
            }
        }
    }

    #[test]
    fn test_parse_counter_paragraphs() {
        let source = r#"
IDENTIFICATION DIVISION.
PROGRAM-ID. COUNTER.

DATA DIVISION.
WORKING-STORAGE SECTION.
01 APP-STATE.
   05 COUNTER-VAL PIC 9(4) VALUE 0.

SCREEN SECTION.
01 MAIN-SCREEN.
   05 CONTROLS.
      10 INC-BTN VALUE "+" ON-ACTION PERFORM HANDLE-INCREMENT.

PROCEDURE DIVISION.
MAIN-LOOP.
    STOP RUN.

HANDLE-INCREMENT.
    ADD 1 TO COUNTER-VAL.

HANDLE-DECREMENT.
    SUBTRACT 1 FROM COUNTER-VAL.

HANDLE-RESET.
    MOVE 0 TO COUNTER-VAL.
"#;
        let app = parse(source).expect("should parse");
        assert_eq!(app.paragraphs.len(), 4);

        // Check HANDLE-INCREMENT has ADD statement
        let inc = &app.paragraphs["HANDLE-INCREMENT"];
        assert_eq!(inc.statements.len(), 1);
        match &inc.statements[0] {
            Statement::Add { source, target } => {
                assert!(matches!(source, Expr::NumericLiteral(n) if *n == 1.0));
                assert_eq!(target, "COUNTER-VAL");
            }
            other => panic!("expected Add, got {:?}", other),
        }

        // Check HANDLE-DECREMENT has SUBTRACT statement
        let dec = &app.paragraphs["HANDLE-DECREMENT"];
        assert_eq!(dec.statements.len(), 1);
        assert!(matches!(&dec.statements[0], Statement::Subtract { .. }));

        // Check HANDLE-RESET has MOVE statement
        let reset = &app.paragraphs["HANDLE-RESET"];
        assert_eq!(reset.statements.len(), 1);
        assert!(matches!(&reset.statements[0], Statement::Move { .. }));
    }

    #[test]
    fn test_parse_if_else() {
        let source = r#"
PROCEDURE DIVISION.
CHECK-VAL.
    IF COUNTER-VAL > 10
        MOVE "BIG" TO STATUS-MSG
    ELSE
        MOVE "SMALL" TO STATUS-MSG
    END-IF.
"#;
        let app = parse(source).expect("should parse");
        let check = &app.paragraphs["CHECK-VAL"];
        assert_eq!(check.statements.len(), 1);
        match &check.statements[0] {
            Statement::If {
                condition,
                then_body,
                else_body,
            } => {
                assert!(matches!(
                    condition,
                    Condition::Compare {
                        op: CompareOp::GreaterThan,
                        ..
                    }
                ));
                assert_eq!(then_body.len(), 1);
                assert_eq!(else_body.len(), 1);
            }
            other => panic!("expected If, got {:?}", other),
        }
    }

    #[test]
    fn test_parse_string_stmt() {
        let source = r#"
PROCEDURE DIVISION.
GREET.
    STRING "HELLO, " DELIMITED BY SIZE
           USER-NAME DELIMITED BY SIZE
           "!" DELIMITED BY SIZE
    INTO STATUS-MSG.
"#;
        let app = parse(source).expect("should parse");
        let greet = &app.paragraphs["GREET"];
        assert_eq!(greet.statements.len(), 1);
        match &greet.statements[0] {
            Statement::StringConcat { sources, into } => {
                assert_eq!(sources.len(), 3);
                assert_eq!(into, "STATUS-MSG");
            }
            other => panic!("expected StringConcat, got {:?}", other),
        }
    }

    #[test]
    fn test_parse_display_stmt() {
        let source = r#"
PROCEDURE DIVISION.
SHOW.
    DISPLAY "Hello!".
"#;
        let app = parse(source).expect("should parse");
        let show = &app.paragraphs["SHOW"];
        assert_eq!(show.statements.len(), 1);
        assert!(matches!(&show.statements[0], Statement::Display { values } if values.len() == 1));
    }

    #[test]
    fn test_parse_perform_stmt() {
        let source = r#"
PROCEDURE DIVISION.
MAIN.
    PERFORM HELPER.
HELPER.
    MOVE 0 TO X.
"#;
        let app = parse(source).expect("should parse");
        let main = &app.paragraphs["MAIN"];
        assert_eq!(main.statements.len(), 1);
        match &main.statements[0] {
            Statement::Perform { paragraph } => assert_eq!(paragraph, "HELPER"),
            other => panic!("expected Perform, got {:?}", other),
        }
    }

    #[test]
    fn test_parse_button_with_action() {
        let source = r#"
DATA DIVISION.
SCREEN SECTION.
01 MAIN-SCREEN.
   05 CONTROLS.
      10 SUBMIT-BTN VALUE "Go" ON-ACTION PERFORM HANDLE-SUBMIT.
"#;

        let app = parse(source).expect("should parse");
        assert_eq!(app.screens.len(), 1);

        if let Node::Container { ref children, .. } = app.screens[0].root {
            if let Node::Container { ref children, .. } = children[0] {
                assert_eq!(children.len(), 1);
                if let Node::Button {
                    ref label,
                    ref action,
                    ..
                } = children[0]
                {
                    assert_eq!(label, "GO");
                    assert_eq!(action.as_deref(), Some("HANDLE-SUBMIT"));
                } else {
                    panic!("expected button node");
                }
            }
        }
    }
}
