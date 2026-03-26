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

    for pair in pairs {
        if pair.as_rule() == Rule::program {
            for inner in pair.into_inner() {
                match inner.as_rule() {
                    Rule::data_division => {
                        parse_data_division(inner, &mut state, &mut screens);
                    }
                    Rule::procedure_division => {
                        parse_procedure_division(inner, &mut handlers);
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

fn parse_procedure_division(pair: pest::iterators::Pair<Rule>, handlers: &mut Vec<Handler>) {
    for inner in pair.into_inner() {
        if inner.as_rule() == Rule::paragraph {
            for p in inner.into_inner() {
                if p.as_rule() == Rule::paragraph_name {
                    for name in p.into_inner() {
                        if name.as_rule() == Rule::ident {
                            handlers.push(Handler {
                                name: name.as_str().to_string(),
                                paragraph_name: name.as_str().to_string(),
                            });
                        }
                    }
                }
            }
        }
    }
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
