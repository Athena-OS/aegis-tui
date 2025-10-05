use std::{env, io};
use std::process::Command;

use log::debug;
use ratatui::crossterm::event::{self, Event};
use ratatui::{
  Terminal,
  crossterm::{
    execute,
    terminal::{
      Clear, ClearType, EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode,
      enable_raw_mode,
    },
  },
  layout::Alignment,
  prelude::CrosstermBackend,
  style::{Color, Modifier, Style},
  text::Line,
  widgets::Paragraph,
};
use std::time::{Duration, Instant};
use tempfile::NamedTempFile;

use crate::installer::{InstallProgress, Installer, Menu, Page, Signal};

pub mod drives;
pub mod installer;
#[macro_use]
pub mod macros;
pub mod widget;

type LineStyle = Option<(Color, Modifier)>;
pub fn styled_block<'a>(lines: Vec<Vec<(LineStyle, impl ToString)>>) -> Vec<Line<'a>> {
  lines
    .into_iter()
    .map(|line| {
      let spans = line
        .into_iter()
        .map(|(style_opt, text)| {
          let mut span = ratatui::text::Span::raw(text.to_string());
          if let Some((color, modifier)) = style_opt {
            span.style = Style::default().fg(color).add_modifier(modifier);
          }
          span
        })
        .collect::<Vec<_>>();
      Line::from(spans)
    })
    .collect()
}

fn check_if_root() -> bool {
    let output = Command::new("id")
        .arg("-u")
        .output()
        .expect("Failed to execute command");

    if let Ok(euid_str) = String::from_utf8(output.stdout) {
        let euid: u32 = euid_str.trim().parse().unwrap_or(1);
        if euid != 0 {
            eprintln!("You must be root to perform this operation.");
            std::process::exit(1);
        }
        return true; // Return true if running as root
    }

    false // If there's an error, return false
}

/// RAII guard to ensure terminal state is properly cleaned up
/// when the TUI exits, either normally or via panic
struct RawModeGuard;

impl RawModeGuard {
  fn new(stdout: &mut io::Stdout) -> anyhow::Result<Self> {
    // Enable raw mode to capture all keyboard input directly
    enable_raw_mode()?;

    // Special handling for "linux" terminal (e.g., TTY console)
    // In dumb terminals, entering alternate screen doesn't auto-clear,
    // so we need to explicitly clear to avoid rendering artifacts
    if let Ok("linux") = env::var("TERM").as_deref() {
      execute!(stdout, Clear(ClearType::All))?;
    }

    // Enter alternate screen buffer to preserve user's terminal content
    execute!(stdout, EnterAlternateScreen)?;
    Ok(Self)
  }
}

/// Cleanup terminal state when the guard is dropped
/// This ensures proper restoration even if the program panics
impl Drop for RawModeGuard {
  fn drop(&mut self) {
    // Ignore errors during cleanup - we're likely panicking or shutting down
    let _ = disable_raw_mode();
    let _ = execute!(io::stdout(), LeaveAlternateScreen);
  }
}

fn main() -> anyhow::Result<()> {
  if env::args().any(|arg| arg == "--version") {
    let version = env!("CARGO_PKG_VERSION");
    println!("aegis-tui version {version}");
    return Ok(());
  }

  check_if_root();
  // Set up panic handler to gracefully restore terminal state
  // This prevents leaving the user's terminal in an unusable state
  // if the installer crashes unexpectedly
  std::panic::set_hook(Box::new(|panic_info| {
    use ratatui::crossterm::{
      execute,
      terminal::{LeaveAlternateScreen, disable_raw_mode},
    };

    // Attempt to restore terminal state - ignore errors since we're panicking
    let _ = disable_raw_mode();
    let _ = execute!(io::stdout(), LeaveAlternateScreen);

    // Print user-friendly panic information to stderr
    eprintln!("====================================================");
    eprintln!("ATHENA OS INSTALLER PANIC - Terminal state restored!");
    eprintln!("====================================================");
    eprintln!("Panic occurred: {panic_info}");
    eprintln!("====================================================");
  }));

  env_logger::init();
  debug!("Logger initialized");

  let mut stdout = io::stdout();
  let res = {
    let _raw_guard = RawModeGuard::new(&mut stdout)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    debug!("Running TUI");
    run_app(&mut terminal)
  };

  debug!("Exiting TUI");

  res
}

/// Processes signals from UI pages to control navigation and installer actions
/// Returns Ok(true) if the application should quit, Ok(false) to continue
fn handle_signal(
  signal: Signal,
  page_stack: &mut Vec<Box<dyn Page>>,
  installer: &mut Installer,
) -> anyhow::Result<bool> {
  match signal {
    Signal::Wait => {
      // Do nothing
    }
    Signal::Push(new_page) => {
      page_stack.push(new_page);
    }
    Signal::Pop => {
      page_stack.pop();
    }
    Signal::PopCount(n) => {
      // Pop n pages from the stack, but never remove the root page
      for _ in 0..n {
        if page_stack.len() > 1 {
          page_stack.pop();
        }
      }
    }
    Signal::Unwind => {
      // Return to the main menu by removing all pages except the root
      while page_stack.len() > 1 {
        page_stack.pop();
      }
    }
    Signal::Quit => {
      debug!("Quit signal received");
      return Ok(true); // Signal to quit
    }
    Signal::WriteCfg => {
        use std::io::Write;
        use serde_json::Value;
    
        debug!("WriteCfg signal received - starting installation process");
    
        // 1) Build the full JSON once
        let config_json = installer.to_json()?;
        debug!(
            "Generated full config JSON:\n{}",
            serde_json::to_string_pretty(&config_json)?
        );
      
        // 2) Extract sections
        let (system_v, drives_v) = match &config_json {
            Value::Object(map) => (
                map.get("config").cloned().unwrap_or(Value::Null),
                map.get("drives").cloned().unwrap_or(Value::Null),
            ),
            _ => (Value::Null, Value::Null),
        };
      
        if system_v.is_null() {
            log::warn!("No 'config' section found; writing null to system_cfg.");
        }
        if drives_v.is_null() {
            log::warn!("No 'drives' section found; writing null to disko_cfg.");
        }
      
        // 3) Write each section to its own temp file
        let mut system_cfg = NamedTempFile::new()?;
        serde_json::to_writer_pretty(&mut system_cfg, &serde_json::json!({ "config": system_v }))?;
        system_cfg.write_all(b"\n")?;
        system_cfg.flush()?;
        debug!("Wrote system_cfg at {}", system_cfg.path().display());
      
        let mut disko_cfg = NamedTempFile::new()?;
        serde_json::to_writer_pretty(&mut disko_cfg,  &serde_json::json!({ "drives": drives_v }))?;
        disko_cfg.write_all(b"\n")?;
        disko_cfg.flush()?;
        debug!("Wrote disko_cfg at {}", disko_cfg.path().display());
      
        // 4) Continue to the progress page, keeping files alive
        page_stack.push(Box::new(InstallProgress::new(
            installer.clone(),
            system_cfg,
            disko_cfg,
        )?));
    }
    Signal::Error(err) => {
      return Err(anyhow::anyhow!("{err}"));
    }
  }
  Ok(false) // Continue running
}

/// Main TUI event loop that manages the installer interface
///
/// This function implements a page-based navigation system using a stack:
/// - Pages are pushed/popped based on user navigation
/// - Each page can send signals to control the overall application flow
/// - The event loop handles both user input and periodic updates (ticks)
pub fn run_app(terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>) -> anyhow::Result<()> {
  let mut installer = Installer::new();
  let mut page_stack: Vec<Box<dyn Page>> = vec![];
  page_stack.push(Box::new(Menu::new()));

  // Set up timing for periodic updates (10 FPS)
  let tick_rate = Duration::from_millis(100);
  let mut last_tick = Instant::now();

  loop {
    // Render the current UI state
    terminal.draw(|f| {
      let chunks = split_vert!(
        f.area(),
        0,
        [
          Constraint::Length(1), // Header height
          Constraint::Min(0),    // Rest of screen
        ]
      );

      // Create three-column header: help text, title, and empty space
      let header_chunks = split_hor!(
        chunks[0],
        0,
        [
          Constraint::Percentage(33), // Left: help text
          Constraint::Percentage(34), // Center: application title
          Constraint::Percentage(33), // Right: reserved for future use
        ]
      );

      // Help text on left
      let help_text = Paragraph::new("Press '?' for help")
        .style(Style::default().fg(Color::Gray))
        .alignment(Alignment::Center);
      f.render_widget(help_text, header_chunks[0]);

      // Title in center
      let title = Paragraph::new("Install Athena OS")
        .style(Style::default().add_modifier(Modifier::BOLD))
        .alignment(Alignment::Center);
      f.render_widget(title, header_chunks[1]);

      // Render the current page (top of the navigation stack)
      if let Some(page) = page_stack.last_mut() {
        page.render(&mut installer, f, chunks[1]);
      }
    })?;

    // Check if the current page has sent any signals
    // Signals control navigation, installation, and application lifecycle
    if let Some(page) = page_stack.last()
      && let Some(signal) = page.signal()
      && handle_signal(signal, &mut page_stack, &mut installer)?
    {
      // handle_signal returned true, meaning we should quit
      break;
    }

    // Calculate remaining time until next tick
    let timeout = tick_rate
      .checked_sub(last_tick.elapsed())
      .unwrap_or_else(|| Duration::from_secs(0));

    // Wait for user input or timeout
    if event::poll(timeout)?
		&& let Event::Key(key) = event::read()? {
			if let Some(page) = page_stack.last_mut() {
				// Forward keyboard input to the current page
				let signal = page.handle_input(&mut installer, key);

				if handle_signal(signal, &mut page_stack, &mut installer)? {
					// Page requested application quit
					break;
				}
			} else {
				// Safety fallback: if no pages exist, return to main menu
				page_stack.push(Box::new(Menu::new()));
			}
		}

    if last_tick.elapsed() >= tick_rate {
      last_tick = Instant::now();
    }
  }

  Ok(())
}
