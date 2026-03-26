//! COBALT Renderer Trait
//!
//! Defines the common interface that all rendering backends must implement.
//! Each backend (terminal, web, Android) provides its own implementation.

use cobalt_ir::{CobaltApp, EventRecord, RuntimeState, Screen};

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
}

/// Run the main event loop: init, render, poll, repeat until exit.
pub fn run_app(renderer: &mut dyn Renderer, app: &CobaltApp) -> Result<()> {
    renderer.init(app)?;

    if app.screens.is_empty() {
        renderer.shutdown()?;
        return Ok(());
    }

    let screen = &app.screens[0];
    let mut state = cobalt_ir::RuntimeState::new();

    // Initialize state from app defaults
    for (name, field) in &app.state {
        if let Some(ref default) = field.default_value {
            state.insert(name.clone(), default.clone());
        } else {
            state.insert(name.clone(), String::new());
        }
    }

    renderer.render(screen, &state)?;

    loop {
        match renderer.poll_event()? {
            Some(event) => {
                match event.event_type.as_str() {
                    "QUIT" => break,
                    "INPUT" => {
                        // Update state binding
                        if !event.target.is_empty() {
                            state.insert(event.target.clone(), event.payload.clone());
                        }
                    }
                    _ => {}
                }
                renderer.render(screen, &state)?;
            }
            None => break,
        }
    }

    renderer.shutdown()?;
    Ok(())
}
