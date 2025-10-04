//! Interactive TUI for configuring display preferences with live preview

use ansi_to_tui::IntoText;
use anyhow::Result;
use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind, KeyModifiers},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{
    Frame, Terminal,
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, List, ListItem, Padding, Paragraph, Wrap},
};
use std::io;

use crate::config::PersonalityPreferences;
use crate::state::SessionState;
use crate::statusline::{WorkspaceInfo, build_statusline};
use crate::types::Activity;

/// Application state for the interactive config TUI
struct ConfigApp {
    /// Current preferences being edited
    prefs: PersonalityPreferences,
    /// List of all display options with their current states
    options: Vec<(&'static str, bool)>,
    /// Current cursor position
    cursor: usize,
    /// Whether the app should quit
    should_quit: bool,
}

impl ConfigApp {
    fn new(prefs: PersonalityPreferences) -> Self {
        let options = prefs.get_display_options();
        Self {
            prefs,
            options,
            cursor: 0,
            should_quit: false,
        }
    }

    /// Toggle the option at the current cursor position
    fn toggle_current(&mut self) {
        if self.cursor < self.options.len() {
            // Toggle the option
            let (name, _) = self.options[self.cursor];

            // Get current selections
            let mut selections: Vec<&str> = self
                .options
                .iter()
                .filter_map(|(n, enabled)| if *enabled { Some(*n) } else { None })
                .collect();

            // Toggle the selected one
            if selections.contains(&name) {
                selections.retain(|&x| x != name);
            } else {
                selections.push(name);
            }

            // Update preferences
            self.prefs.update_from_selections(&selections);

            // Refresh options list
            self.options = self.prefs.get_display_options();
        } else if self.cursor == self.options.len() {
            // "Done" option selected
            self.should_quit = true;
        }
    }

    fn move_cursor_up(&mut self) {
        if self.cursor > 0 {
            self.cursor -= 1;
        }
    }

    fn move_cursor_down(&mut self) {
        // +1 for "Done" option
        if self.cursor < self.options.len() {
            self.cursor += 1;
        }
    }
}

/// Create a sample session state for preview
fn create_preview_state() -> SessionState {
    SessionState {
        session_id: "preview".to_string(),
        activity: Activity::Coding,
        current_job: None,
        current_file: Some("main.rs".to_string()),
        git_branch: Some("main".to_string()),
        personality: "ლ(╹◡╹ლ) Cowder".to_string(),
        previous_personality: None,
        consecutive_actions: 5,
        error_count: 1,
        recent_activities: vec![Activity::Editing, Activity::Reading],
        mood: crate::state::MoodState::default(),
    }
}

/// Create a sample workspace for preview
fn create_preview_workspace() -> WorkspaceInfo {
    WorkspaceInfo {
        current_dir: Some("/home/user/projects/claude-code-personalities".to_string()),
        project_dir: Some("/home/user/projects/claude-code-personalities".to_string()),
    }
}

/// Run the interactive configuration TUI
pub async fn run_config_ui(prefs: PersonalityPreferences) -> Result<PersonalityPreferences> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create app
    let mut app = ConfigApp::new(prefs);

    // Run event loop
    let result = run_app(&mut terminal, &mut app);

    // Restore terminal
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    result?;

    Ok(app.prefs)
}

/// Main event loop
fn run_app<B: ratatui::backend::Backend>(
    terminal: &mut Terminal<B>,
    app: &mut ConfigApp,
) -> Result<()> {
    loop {
        terminal.draw(|f| ui(f, app))?;

        if app.should_quit {
            break;
        }

        // Handle keyboard events
        if event::poll(std::time::Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    match key.code {
                        KeyCode::Char('c') if key.modifiers == KeyModifiers::CONTROL => {
                            app.should_quit = true;
                        }
                        KeyCode::Char('q') | KeyCode::Esc => {
                            app.should_quit = true;
                        }
                        KeyCode::Up | KeyCode::Char('k') => {
                            app.move_cursor_up();
                        }
                        KeyCode::Down | KeyCode::Char('j') => {
                            app.move_cursor_down();
                        }
                        KeyCode::Char(' ') => {
                            app.toggle_current();
                        }
                        KeyCode::Enter => {
                            if app.cursor == app.options.len() {
                                app.should_quit = true;
                            } else {
                                app.toggle_current();
                            }
                        }
                        _ => {}
                    }
                }
            }
        }
    }

    Ok(())
}

/// Render the UI
fn ui(f: &mut Frame, app: &ConfigApp) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(5), // Preview section (compact)
            Constraint::Min(10),   // Options list
            Constraint::Length(3), // Help text
        ])
        .split(f.area());

    // Render preview
    render_preview(f, chunks[0], app);

    // Render options list
    render_options(f, chunks[1], app);

    // Render help text
    render_help(f, chunks[2]);
}

/// Render the statusline preview
fn render_preview(f: &mut Frame, area: Rect, app: &ConfigApp) {
    let state = create_preview_state();
    let workspace = create_preview_workspace();
    let statusline = build_statusline(&state, "Sonnet", &app.prefs, Some(&workspace));

    // Fancy block with decorative title
    let block = Block::default()
        .title(Line::from(vec![
            Span::styled("✨ ", Style::default().fg(Color::Yellow)),
            Span::styled(
                "Statusline Preview",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(" ✨", Style::default().fg(Color::Yellow)),
        ]))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan))
        .padding(Padding::horizontal(2)); // Horizontal padding only

    // Convert ANSI codes to ratatui Text
    let mut text = statusline
        .as_bytes()
        .to_vec()
        .into_text()
        .unwrap_or_else(|_| {
            // Fallback: if ANSI parsing fails, show raw text
            Text::raw(statusline.clone())
        });

    // Add empty line before to vertically center the single-line statusline
    text.lines.insert(0, Line::from(""));

    let preview_widget = Paragraph::new(text)
        .block(block)
        .alignment(Alignment::Left)
        .wrap(Wrap { trim: false });

    f.render_widget(preview_widget, area);
}

/// Render the options list
fn render_options(f: &mut Frame, area: Rect, app: &ConfigApp) {
    let mut items: Vec<ListItem> = app
        .options
        .iter()
        .enumerate()
        .map(|(idx, (name, enabled))| {
            let visibility = if *enabled { "Visible" } else { "Hidden " };
            let visibility_color = if *enabled { Color::Green } else { Color::Gray };

            let content = Line::from(vec![
                Span::styled("[", Style::default().fg(Color::DarkGray)),
                Span::styled(visibility, Style::default().fg(visibility_color)),
                Span::styled("] ", Style::default().fg(Color::DarkGray)),
                Span::raw(*name),
            ]);

            let style = if idx == app.cursor {
                Style::default()
                    .bg(Color::DarkGray)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };

            ListItem::new(content).style(style)
        })
        .collect();

    // Add "Done" option
    let done_content = Line::from(vec![
        Span::styled("→ ", Style::default().fg(Color::Yellow)),
        Span::raw("Done - Save and Exit"),
    ]);

    let done_style = if app.cursor == app.options.len() {
        Style::default()
            .bg(Color::DarkGray)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default()
    };

    items.push(ListItem::new(done_content).style(done_style));

    let list = List::new(items).block(
        Block::default()
            .title("Configure Display Options")
            .borders(Borders::ALL)
            .style(Style::default().fg(Color::White)),
    );

    f.render_widget(list, area);
}

/// Render help text
fn render_help(f: &mut Frame, area: Rect) {
    let help_text = Paragraph::new("↑↓/jk Navigate • Space Toggle • Enter/q/Ctrl+C Save & Exit")
        .style(Style::default().fg(Color::DarkGray))
        .block(Block::default().borders(Borders::ALL));

    f.render_widget(help_text, area);
}
