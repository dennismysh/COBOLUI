//! COBALT Terminal Renderer
//!
//! Renders COBALT IR to a terminal UI using ratatui (crossterm backend).
//! Also contains the renderer trait, event loop, and COBOL statement interpreter
//! (previously in cobalt-render).

use cobalt_ir::{
    AcceptSource, ArithExpr, ArithOp, CobaltApp, CompareOp, Condition, EventRecord, Expr, Node,
    RuntimeState, Screen, StateMap, Statement,
};
use cobalt_ir::Paragraph as CobaltParagraph;
use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style as RatStyle},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame, Terminal,
};
use std::collections::HashMap;
use std::io::{self, stdout};

// ---------------------------------------------------------------------------
// Renderer Trait & Error Types
// ---------------------------------------------------------------------------

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
    fn init(&mut self, app: &CobaltApp) -> Result<()>;
    fn render(&mut self, screen: &Screen, state: &RuntimeState) -> Result<()>;
    fn poll_event(&mut self) -> Result<Option<EventRecord>>;
    fn shutdown(&mut self) -> Result<()>;
    fn rebuild_focus(&mut self, _screen: &Screen) -> Result<()> {
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// Event Loop
// ---------------------------------------------------------------------------

/// Run the main event loop: init, render, poll, repeat until exit.
pub fn run_app(renderer: &mut dyn Renderer, app: &CobaltApp) -> Result<()> {
    renderer.init(app)?;

    if app.screens.is_empty() {
        renderer.shutdown()?;
        return Ok(());
    }

    let mut current_screen: usize = 0;
    let mut state = cobalt_ir::RuntimeState::new();

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
                        if !event.payload.is_empty() {
                            execute_paragraph(&event.payload, &app.paragraphs, &app.state, &mut state, 0);
                        }
                    }
                    "NAVIGATE" => {
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

const MAX_LOOP_ITERATIONS: usize = 10_000;

/// Execute all statements in a named paragraph.
pub fn execute_paragraph(
    name: &str,
    paragraphs: &HashMap<String, CobaltParagraph>,
    state_defs: &StateMap,
    state: &mut RuntimeState,
    depth: usize,
) {
    if depth > MAX_RECURSION_DEPTH {
        return;
    }
    if let Some(para) = paragraphs.get(name) {
        for stmt in &para.statements {
            execute_statement(stmt, paragraphs, state_defs, state, depth);
        }
    }
}

fn execute_statement(
    stmt: &Statement,
    paragraphs: &HashMap<String, CobaltParagraph>,
    state_defs: &StateMap,
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
            state.insert("STATUS-MSG".to_string(), msg);
        }
        Statement::If {
            condition,
            then_body,
            else_body,
        } => {
            if eval_condition(condition, state_defs, state) {
                for s in then_body {
                    execute_statement(s, paragraphs, state_defs, state, depth);
                }
            } else {
                for s in else_body {
                    execute_statement(s, paragraphs, state_defs, state, depth);
                }
            }
        }
        Statement::Perform { paragraph } => {
            execute_paragraph(paragraph, paragraphs, state_defs, state, depth + 1);
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
        Statement::Evaluate {
            subject,
            whens,
            other,
        } => {
            let subject_val = eval_expr(subject, state);
            let mut matched = false;
            for when in whens {
                let when_val = eval_expr(&when.value, state);
                if subject_val == when_val {
                    for s in &when.body {
                        execute_statement(s, paragraphs, state_defs, state, depth);
                    }
                    matched = true;
                    break;
                }
            }
            if !matched {
                for s in other {
                    execute_statement(s, paragraphs, state_defs, state, depth);
                }
            }
        }
        Statement::PerformUntil {
            paragraph,
            condition,
        } => {
            let mut iterations = 0;
            while !eval_condition(condition, state_defs, state) && iterations < MAX_LOOP_ITERATIONS {
                execute_paragraph(paragraph, paragraphs, state_defs, state, depth + 1);
                iterations += 1;
            }
        }
        Statement::Compute { target, expression } => {
            let result = eval_arith_expr(expression, state);
            state.insert(target.clone(), format_numeric(result));
        }
        Statement::Accept { target, source } => {
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs();
            // Simple date/time formatting without external deps
            let val = match source {
                AcceptSource::Date => {
                    // YYYYMMDD format
                    let days = now / 86400;
                    let (y, m, d) = days_to_ymd(days);
                    format!("{:04}{:02}{:02}", y, m, d)
                }
                AcceptSource::Time => {
                    // HHMMSS format
                    let secs_today = now % 86400;
                    let h = secs_today / 3600;
                    let m = (secs_today % 3600) / 60;
                    let s = secs_today % 60;
                    format!("{:02}{:02}{:02}", h, m, s)
                }
                AcceptSource::DayOfWeek => {
                    // 1=Monday .. 7=Sunday
                    let days = now / 86400;
                    let dow = ((days + 3) % 7) + 1; // epoch was Thursday
                    format!("{}", dow)
                }
            };
            state.insert(target.clone(), val);
        }
        Statement::Set { condition, value } => {
            // Find which field owns this condition name and set it accordingly
            for (_field_name, field_def) in state_defs.iter() {
                for (cond_name, cond_value) in &field_def.conditions {
                    if cond_name == condition {
                        if *value {
                            state.insert(field_def.name.clone(), cond_value.clone());
                        }
                        return;
                    }
                }
            }
        }
        Statement::StopRun => {}
    }
}

fn eval_arith_expr(expr: &ArithExpr, state: &RuntimeState) -> f64 {
    match expr {
        ArithExpr::Num(n) => *n,
        ArithExpr::Var(name) => state
            .get(name)
            .and_then(|v| v.parse::<f64>().ok())
            .unwrap_or(0.0),
        ArithExpr::BinOp { left, op, right } => {
            let l = eval_arith_expr(left, state);
            let r = eval_arith_expr(right, state);
            match op {
                ArithOp::Add => l + r,
                ArithOp::Subtract => l - r,
                ArithOp::Multiply => l * r,
                ArithOp::Divide => {
                    if r != 0.0 {
                        l / r
                    } else {
                        0.0
                    }
                }
            }
        }
    }
}

/// Convert days since epoch to (year, month, day).
fn days_to_ymd(days_since_epoch: u64) -> (u64, u64, u64) {
    // Algorithm from http://howardhinnant.github.io/date_algorithms.html
    let z = days_since_epoch + 719468;
    let era = z / 146097;
    let doe = z - era * 146097;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if m <= 2 { y + 1 } else { y };
    (y, m, d)
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

fn eval_condition(condition: &Condition, state_defs: &StateMap, state: &RuntimeState) -> bool {
    match condition {
        Condition::Compare { left, op, right } => {
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
        Condition::ConditionName(name) => {
            // Level-88: find the parent field, compare its current value to the condition value
            for (_field_name, field_def) in state_defs.iter() {
                for (cond_name, cond_value) in &field_def.conditions {
                    if cond_name == name {
                        let current = state.get(&field_def.name).cloned().unwrap_or_default();
                        return current == *cond_value;
                    }
                }
            }
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

// ---------------------------------------------------------------------------
// Terminal Renderer Implementation
// ---------------------------------------------------------------------------

/// Map COBALT color index to crossterm color.
fn map_color(index: Option<u8>) -> Color {
    match index {
        Some(0) => Color::Black,
        Some(1) => Color::Blue,
        Some(2) => Color::White,
        Some(3) => Color::Gray,
        Some(4) => Color::DarkGray,
        Some(5) => Color::Cyan,
        Some(6) => Color::Green,
        Some(7) => Color::Red,
        _ => Color::Reset,
    }
}

struct FocusState {
    elements: Vec<FocusableElement>,
    index: usize,
}

#[derive(Clone)]
struct FocusableElement {
    name: String,
    kind: FocusKind,
    binding: Option<String>,
    action: Option<String>,
    navigate: Option<String>,
}

#[derive(Clone, PartialEq)]
enum FocusKind {
    TextInput,
    NumericInput,
    Button,
}

impl FocusState {
    fn new() -> Self {
        Self {
            elements: Vec::new(),
            index: 0,
        }
    }

    fn current(&self) -> Option<&FocusableElement> {
        self.elements.get(self.index)
    }

    fn next(&mut self) {
        if !self.elements.is_empty() {
            self.index = (self.index + 1) % self.elements.len();
        }
    }

    fn prev(&mut self) {
        if !self.elements.is_empty() {
            self.index = if self.index == 0 {
                self.elements.len() - 1
            } else {
                self.index - 1
            };
        }
    }
}

fn collect_focusable(node: &Node, elements: &mut Vec<FocusableElement>) {
    match node {
        Node::Container { children, .. } => {
            for child in children {
                collect_focusable(child, elements);
            }
        }
        Node::Text {
            name, binding, ..
        } => {
            elements.push(FocusableElement {
                name: name.clone(),
                kind: FocusKind::TextInput,
                binding: binding.clone(),
                action: None,
                navigate: None,
            });
        }
        Node::Numeric {
            name, binding, ..
        } => {
            elements.push(FocusableElement {
                name: name.clone(),
                kind: FocusKind::NumericInput,
                binding: binding.clone(),
                action: None,
                navigate: None,
            });
        }
        Node::Button {
            name,
            action,
            navigate,
            ..
        } => {
            elements.push(FocusableElement {
                name: name.clone(),
                kind: FocusKind::Button,
                binding: None,
                action: action.clone(),
                navigate: navigate.clone(),
            });
        }
    }
}

pub struct TermRenderer {
    terminal: Option<Terminal<ratatui::backend::CrosstermBackend<io::Stdout>>>,
    focus: FocusState,
}

impl TermRenderer {
    pub fn new() -> Self {
        Self {
            terminal: None,
            focus: FocusState::new(),
        }
    }
}

impl Default for TermRenderer {
    fn default() -> Self {
        Self::new()
    }
}

impl Renderer for TermRenderer {
    fn init(&mut self, app: &CobaltApp) -> Result<()> {
        enable_raw_mode().map_err(RenderError::Io)?;
        stdout()
            .execute(EnterAlternateScreen)
            .map_err(RenderError::Io)?;
        let backend = ratatui::backend::CrosstermBackend::new(stdout());
        self.terminal =
            Some(Terminal::new(backend).map_err(RenderError::Io)?);

        if let Some(screen) = app.screens.first() {
            let mut elements = Vec::new();
            collect_focusable(&screen.root, &mut elements);
            self.focus = FocusState {
                elements,
                index: 0,
            };
        }

        Ok(())
    }

    fn render(&mut self, screen: &Screen, state: &RuntimeState) -> Result<()> {
        let terminal = self
            .terminal
            .as_mut()
            .ok_or_else(|| RenderError::RenderFailed("not initialized".into()))?;

        let focus = &self.focus;

        terminal
            .draw(|frame| {
                let area = frame.area();
                render_node(frame, &screen.root, area, state, focus);
            })
            .map_err(RenderError::Io)?;

        Ok(())
    }

    fn poll_event(&mut self) -> Result<Option<EventRecord>> {
        loop {
            let ev = event::read().map_err(RenderError::Io)?;
            match ev {
                Event::Key(key) if key.kind == KeyEventKind::Press => match key.code {
                    KeyCode::Esc => {
                        return Ok(None);
                    }
                    KeyCode::Tab => {
                        self.focus.next();
                        return Ok(Some(EventRecord {
                            event_type: "FOCUS".into(),
                            target: self.focus.current().map_or(String::new(), |e| e.name.clone()),
                            payload: String::new(),
                        }));
                    }
                    KeyCode::BackTab => {
                        self.focus.prev();
                        return Ok(Some(EventRecord {
                            event_type: "FOCUS".into(),
                            target: self.focus.current().map_or(String::new(), |e| e.name.clone()),
                            payload: String::new(),
                        }));
                    }
                    KeyCode::Enter => {
                        if let Some(el) = self.focus.current() {
                            if el.kind == FocusKind::Button {
                                if let Some(ref target) = el.navigate {
                                    return Ok(Some(EventRecord {
                                        event_type: "NAVIGATE".into(),
                                        target: el.name.clone(),
                                        payload: target.clone(),
                                    }));
                                }
                                return Ok(Some(EventRecord {
                                    event_type: "CLICK".into(),
                                    target: el.name.clone(),
                                    payload: el
                                        .action
                                        .clone()
                                        .unwrap_or_default(),
                                }));
                            }
                        }
                        return Ok(Some(EventRecord {
                            event_type: "INPUT".into(),
                            target: String::new(),
                            payload: String::new(),
                        }));
                    }
                    KeyCode::Char(c) => {
                        if let Some(el) = self.focus.current() {
                            let binding = el.binding.clone().unwrap_or(el.name.clone());
                            if el.kind == FocusKind::NumericInput && !c.is_ascii_digit() {
                                continue;
                            }
                            return Ok(Some(EventRecord {
                                event_type: "INPUT".into(),
                                target: binding,
                                payload: c.to_string(),
                            }));
                        }
                    }
                    KeyCode::Backspace => {
                        if let Some(el) = self.focus.current() {
                            let binding = el.binding.clone().unwrap_or(el.name.clone());
                            return Ok(Some(EventRecord {
                                event_type: "INPUT".into(),
                                target: binding,
                                payload: "\x08".to_string(),
                            }));
                        }
                    }
                    _ => {}
                },
                Event::Resize(_, _) => {
                    return Ok(Some(EventRecord {
                        event_type: "RESIZE".into(),
                        target: String::new(),
                        payload: String::new(),
                    }));
                }
                _ => {}
            }
        }
    }

    fn rebuild_focus(&mut self, screen: &Screen) -> Result<()> {
        let mut elements = Vec::new();
        collect_focusable(&screen.root, &mut elements);
        self.focus = FocusState {
            elements,
            index: 0,
        };
        Ok(())
    }

    fn shutdown(&mut self) -> Result<()> {
        disable_raw_mode().map_err(RenderError::Io)?;
        stdout()
            .execute(LeaveAlternateScreen)
            .map_err(RenderError::Io)?;
        Ok(())
    }
}

fn render_node(
    frame: &mut Frame,
    node: &Node,
    area: Rect,
    state: &RuntimeState,
    focus: &FocusState,
) {
    match node {
        Node::Container {
            name,
            children,
            style,
        } => {
            let block = Block::default()
                .title(format!(" {} ", name))
                .borders(Borders::ALL)
                .border_style(RatStyle::default().fg(map_color(style.fg_color)))
                .style(RatStyle::default().bg(map_color(style.bg_color)));

            let inner = block.inner(area);
            frame.render_widget(block, area);

            if children.is_empty() {
                return;
            }

            let constraints: Vec<Constraint> = children
                .iter()
                .map(|child| match child {
                    Node::Container { .. } => Constraint::Min(3),
                    _ => Constraint::Length(1),
                })
                .collect();

            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints(constraints)
                .split(inner);

            for (i, child) in children.iter().enumerate() {
                if i < chunks.len() {
                    render_node(frame, child, chunks[i], state, focus);
                }
            }
        }
        Node::Text {
            name,
            pic,
            value,
            binding,
            style,
        } => {
            let display_value = binding
                .as_ref()
                .and_then(|b| state.get(b))
                .or(value.as_ref())
                .cloned()
                .unwrap_or_default();

            let is_focused = focus.current().map_or(false, |f| f.name == *name);

            let label = format!(
                "{}: [{}]{}",
                name,
                format!("{:<width$}", display_value, width = pic.width),
                if is_focused { " <" } else { "" }
            );

            let mut rat_style = RatStyle::default()
                .fg(map_color(style.fg_color))
                .bg(map_color(style.bg_color));

            if is_focused {
                rat_style = rat_style.add_modifier(Modifier::REVERSED);
            }

            let paragraph = Paragraph::new(Line::from(Span::styled(label, rat_style)));
            frame.render_widget(paragraph, area);
        }
        Node::Numeric {
            name,
            pic,
            value,
            binding,
            style,
        } => {
            let display_value = binding
                .as_ref()
                .and_then(|b| state.get(b))
                .or(value.as_ref())
                .cloned()
                .unwrap_or_else(|| "0".to_string());

            let is_focused = focus.current().map_or(false, |f| f.name == *name);

            let label = format!(
                "{}: [{}]{}",
                name,
                format!("{:>width$}", display_value, width = pic.width),
                if is_focused { " <" } else { "" }
            );

            let mut rat_style = RatStyle::default()
                .fg(map_color(style.fg_color))
                .bg(map_color(style.bg_color));

            if is_focused {
                rat_style = rat_style.add_modifier(Modifier::REVERSED);
            }

            let paragraph = Paragraph::new(Line::from(Span::styled(label, rat_style)));
            frame.render_widget(paragraph, area);
        }
        Node::Button {
            name,
            label,
            style,
            ..
        } => {
            let is_focused = focus.current().map_or(false, |f| f.name == *name);

            let display = if is_focused {
                format!("[ {} ] <", label)
            } else {
                format!("[ {} ]", label)
            };

            let mut rat_style = RatStyle::default()
                .fg(map_color(style.fg_color))
                .bg(map_color(style.bg_color));

            if is_focused {
                rat_style = rat_style.add_modifier(Modifier::BOLD | Modifier::REVERSED);
            }

            let paragraph = Paragraph::new(Line::from(Span::styled(display, rat_style)));
            frame.render_widget(paragraph, area);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cobalt_ir::{ArithExpr, ArithOp, CompareOp, Condition, Expr, PicClause, PicKind, Paragraph as CobaltParagraph, RuntimeState, StateField, Statement, WhenClause};

    fn make_paragraphs(entries: Vec<(&str, Vec<Statement>)>) -> HashMap<String, CobaltParagraph> {
        entries
            .into_iter()
            .map(|(name, stmts)| {
                (
                    name.to_string(),
                    CobaltParagraph {
                        name: name.to_string(),
                        statements: stmts,
                    },
                )
            })
            .collect()
    }

    fn empty_defs() -> StateMap {
        StateMap::new()
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

        execute_paragraph("HANDLE-RESET", &paragraphs, &empty_defs(), &mut state, 0);
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

        execute_paragraph("HANDLE-INC", &paragraphs, &empty_defs(), &mut state, 0);
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

        execute_paragraph("HANDLE-DEC", &paragraphs, &empty_defs(), &mut state, 0);
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

        execute_paragraph("HANDLE-MUL", &paragraphs, &empty_defs(), &mut state, 0);
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

        execute_paragraph("HANDLE-DIV", &paragraphs, &empty_defs(), &mut state, 0);
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
        execute_paragraph("CHECK", &paragraphs, &empty_defs(), &mut state, 0);
        assert_eq!(state.get("STATUS-MSG").unwrap(), "BIG");

        state.insert("VAL".to_string(), "5".to_string());
        execute_paragraph("CHECK", &paragraphs, &empty_defs(), &mut state, 0);
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

        execute_paragraph("MAIN", &paragraphs, &empty_defs(), &mut state, 0);
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

        execute_paragraph("GREET", &paragraphs, &empty_defs(), &mut state, 0);
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

        execute_paragraph("GREET", &paragraphs, &empty_defs(), &mut state, 0);
        assert_eq!(state.get("STATUS-MSG").unwrap(), "Hello, Alice!");
    }

    #[test]
    fn test_evaluate_when() {
        let paragraphs = make_paragraphs(vec![(
            "CHECK-OP",
            vec![Statement::Evaluate {
                subject: Expr::Variable("OPERATION".to_string()),
                whens: vec![
                    WhenClause {
                        value: Expr::Literal("ADD".to_string()),
                        body: vec![Statement::Move {
                            source: Expr::Literal("Adding".to_string()),
                            target: "STATUS-MSG".to_string(),
                        }],
                    },
                    WhenClause {
                        value: Expr::Literal("SUB".to_string()),
                        body: vec![Statement::Move {
                            source: Expr::Literal("Subtracting".to_string()),
                            target: "STATUS-MSG".to_string(),
                        }],
                    },
                ],
                other: vec![Statement::Move {
                    source: Expr::Literal("Unknown".to_string()),
                    target: "STATUS-MSG".to_string(),
                }],
            }],
        )]);

        let mut state = RuntimeState::new();
        state.insert("OPERATION".to_string(), "ADD".to_string());
        state.insert("STATUS-MSG".to_string(), String::new());
        execute_paragraph("CHECK-OP", &paragraphs, &empty_defs(), &mut state, 0);
        assert_eq!(state.get("STATUS-MSG").unwrap(), "Adding");

        state.insert("OPERATION".to_string(), "MUL".to_string());
        execute_paragraph("CHECK-OP", &paragraphs, &empty_defs(), &mut state, 0);
        assert_eq!(state.get("STATUS-MSG").unwrap(), "Unknown");
    }

    #[test]
    fn test_perform_until() {
        let paragraphs = make_paragraphs(vec![
            (
                "LOOP",
                vec![Statement::PerformUntil {
                    paragraph: "INC".to_string(),
                    condition: Condition::Compare {
                        left: Expr::Variable("COUNTER".to_string()),
                        op: CompareOp::GreaterOrEqual,
                        right: Expr::NumericLiteral(5.0),
                    },
                }],
            ),
            (
                "INC",
                vec![Statement::Add {
                    source: Expr::NumericLiteral(1.0),
                    target: "COUNTER".to_string(),
                }],
            ),
        ]);

        let mut state = RuntimeState::new();
        state.insert("COUNTER".to_string(), "0".to_string());
        execute_paragraph("LOOP", &paragraphs, &empty_defs(), &mut state, 0);
        assert_eq!(state.get("COUNTER").unwrap(), "5");
    }

    #[test]
    fn test_compute() {
        let paragraphs = make_paragraphs(vec![(
            "CALC",
            vec![Statement::Compute {
                target: "RESULT".to_string(),
                expression: ArithExpr::BinOp {
                    left: Box::new(ArithExpr::Var("A".to_string())),
                    op: ArithOp::Add,
                    right: Box::new(ArithExpr::BinOp {
                        left: Box::new(ArithExpr::Var("B".to_string())),
                        op: ArithOp::Multiply,
                        right: Box::new(ArithExpr::Num(2.0)),
                    }),
                },
            }],
        )]);

        let mut state = RuntimeState::new();
        state.insert("A".to_string(), "10".to_string());
        state.insert("B".to_string(), "3".to_string());
        execute_paragraph("CALC", &paragraphs, &empty_defs(), &mut state, 0);
        assert_eq!(state.get("RESULT").unwrap(), "16");
    }

    #[test]
    fn test_level_88_condition() {
        let mut state_defs = StateMap::new();
        state_defs.insert(
            "STATUS".to_string(),
            StateField {
                name: "STATUS".to_string(),
                pic: PicClause {
                    kind: PicKind::Alphanumeric,
                    width: 10,
                    decimals: None,
                },
                default_value: Some("ACTIVE".to_string()),
                conditions: vec![
                    ("IS-ACTIVE".to_string(), "ACTIVE".to_string()),
                    ("IS-CLOSED".to_string(), "CLOSED".to_string()),
                ],
            },
        );

        let paragraphs = make_paragraphs(vec![(
            "CHECK",
            vec![Statement::If {
                condition: Condition::ConditionName("IS-ACTIVE".to_string()),
                then_body: vec![Statement::Move {
                    source: Expr::Literal("YES".to_string()),
                    target: "RESULT".to_string(),
                }],
                else_body: vec![Statement::Move {
                    source: Expr::Literal("NO".to_string()),
                    target: "RESULT".to_string(),
                }],
            }],
        )]);

        let mut state = RuntimeState::new();
        state.insert("STATUS".to_string(), "ACTIVE".to_string());
        state.insert("RESULT".to_string(), String::new());
        execute_paragraph("CHECK", &paragraphs, &state_defs, &mut state, 0);
        assert_eq!(state.get("RESULT").unwrap(), "YES");

        state.insert("STATUS".to_string(), "CLOSED".to_string());
        execute_paragraph("CHECK", &paragraphs, &state_defs, &mut state, 0);
        assert_eq!(state.get("RESULT").unwrap(), "NO");
    }
}
