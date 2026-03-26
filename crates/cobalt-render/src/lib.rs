//! COBALT Renderer Trait
//!
//! Defines the common interface that all rendering backends must implement.
//! Each backend (terminal, web, Android) provides its own implementation.

use cobalt_ir::{
    CobaltApp, CompareOp, Condition, EventRecord, Expr, Paragraph, RuntimeState, Screen, Statement,
};
use std::collections::HashMap;

#[derive(Debug, thiserror::Error)]
pub enum RenderError {
    #[error("Initialization failed: {0}")]
    InitFailed(String),
    #[error("Render failed: {0}")]
    RenderFailed(String),
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
}

pub type Result<T> = std::result::Result<T, RenderError>;

/// The core renderer trait. Each platform backend implements this.
pub trait Renderer {
    /// Initialize the rendering context for the given application.
    fn init(&mut self, app: &CobaltApp) -> Result<()>;

    /// Render a screen with the current runtime state.
    fn render(&mut self, screen: &Screen, state: &RuntimeState) -> Result<()>;

    /// Poll for the next UI event. Returns `None` if the application should exit.
    fn poll_event(&mut self) -> Result<Option<EventRecord>>;

    /// Tear down the rendering context and free resources.
    fn shutdown(&mut self) -> Result<()>;

    /// Rebuild internal focus state for a new screen (called on navigation).
    fn rebuild_focus(&mut self, _screen: &Screen) -> Result<()> {
        Ok(())
    }
}

/// Run the main event loop: init, render, poll, repeat until exit.
pub fn run_app(renderer: &mut dyn Renderer, app: &CobaltApp) -> Result<()> {
    renderer.init(app)?;

    if app.screens.is_empty() {
        renderer.shutdown()?;
        return Ok(());
    }

    let mut current_screen: usize = 0;
    let mut state = cobalt_ir::RuntimeState::new();

    // Initialize state from app defaults
    for (name, field) in &app.state {
        if let Some(ref default) = field.default_value {
            state.insert(name.clone(), default.clone());
        } else {
            state.insert(name.clone(), String::new());
        }
    }

    renderer.render(&app.screens[current_screen], &state)?;

    loop {
        match renderer.poll_event()? {
            Some(event) => {
                match event.event_type.as_str() {
                    "QUIT" => break,
                    "INPUT" => {
                        if !event.target.is_empty() {
                            let current = state.get(&event.target).cloned().unwrap_or_default();
                            if event.payload == "\x08" {
                                // Backspace: remove last character
                                let new_val = if current.is_empty() {
                                    current
                                } else {
                                    current[..current.len() - 1].to_string()
                                };
                                state.insert(event.target.clone(), new_val);
                            } else {
                                state.insert(event.target.clone(), current + &event.payload);
                            }
                        }
                    }
                    "CLICK" => {
                        // Execute the COBOL paragraph associated with this handler
                        if !event.payload.is_empty() {
                            execute_paragraph(&event.payload, &app.paragraphs, &mut state, 0);
                        }
                    }
                    "NAVIGATE" => {
                        // Switch to target screen by name
                        if let Some(idx) =
                            app.screens.iter().position(|s| s.name == event.payload)
                        {
                            current_screen = idx;
                            renderer.rebuild_focus(&app.screens[current_screen])?;
                        }
                    }
                    _ => {}
                }
                renderer.render(&app.screens[current_screen], &state)?;
            }
            None => break,
        }
    }

    renderer.shutdown()?;
    Ok(())
}

// ---------------------------------------------------------------------------
// COBOL Statement Interpreter
// ---------------------------------------------------------------------------

const MAX_RECURSION_DEPTH: usize = 100;

/// Execute all statements in a named paragraph.
pub fn execute_paragraph(
    name: &str,
    paragraphs: &HashMap<String, Paragraph>,
    state: &mut RuntimeState,
    depth: usize,
) {
    if depth > MAX_RECURSION_DEPTH {
        return;
    }
    if let Some(para) = paragraphs.get(name) {
        for stmt in &para.statements {
            execute_statement(stmt, paragraphs, state, depth);
        }
    }
}

fn execute_statement(
    stmt: &Statement,
    paragraphs: &HashMap<String, Paragraph>,
    state: &mut RuntimeState,
    depth: usize,
) {
    match stmt {
        Statement::Move { source, target } => {
            let val = eval_expr(source, state);
            state.insert(target.clone(), val);
        }
        Statement::Add { source, target } => {
            let src = eval_expr_numeric(source, state);
            let cur = state
                .get(target)
                .and_then(|v| v.parse::<f64>().ok())
                .unwrap_or(0.0);
            state.insert(target.clone(), format_numeric(cur + src));
        }
        Statement::Subtract { source, target } => {
            let src = eval_expr_numeric(source, state);
            let cur = state
                .get(target)
                .and_then(|v| v.parse::<f64>().ok())
                .unwrap_or(0.0);
            state.insert(target.clone(), format_numeric(cur - src));
        }
        Statement::Multiply { source, target } => {
            let src = eval_expr_numeric(source, state);
            let cur = state
                .get(target)
                .and_then(|v| v.parse::<f64>().ok())
                .unwrap_or(0.0);
            state.insert(target.clone(), format_numeric(cur * src));
        }
        Statement::Divide { source, target } => {
            let src = eval_expr_numeric(source, state);
            let cur = state
                .get(target)
                .and_then(|v| v.parse::<f64>().ok())
                .unwrap_or(0.0);
            if src != 0.0 {
                state.insert(target.clone(), format_numeric(cur / src));
            }
        }
        Statement::Display { values } => {
            let msg: String = values
                .iter()
                .map(|v| eval_expr(v, state))
                .collect::<Vec<_>>()
                .join(" ");
            // Display writes to STATUS-MSG by convention
            state.insert("STATUS-MSG".to_string(), msg);
        }
        Statement::If {
            condition,
            then_body,
            else_body,
        } => {
            if eval_condition(condition, state) {
                for s in then_body {
                    execute_statement(s, paragraphs, state, depth);
                }
            } else {
                for s in else_body {
                    execute_statement(s, paragraphs, state, depth);
                }
            }
        }
        Statement::Perform { paragraph } => {
            execute_paragraph(paragraph, paragraphs, state, depth + 1);
        }
        Statement::StringConcat { sources, into } => {
            let mut result = String::new();
            for (src_expr, delim_expr) in sources {
                let src_val = eval_expr(src_expr, state);
                let delim = eval_expr(delim_expr, state);
                match delim.as_str() {
                    "SIZE" => result.push_str(&src_val),
                    "SPACE" => {
                        if let Some(pos) = src_val.find(' ') {
                            result.push_str(&src_val[..pos]);
                        } else {
                            result.push_str(&src_val);
                        }
                    }
                    d => {
                        if let Some(pos) = src_val.find(d) {
                            result.push_str(&src_val[..pos]);
                        } else {
                            result.push_str(&src_val);
                        }
                    }
                }
            }
            state.insert(into.clone(), result);
        }
        Statement::StopRun => {
            // In the context of a handler, this is a no-op
        }
    }
}

fn eval_expr(expr: &Expr, state: &RuntimeState) -> String {
    match expr {
        Expr::Literal(s) => s.clone(),
        Expr::NumericLiteral(n) => format_numeric(*n),
        Expr::Variable(name) => state.get(name).cloned().unwrap_or_default(),
    }
}

fn eval_expr_numeric(expr: &Expr, state: &RuntimeState) -> f64 {
    match expr {
        Expr::NumericLiteral(n) => *n,
        Expr::Literal(s) => s.parse::<f64>().unwrap_or(0.0),
        Expr::Variable(name) => state
            .get(name)
            .and_then(|v| v.parse::<f64>().ok())
            .unwrap_or(0.0),
    }
}

fn eval_condition(condition: &Condition, state: &RuntimeState) -> bool {
    match condition {
        Condition::Compare { left, op, right } => {
            // Try numeric comparison first
            let left_num = eval_expr_numeric(left, state);
            let right_num = eval_expr_numeric(right, state);
            let both_numeric = matches!(
                (left, right),
                (Expr::NumericLiteral(_), _)
                    | (_, Expr::NumericLiteral(_))
                    | (Expr::Variable(_), Expr::Variable(_))
            );

            if both_numeric
                && eval_expr(left, state).parse::<f64>().is_ok()
                && eval_expr(right, state).parse::<f64>().is_ok()
            {
                match op {
                    CompareOp::Equal => left_num == right_num,
                    CompareOp::NotEqual => left_num != right_num,
                    CompareOp::GreaterThan => left_num > right_num,
                    CompareOp::LessThan => left_num < right_num,
                    CompareOp::GreaterOrEqual => left_num >= right_num,
                    CompareOp::LessOrEqual => left_num <= right_num,
                }
            } else {
                // String comparison
                let left_str = eval_expr(left, state);
                let right_str = eval_expr(right, state);
                match op {
                    CompareOp::Equal => left_str == right_str,
                    CompareOp::NotEqual => left_str != right_str,
                    CompareOp::GreaterThan => left_str > right_str,
                    CompareOp::LessThan => left_str < right_str,
                    CompareOp::GreaterOrEqual => left_str >= right_str,
                    CompareOp::LessOrEqual => left_str <= right_str,
                }
            }
        }
        Condition::ConditionName(_name) => {
            // Level-88 condition name evaluation - not yet implemented
            false
        }
    }
}

fn format_numeric(n: f64) -> String {
    if n == n.trunc() {
        format!("{}", n as i64)
    } else {
        format!("{}", n)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cobalt_ir::{CompareOp, Condition, Expr, Paragraph, RuntimeState, Statement};

    fn make_paragraphs(entries: Vec<(&str, Vec<Statement>)>) -> HashMap<String, Paragraph> {
        entries
            .into_iter()
            .map(|(name, stmts)| {
                (
                    name.to_string(),
                    Paragraph {
                        name: name.to_string(),
                        statements: stmts,
                    },
                )
            })
            .collect()
    }

    #[test]
    fn test_move_statement() {
        let paragraphs = make_paragraphs(vec![(
            "HANDLE-RESET",
            vec![Statement::Move {
                source: Expr::NumericLiteral(0.0),
                target: "COUNTER-VAL".to_string(),
            }],
        )]);
        let mut state = RuntimeState::new();
        state.insert("COUNTER-VAL".to_string(), "42".to_string());

        execute_paragraph("HANDLE-RESET", &paragraphs, &mut state, 0);
        assert_eq!(state.get("COUNTER-VAL").unwrap(), "0");
    }

    #[test]
    fn test_add_statement() {
        let paragraphs = make_paragraphs(vec![(
            "HANDLE-INC",
            vec![Statement::Add {
                source: Expr::NumericLiteral(1.0),
                target: "COUNTER-VAL".to_string(),
            }],
        )]);
        let mut state = RuntimeState::new();
        state.insert("COUNTER-VAL".to_string(), "5".to_string());

        execute_paragraph("HANDLE-INC", &paragraphs, &mut state, 0);
        assert_eq!(state.get("COUNTER-VAL").unwrap(), "6");
    }

    #[test]
    fn test_subtract_statement() {
        let paragraphs = make_paragraphs(vec![(
            "HANDLE-DEC",
            vec![Statement::Subtract {
                source: Expr::NumericLiteral(1.0),
                target: "COUNTER-VAL".to_string(),
            }],
        )]);
        let mut state = RuntimeState::new();
        state.insert("COUNTER-VAL".to_string(), "5".to_string());

        execute_paragraph("HANDLE-DEC", &paragraphs, &mut state, 0);
        assert_eq!(state.get("COUNTER-VAL").unwrap(), "4");
    }

    #[test]
    fn test_multiply_statement() {
        let paragraphs = make_paragraphs(vec![(
            "HANDLE-MUL",
            vec![Statement::Multiply {
                source: Expr::NumericLiteral(3.0),
                target: "VAL".to_string(),
            }],
        )]);
        let mut state = RuntimeState::new();
        state.insert("VAL".to_string(), "4".to_string());

        execute_paragraph("HANDLE-MUL", &paragraphs, &mut state, 0);
        assert_eq!(state.get("VAL").unwrap(), "12");
    }

    #[test]
    fn test_divide_statement() {
        let paragraphs = make_paragraphs(vec![(
            "HANDLE-DIV",
            vec![Statement::Divide {
                source: Expr::NumericLiteral(2.0),
                target: "VAL".to_string(),
            }],
        )]);
        let mut state = RuntimeState::new();
        state.insert("VAL".to_string(), "10".to_string());

        execute_paragraph("HANDLE-DIV", &paragraphs, &mut state, 0);
        assert_eq!(state.get("VAL").unwrap(), "5");
    }

    #[test]
    fn test_if_then_else() {
        let paragraphs = make_paragraphs(vec![(
            "CHECK",
            vec![Statement::If {
                condition: Condition::Compare {
                    left: Expr::Variable("VAL".to_string()),
                    op: CompareOp::GreaterThan,
                    right: Expr::NumericLiteral(10.0),
                },
                then_body: vec![Statement::Move {
                    source: Expr::Literal("BIG".to_string()),
                    target: "STATUS-MSG".to_string(),
                }],
                else_body: vec![Statement::Move {
                    source: Expr::Literal("SMALL".to_string()),
                    target: "STATUS-MSG".to_string(),
                }],
            }],
        )]);

        let mut state = RuntimeState::new();
        state.insert("VAL".to_string(), "15".to_string());
        state.insert("STATUS-MSG".to_string(), String::new());
        execute_paragraph("CHECK", &paragraphs, &mut state, 0);
        assert_eq!(state.get("STATUS-MSG").unwrap(), "BIG");

        state.insert("VAL".to_string(), "5".to_string());
        execute_paragraph("CHECK", &paragraphs, &mut state, 0);
        assert_eq!(state.get("STATUS-MSG").unwrap(), "SMALL");
    }

    #[test]
    fn test_perform_calls_paragraph() {
        let paragraphs = make_paragraphs(vec![
            (
                "MAIN",
                vec![Statement::Perform {
                    paragraph: "HELPER".to_string(),
                }],
            ),
            (
                "HELPER",
                vec![Statement::Move {
                    source: Expr::Literal("DONE".to_string()),
                    target: "STATUS-MSG".to_string(),
                }],
            ),
        ]);
        let mut state = RuntimeState::new();
        state.insert("STATUS-MSG".to_string(), String::new());

        execute_paragraph("MAIN", &paragraphs, &mut state, 0);
        assert_eq!(state.get("STATUS-MSG").unwrap(), "DONE");
    }

    #[test]
    fn test_display_sets_status_msg() {
        let paragraphs = make_paragraphs(vec![(
            "GREET",
            vec![Statement::Display {
                values: vec![Expr::Literal("Hello!".to_string())],
            }],
        )]);
        let mut state = RuntimeState::new();

        execute_paragraph("GREET", &paragraphs, &mut state, 0);
        assert_eq!(state.get("STATUS-MSG").unwrap(), "Hello!");
    }

    #[test]
    fn test_string_concat() {
        let paragraphs = make_paragraphs(vec![(
            "GREET",
            vec![Statement::StringConcat {
                sources: vec![
                    (
                        Expr::Literal("Hello, ".to_string()),
                        Expr::Literal("SIZE".to_string()),
                    ),
                    (
                        Expr::Variable("USER-NAME".to_string()),
                        Expr::Literal("SIZE".to_string()),
                    ),
                    (
                        Expr::Literal("!".to_string()),
                        Expr::Literal("SIZE".to_string()),
                    ),
                ],
                into: "STATUS-MSG".to_string(),
            }],
        )]);
        let mut state = RuntimeState::new();
        state.insert("USER-NAME".to_string(), "Alice".to_string());

        execute_paragraph("GREET", &paragraphs, &mut state, 0);
        assert_eq!(state.get("STATUS-MSG").unwrap(), "Hello, Alice!");
    }
}
