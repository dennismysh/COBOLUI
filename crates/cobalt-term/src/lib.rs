//! COBALT Terminal Renderer
//!
//! Renders COBALT IR to a terminal UI using ratatui (crossterm backend).

use cobalt_ir::*;
use cobalt_render::{RenderError, Renderer, Result};
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
use std::io::{self, stdout};

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

/// Focus state for interactive elements.
struct FocusState {
    /// List of focusable element names in order.
    elements: Vec<FocusableElement>,
    /// Current focus index.
    index: usize,
}

#[derive(Clone)]
struct FocusableElement {
    name: String,
    kind: FocusKind,
    binding: Option<String>,
    action: Option<String>,
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

/// Collect all focusable elements from the node tree (depth-first).
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
            });
        }
        Node::Button {
            name, action, ..
        } => {
            elements.push(FocusableElement {
                name: name.clone(),
                kind: FocusKind::Button,
                binding: None,
                action: action.clone(),
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

        // Build focus list from first screen
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
                                payload: "\x08".to_string(), // backspace signal
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

    fn shutdown(&mut self) -> Result<()> {
        disable_raw_mode().map_err(RenderError::Io)?;
        stdout()
            .execute(LeaveAlternateScreen)
            .map_err(RenderError::Io)?;
        Ok(())
    }
}

/// Recursively render a node tree into the given frame area.
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

            // Split inner area vertically among children
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
