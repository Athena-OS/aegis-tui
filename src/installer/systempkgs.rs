use std::{collections::HashSet, sync::LazyLock};

use ratatui::{layout::Constraint, text::Line};
use serde_json::Value;

use crate::{
  installer::{Installer, Page, Signal},
  styled_block,
  widget::{ConfigWidget, PackagePicker, TableWidget},
};

use std::{
  sync::{Arc, RwLock},
  thread,
};

pub static NIXPKGS: LazyLock<Arc<RwLock<Option<Vec<String>>>>> =
  LazyLock::new(|| Arc::new(RwLock::new(None)));

pub fn init_nixpkgs() {
  let pkgs_ref = NIXPKGS.clone();
  thread::spawn(move || {
    let pkgs = fetch_nixpkgs().unwrap_or_else(|e| {
      eprintln!("Failed to fetch nixpkgs: {e}");
      vec![]
    });
    let mut pkgs_lock = pkgs_ref.write().unwrap();
    *pkgs_lock = Some(pkgs);
  });
}

pub fn fetch_nixpkgs() -> anyhow::Result<Vec<String>> {
  let json: Value = {
    /*
    TODO: find a better way to do this? it kind of sucks
    let output = Command::new("nix")
      .args(["--extra-experimental-features", "nix-command flakes", "search", "nixpkgs", "^", "--json"])
      .output()?;
    */
    let precomputed = include_str!("../../pkgs.json");
    serde_json::from_str(precomputed)?
  };
  let pkgs_object = json
    .as_object()
    .ok_or_else(|| anyhow::anyhow!("Expected JSON object"))?;

  let mut pkgs = Vec::with_capacity(pkgs_object.len());

  for key in pkgs_object.keys() {
    let stripped = key
      .strip_prefix("legacyPackages.x86_64-linux.")
      .unwrap_or(key);
    pkgs.push(stripped.to_string());
  }

  let mut seen = HashSet::new();
  pkgs.retain(|pkg| seen.insert(pkg.clone()));

  Ok(pkgs)
}

pub fn get_available_pkgs() -> anyhow::Result<Vec<String>> {
  let mut retries = 0;
  loop {
    let guard = NIXPKGS.read().unwrap();
    if let Some(nixpkgs) = guard.as_ref() {
      // Great, the package list has been populated
      break Ok(nixpkgs.clone());
    }
    drop(guard); // Release lock before sleeping

    if retries >= 5 {
      // Last attempt to grab the package list before breaking
      break Ok(fetch_nixpkgs().unwrap_or_default());
    }

    std::thread::sleep(std::time::Duration::from_millis(500));
    retries += 1;
  }
}

pub struct SystemPackages {
  package_picker: PackagePicker,
}

impl SystemPackages {
  pub fn new(selected_pkgs: Vec<String>, available_pkgs: Vec<String>) -> Self {
    let package_picker = PackagePicker::new(
      "Selected Packages",
      "Available Packages",
      selected_pkgs,
      available_pkgs,
    );

    Self { package_picker }
  }
  pub fn display_widget(installer: &mut Installer) -> Option<Box<dyn ConfigWidget>> {
    let sys_pkgs: Vec<Vec<String>> = installer
      .system_pkgs
      .clone()
      .into_iter()
      .map(|item| vec![item])
      .collect();
    if sys_pkgs.is_empty() {
      return None;
    }
    Some(Box::new(TableWidget::new(
      "",
      vec![Constraint::Percentage(100)],
      vec!["Packages".into()],
      sys_pkgs,
    )) as Box<dyn ConfigWidget>)
  }
  pub fn page_info<'a>() -> (String, Vec<Line<'a>>) {
    (
      "System Packages".to_string(),
      styled_block(vec![vec![(
        None,
        "Select extra system packages to include in the configuration",
      )]]),
    )
  }
}

impl Page for SystemPackages {
  fn render(
    &mut self,
    _installer: &mut super::Installer,
    f: &mut ratatui::Frame,
    area: ratatui::prelude::Rect,
  ) {
    self.package_picker.render(f, area);
  }

  fn handle_input(
    &mut self,
    installer: &mut super::Installer,
    event: ratatui::crossterm::event::KeyEvent,
  ) -> super::Signal {
    use ratatui::crossterm::event::KeyCode;

    // Handle quit/escape at the top level
    match event.code {
      KeyCode::Esc | KeyCode::Char('q') => return Signal::Pop,
      _ => {}
    }

    // Store the current selected packages before handling input
    let previous_selection = self.package_picker.get_selected_packages();

    // Handle the input with the package picker
    let signal = self.package_picker.handle_input(event);

    // Update installer's system_pkgs if the selection changed
    let current_selection = self.package_picker.get_selected_packages();
    if previous_selection != current_selection {
      installer.system_pkgs = current_selection;
    }

    signal
  }

  fn get_help_content(&self) -> (String, Vec<Line<'_>>) {
    let help_content = styled_block(vec![
      vec![
        (
          Some((
            ratatui::style::Color::Yellow,
            ratatui::style::Modifier::BOLD,
          )),
          "Tab",
        ),
        (None, " - Switch between lists and search"),
      ],
      vec![
        (
          Some((
            ratatui::style::Color::Yellow,
            ratatui::style::Modifier::BOLD,
          )),
          "↑/↓, j/k",
        ),
        (None, " - Navigate package lists"),
      ],
      vec![
        (
          Some((
            ratatui::style::Color::Yellow,
            ratatui::style::Modifier::BOLD,
          )),
          "Enter",
        ),
        (None, " - Add/remove package to/from selection"),
      ],
      vec![
        (
          Some((
            ratatui::style::Color::Yellow,
            ratatui::style::Modifier::BOLD,
          )),
          "/",
        ),
        (None, " - Focus search bar"),
      ],
      vec![
        (
          Some((
            ratatui::style::Color::Yellow,
            ratatui::style::Modifier::BOLD,
          )),
          "Esc",
        ),
        (None, " - Return to main menu"),
      ],
      vec![
        (
          Some((
            ratatui::style::Color::Yellow,
            ratatui::style::Modifier::BOLD,
          )),
          "?",
        ),
        (None, " - Show this help"),
      ],
      vec![(None, "")],
      vec![(None, "Search filters packages in real-time as you type.")],
      vec![(None, "Filter persists when adding/removing packages.")],
      vec![(
        None,
        "Selected packages will be installed on your NixOS system.",
      )],
    ]);
    ("System Packages".to_string(), help_content)
  }
}
