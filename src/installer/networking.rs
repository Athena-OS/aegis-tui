use ratatui::{
  Frame,
  crossterm::event::{KeyCode, KeyEvent},
  layout::{Constraint, Direction, Layout, Rect},
  style::{Color, Modifier},
  text::Line,
};
use serde_json::Value;

use crate::{
  installer::{Installer, Page, Signal, SshCfg},
  split_hor, split_vert, styled_block, ui_back, ui_close, ui_down, ui_up,
  widget::{Button, CheckBox, ConfigWidget, HelpModal, InfoBox, LineEditor, StrList, WidgetBox},
};

const HIGHLIGHT: Option<(Color, Modifier)> = Some((Color::Yellow, Modifier::BOLD));

pub struct NetworkConfig {
  menu_items: StrList,
  help_modal: HelpModal<'static>,
}

impl NetworkConfig {
  pub fn new() -> Self {
    let items = vec![
      "Network Backend".to_string(),
      "SSH Configuration".to_string(),
      "Back".to_string(),
    ];
    let mut menu_items = StrList::new("Network Configuration", items);
    menu_items.focus();

    let help_content = styled_block(vec![
      vec![
        (Some((Color::Yellow, Modifier::BOLD)), "↑/↓, j/k"),
        (None, " - Navigate menu items"),
      ],
      vec![
        (Some((Color::Yellow, Modifier::BOLD)), "Enter"),
        (None, " - Select menu item"),
      ],
      vec![
        (Some((Color::Yellow, Modifier::BOLD)), "Esc, q, ←, h"),
        (None, " - Return to main menu"),
      ],
      vec![
        (Some((Color::Yellow, Modifier::BOLD)), "?"),
        (None, " - Show this help"),
      ],
      vec![(None, "")],
      vec![(
        None,
        "Configure network settings including backend and SSH.",
      )],
    ]);
    let help_modal = HelpModal::new("Network Configuration", help_content);

    Self {
      menu_items,
      help_modal,
    }
  }

  pub fn display_widget(installer: &mut Installer) -> Option<Box<dyn ConfigWidget>> {
    let mut lines = vec![];

    if let Some(ref backend) = installer.network_backend {
      lines.push(vec![(None, "Network backend: ".into())]);
      lines.push(vec![(HIGHLIGHT, backend.to_string())]);
    } else {
      lines.push(vec![(None, "Network backend: ".into())]);
      lines.push(vec![(HIGHLIGHT, "Not configured".into())]);
    }

    lines.push(vec![(None, "".into())]);

    if let Some(ref ssh) = installer.ssh_config {
      if ssh.enable {
        lines.push(vec![(None, "SSH: ".into())]);
        lines.push(vec![(HIGHLIGHT, "Enabled".into())]);
        lines.push(vec![
          (None, "Port: ".into()),
          (
            HIGHLIGHT,
            installer
              .ssh_config
              .as_ref()
              .map(|cfg| cfg.port)
              .unwrap_or(22)
              .to_string(),
          ), // Use static string for now
        ]);
      } else {
        lines.push(vec![(None, "SSH: ".into())]);
        lines.push(vec![(HIGHLIGHT, "Disabled".into())]);
      }
    } else {
      lines.push(vec![(None, "SSH: ".into())]);
      lines.push(vec![(HIGHLIGHT, "Not configured".into())]);
    }

    let ib = InfoBox::new("", styled_block(lines));
    Some(Box::new(ib) as Box<dyn ConfigWidget>)
  }

  pub fn page_info<'a>() -> (String, Vec<Line<'a>>) {
    (
      "Network".to_string(),
      styled_block(vec![
        vec![(None, "Configure network settings for your system.")],
        vec![(
          None,
          "This includes selecting a network backend and configuring SSH access.",
        )],
      ]),
    )
  }
}

impl Default for NetworkConfig {
  fn default() -> Self {
    Self::new()
  }
}

impl Page for NetworkConfig {
  fn render(&mut self, installer: &mut Installer, f: &mut Frame, area: Rect) {
    let chunks = split_vert!(
      area,
      1,
      [Constraint::Percentage(40), Constraint::Percentage(60)]
    );

    let hor_chunks = split_hor!(
      chunks[1],
      1,
      [
        Constraint::Percentage(30),
        Constraint::Percentage(40),
        Constraint::Percentage(30),
      ]
    );

    // Info box showing current configuration
    let mut info_lines = vec![
      vec![(HIGHLIGHT, "Current Network Configuration".to_string())],
      vec![(None, "".into())],
    ];

    if let Some(ref backend) = installer.network_backend {
      info_lines.push(vec![
        (None, "Network Backend: ".into()),
        (HIGHLIGHT, backend.to_string()),
      ]);
    } else {
      info_lines.push(vec![
        (None, "Network Backend: ".into()),
        (None, "Not configured".into()),
      ]);
    }

    if let Some(ref ssh) = installer.ssh_config {
      if ssh.enable {
        info_lines.push(vec![
          (None, "SSH Server: ".into()),
          (HIGHLIGHT, "Enabled".into()),
        ]);
        info_lines.push(vec![
          (None, "  Port: ".into()),
          (HIGHLIGHT, ssh.port.to_string()), // Use static string for now
        ]);
        info_lines.push(vec![
          (None, "  Password Auth: ".into()),
          (
            HIGHLIGHT,
            if ssh.password_auth {
              "Yes".into()
            } else {
              "No".into()
            },
          ),
        ]);
        info_lines.push(vec![
          (None, "  Root Login: ".into()),
          (
            HIGHLIGHT,
            if ssh.root_login {
              "Yes".into()
            } else {
              "No".into()
            },
          ),
        ]);
      } else {
        info_lines.push(vec![
          (None, "SSH Server: ".into()),
          (None, "Disabled".into()),
        ]);
      }
    } else {
      info_lines.push(vec![
        (None, "SSH Server: ".into()),
        (None, "Not configured".into()),
      ]);
    }

    let info_box = InfoBox::new("", styled_block(info_lines));
    info_box.render(f, chunks[0]);

    self.menu_items.render(f, hor_chunks[1]);
    self.help_modal.render(f, area);
  }

  fn get_help_content(&self) -> (String, Vec<Line<'_>>) {
    let help_content = styled_block(vec![
      vec![
        (Some((Color::Yellow, Modifier::BOLD)), "↑/↓, j/k"),
        (None, " - Navigate menu items"),
      ],
      vec![
        (Some((Color::Yellow, Modifier::BOLD)), "Enter"),
        (None, " - Select menu item"),
      ],
      vec![
        (Some((Color::Yellow, Modifier::BOLD)), "Esc, q, ←, h"),
        (None, " - Return to main menu"),
      ],
      vec![
        (Some((Color::Yellow, Modifier::BOLD)), "?"),
        (None, " - Show this help"),
      ],
      vec![(None, "")],
      vec![(
        None,
        "Configure network settings including backend and SSH.",
      )],
    ]);
    ("Network Configuration".to_string(), help_content)
  }

  fn handle_input(&mut self, _installer: &mut Installer, event: KeyEvent) -> Signal {
    match event.code {
      KeyCode::Char('?') => {
        self.help_modal.toggle();
        Signal::Wait
      }
      ui_close!() if self.help_modal.visible => {
        self.help_modal.hide();
        Signal::Wait
      }
      _ if self.help_modal.visible => Signal::Wait,
      ui_back!() => Signal::Pop,
      KeyCode::Enter => {
        match self.menu_items.selected_idx {
          0 => Signal::Push(Box::new(NetworkBackend::new())),
          1 => Signal::Push(Box::new(SshConfig::new())),
          2 => Signal::Pop, // Back
          _ => Signal::Wait,
        }
      }
      ui_up!() => {
        if !self.menu_items.previous_item() {
          self.menu_items.last_item();
        }
        Signal::Wait
      }
      ui_down!() => {
        if !self.menu_items.next_item() {
          self.menu_items.first_item();
        }
        Signal::Wait
      }
      _ => self.menu_items.handle_input(event),
    }
  }
}

// Network Backend selection page (same as before)
pub struct NetworkBackend {
  backends: StrList,
  help_modal: HelpModal<'static>,
}

impl NetworkBackend {
  pub fn new() -> Self {
    let backends = [
      "NetworkManager",
      "wpa_supplicant",
      "systemd-networkd",
      "None",
    ]
    .iter()
    .map(|s| s.to_string())
    .collect::<Vec<_>>();
    let mut backends = StrList::new("Select Network Backend", backends);
    backends.focus();

    let help_content = styled_block(vec![
      vec![
        (Some((Color::Yellow, Modifier::BOLD)), "↑/↓, j/k"),
        (None, " - Navigate network backend options"),
      ],
      vec![
        (Some((Color::Yellow, Modifier::BOLD)), "Enter"),
        (None, " - Select network backend and return"),
      ],
      vec![
        (Some((Color::Yellow, Modifier::BOLD)), "Esc, q, ←, h"),
        (None, " - Cancel and return"),
      ],
      vec![
        (Some((Color::Yellow, Modifier::BOLD)), "?"),
        (None, " - Show this help"),
      ],
      vec![(None, "")],
      vec![(
        None,
        "Select the network management backend for connections.",
      )],
    ]);
    let help_modal = HelpModal::new("Network Backend", help_content);

    Self {
      backends,
      help_modal,
    }
  }

  pub fn get_network_info<'a>(idx: usize) -> InfoBox<'a> {
    match idx {
      0 => InfoBox::new(
        "NetworkManager",
        styled_block(vec![
          vec![
            (HIGHLIGHT, "NetworkManager"),
            (None, " is a "),
            (HIGHLIGHT, "comprehensive network management daemon"),
            (None, " that provides "),
            (HIGHLIGHT, "automatic network configuration"),
            (None, " and "),
            (HIGHLIGHT, "seamless connectivity management"),
            (None, "."),
          ],
          vec![
            (None, "It supports "),
            (HIGHLIGHT, "WiFi, Ethernet, VPN, and mobile broadband"),
            (None, " connections with "),
            (HIGHLIGHT, "automatic switching"),
            (None, " between available networks."),
          ],
          vec![
            (None, "NetworkManager provides "),
            (HIGHLIGHT, "GUI integration"),
            (None, " and is the "),
            (HIGHLIGHT, "most user-friendly option"),
            (None, " for desktop environments."),
          ],
        ]),
      ),
      1 => InfoBox::new(
        "wpa_supplicant",
        styled_block(vec![
          vec![
            (HIGHLIGHT, "wpa_supplicant"),
            (None, " is a "),
            (HIGHLIGHT, "lightweight WiFi authentication client"),
            (None, " that handles "),
            (HIGHLIGHT, "WPA/WPA2 and WPA3 security protocols"),
            (None, "."),
          ],
          vec![
            (None, "It provides "),
            (HIGHLIGHT, "minimal overhead"),
            (None, " and "),
            (HIGHLIGHT, "direct control"),
            (None, " over wireless connections but requires "),
            (HIGHLIGHT, "manual configuration"),
            (None, " for most setups."),
          ],
          vec![
            (None, "wpa_supplicant is "),
            (HIGHLIGHT, "ideal for servers"),
            (None, " or users who prefer "),
            (HIGHLIGHT, "command-line network management"),
            (None, " with minimal dependencies."),
          ],
        ]),
      ),
      2 => InfoBox::new(
        "systemd-networkd",
        styled_block(vec![
          vec![
            (HIGHLIGHT, "systemd-networkd"),
            (None, " is a "),
            (HIGHLIGHT, "systemd-native network manager"),
            (None, " that provides "),
            (HIGHLIGHT, "efficient and lightweight"),
            (None, " network configuration."),
          ],
          vec![
            (None, "It offers "),
            (HIGHLIGHT, "declarative configuration"),
            (
              None,
              " through configuration files and integrates well with ",
            ),
            (HIGHLIGHT, "systemd-resolved"),
            (None, " for DNS management."),
          ],
          vec![
            (None, "systemd-networkd is "),
            (HIGHLIGHT, "perfect for servers"),
            (None, " and "),
            (HIGHLIGHT, "headless systems"),
            (
              None,
              " but has limited support for complex desktop networking scenarios.",
            ),
          ],
        ]),
      ),
      _ => InfoBox::new(
        "No Backend",
        styled_block(vec![vec![(
          None,
          "No network backend will be installed. Manual network configuration will be required.",
        )]]),
      ),
    }
  }
}

impl Page for NetworkBackend {
  fn render(&mut self, _installer: &mut Installer, f: &mut Frame, area: Rect) {
    let vert_chunks = Layout::default()
      .direction(Direction::Vertical)
      .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
      .split(area);
    let hor_chunks = split_hor!(
      vert_chunks[0],
      1,
      [
        Constraint::Percentage(40),
        Constraint::Percentage(20),
        Constraint::Percentage(40),
      ]
    );

    let idx = self.backends.selected_idx;
    let info_box = Self::get_network_info(idx);
    self.backends.render(f, hor_chunks[1]);
    info_box.render(f, vert_chunks[1]);

    self.help_modal.render(f, area);
  }

  fn get_help_content(&self) -> (String, Vec<Line<'_>>) {
    let help_content = styled_block(vec![
      vec![
        (Some((Color::Yellow, Modifier::BOLD)), "↑/↓, j/k"),
        (None, " - Navigate network backend options"),
      ],
      vec![
        (Some((Color::Yellow, Modifier::BOLD)), "Enter"),
        (None, " - Select network backend and return"),
      ],
      vec![
        (Some((Color::Yellow, Modifier::BOLD)), "Esc, q, ←, h"),
        (None, " - Cancel and return"),
      ],
      vec![
        (Some((Color::Yellow, Modifier::BOLD)), "?"),
        (None, " - Show this help"),
      ],
      vec![(None, "")],
      vec![(
        None,
        "Select the network management backend for connections.",
      )],
    ]);
    ("Network Backend".to_string(), help_content)
  }

  fn handle_input(&mut self, installer: &mut Installer, event: KeyEvent) -> Signal {
    match event.code {
      KeyCode::Char('?') => {
        self.help_modal.toggle();
        Signal::Wait
      }
      ui_close!() if self.help_modal.visible => {
        self.help_modal.hide();
        Signal::Wait
      }
      _ if self.help_modal.visible => Signal::Wait,
      ui_back!() => Signal::Pop,
      KeyCode::Enter => {
        let backend = if self.backends.selected_idx == 3 {
          None
        } else {
          Some(self.backends.items[self.backends.selected_idx].clone())
        };
        installer.network_backend = backend;
        Signal::Pop
      }
      ui_up!() => {
        if !self.backends.previous_item() {
          self.backends.last_item();
        }
        Signal::Wait
      }
      ui_down!() => {
        if !self.backends.next_item() {
          self.backends.first_item();
        }
        Signal::Wait
      }
      _ => self.backends.handle_input(event),
    }
  }
}

// Simplified SSH Configuration page (no authorized keys)
pub struct SshConfig {
  buttons: WidgetBox,
  port_input: LineEditor,
  help_modal: HelpModal<'static>,
  input_mode: SshInputMode,
  // State tracking
  enable_ssh: bool,
  password_auth: bool,
  root_login: bool,
  initialized: bool,
}

enum SshInputMode {
  Buttons,
  Port,
}

impl SshConfig {
  pub fn new() -> Self {
    let enable_ssh = CheckBox::new("Enable SSH", false);
    let password_auth = CheckBox::new("Allow Password Authentication", true);
    let root_login = CheckBox::new("Allow Root Login", false);
    let port_btn = Button::new("Configure Port");
    let back_btn = Button::new("Back");

    let mut buttons = WidgetBox::button_menu(vec![
      Box::new(enable_ssh),
      Box::new(password_auth),
      Box::new(root_login),
      Box::new(port_btn),
      Box::new(back_btn),
    ]);
    buttons.focus();

    let port_input = LineEditor::new("SSH Port", Some("Default: 22"));

    let help_content = styled_block(vec![
      vec![
        (Some((Color::Yellow, Modifier::BOLD)), "↑/↓, j/k"),
        (None, " - Navigate options"),
      ],
      vec![
        (Some((Color::Yellow, Modifier::BOLD)), "Enter"),
        (None, " - Toggle option or select action"),
      ],
      vec![
        (Some((Color::Yellow, Modifier::BOLD)), "Tab"),
        (None, " - Move to port input"),
      ],
      vec![
        (Some((Color::Yellow, Modifier::BOLD)), "Esc, q, ←, h"),
        (None, " - Cancel and return"),
      ],
      vec![
        (Some((Color::Yellow, Modifier::BOLD)), "?"),
        (None, " - Show this help"),
      ],
      vec![(None, "")],
      vec![(None, "Configure SSH server settings for remote access.")],
    ]);
    let help_modal = HelpModal::new("SSH Configuration", help_content);

    Self {
      buttons,
      port_input,
      help_modal,
      input_mode: SshInputMode::Buttons,
      enable_ssh: false,
      password_auth: true,
      root_login: false,
      initialized: false,
    }
  }

  fn update_from_config(&mut self, installer: &Installer) {
    if let Some(ref cfg) = installer.ssh_config {
      // Update instance state
      self.enable_ssh = cfg.enable;
      self.password_auth = cfg.password_auth;
      self.root_login = cfg.root_login;

      // Update inputs
      self.port_input.set_value(&cfg.port.to_string());
    }

    // Always recreate buttons with current state values
    let enable_ssh = CheckBox::new("Enable SSH", self.enable_ssh);
    let password_auth = CheckBox::new("Allow Password Authentication", self.password_auth);
    let root_login = CheckBox::new("Allow Root Login", self.root_login);
    let port_btn = Button::new("Configure Port");
    let back_btn = Button::new("Back");

    self.buttons.set_children_inplace(vec![
      Box::new(enable_ssh),
      Box::new(password_auth),
      Box::new(root_login),
      Box::new(port_btn),
      Box::new(back_btn),
    ]);

    // Ensure buttons are focused
    self.buttons.focus();
  }

  fn save_to_config(&self, installer: &mut Installer) {
    let port = self
      .port_input
      .get_value()
      .and_then(|v| {
        if let Value::String(s) = v {
          Some(s)
        } else {
          None
        }
      })
      .and_then(|s| s.parse::<u16>().ok())
      .unwrap_or(22);

    installer.ssh_config = Some(SshCfg {
      enable: self.enable_ssh,
      port,
      password_auth: self.password_auth,
      root_login: self.root_login,
    });
  }
}

impl Page for SshConfig {
  fn render(&mut self, installer: &mut Installer, f: &mut Frame, area: Rect) {
    // Only update config on first render
    if !self.initialized {
      self.update_from_config(installer);
      self.initialized = true;
    }

    let chunks = split_vert!(
      area,
      1,
      [Constraint::Percentage(30), Constraint::Percentage(70)]
    );

    // Info box
    let info_lines = vec![
      vec![(
        None,
        "Configure SSH server for secure remote access to your system.",
      )],
      vec![(None, "")],
      vec![(
        None,
        "SSH (Secure Shell) allows encrypted remote login and command execution.",
      )],
      vec![(None, "")],
      vec![(HIGHLIGHT, "Security Recommendations:")],
      vec![(None, "• Use key-based authentication when possible")],
      vec![(None, "• Disable root login for better security")],
      vec![(None, "• Consider changing the default port")],
    ];

    let info_box = InfoBox::new("SSH Server", styled_block(info_lines));
    info_box.render(f, chunks[0]);

    // Controls area
    match self.input_mode {
      SshInputMode::Buttons => {
        let hor_chunks = split_hor!(
          chunks[1],
          1,
          [
            Constraint::Percentage(25),
            Constraint::Percentage(50),
            Constraint::Percentage(25),
          ]
        );
        self.buttons.render(f, hor_chunks[1]);
      }
      SshInputMode::Port => {
        let input_chunks = split_hor!(
          chunks[1],
          1,
          [
            Constraint::Percentage(25),
            Constraint::Percentage(50),
            Constraint::Percentage(25),
          ]
        );
        self.port_input.render(f, input_chunks[1]);
      }
    }

    self.help_modal.render(f, area);
  }

  fn get_help_content(&self) -> (String, Vec<Line<'_>>) {
    let help_content = styled_block(vec![
      vec![
        (Some((Color::Yellow, Modifier::BOLD)), "↑/↓, j/k"),
        (None, " - Navigate options"),
      ],
      vec![
        (Some((Color::Yellow, Modifier::BOLD)), "Enter"),
        (None, " - Toggle option or select action"),
      ],
      vec![
        (Some((Color::Yellow, Modifier::BOLD)), "Tab"),
        (None, " - Move to port input"),
      ],
      vec![
        (Some((Color::Yellow, Modifier::BOLD)), "Esc, q, ←, h"),
        (None, " - Cancel and return"),
      ],
      vec![
        (Some((Color::Yellow, Modifier::BOLD)), "?"),
        (None, " - Show this help"),
      ],
      vec![(None, "")],
      vec![(None, "Configure SSH server settings for remote access.")],
    ]);
    ("SSH Configuration".to_string(), help_content)
  }

  fn handle_input(&mut self, installer: &mut Installer, event: KeyEvent) -> Signal {
    match event.code {
      KeyCode::Char('?') => {
        self.help_modal.toggle();
        Signal::Wait
      }
      ui_close!() if self.help_modal.visible => {
        self.help_modal.hide();
        Signal::Wait
      }
      _ if self.help_modal.visible => Signal::Wait,
      ui_back!() => match self.input_mode {
        SshInputMode::Port => {
          self.input_mode = SshInputMode::Buttons;
          self.port_input.unfocus();
          self.buttons.focus();
          Signal::Wait
        }
        SshInputMode::Buttons => {
          self.save_to_config(installer);
          Signal::Pop
        }
      },
      _ => {
        match self.input_mode {
          SshInputMode::Buttons => {
            match event.code {
              ui_up!() => {
                self.buttons.prev_child();
                Signal::Wait
              }
              ui_down!() => {
                self.buttons.next_child();
                Signal::Wait
              }
              KeyCode::Enter => {
                match self.buttons.selected_child() {
                  Some(0) => {
                    // Toggle Enable SSH checkbox
                    if let Some(checkbox) = self.buttons.focused_child_mut() {
                      checkbox.interact();
                      if let Some(Value::Bool(enabled)) = checkbox.get_value() {
                        self.enable_ssh = enabled;
                      }
                    }
                    Signal::Wait
                  }
                  Some(1) => {
                    // Toggle Password Auth checkbox
                    if let Some(checkbox) = self.buttons.focused_child_mut() {
                      checkbox.interact();
                      if let Some(Value::Bool(enabled)) = checkbox.get_value() {
                        self.password_auth = enabled;
                      }
                    }
                    Signal::Wait
                  }
                  Some(2) => {
                    // Toggle Root Login checkbox
                    if let Some(checkbox) = self.buttons.focused_child_mut() {
                      checkbox.interact();
                      if let Some(Value::Bool(enabled)) = checkbox.get_value() {
                        self.root_login = enabled;
                      }
                    }
                    Signal::Wait
                  }
                  Some(3) => {
                    // Configure Port
                    self.input_mode = SshInputMode::Port;
                    self.buttons.unfocus();
                    self.port_input.focus();
                    Signal::Wait
                  }
                  Some(4) => {
                    // Back button
                    self.save_to_config(installer);
                    Signal::Pop
                  }
                  _ => Signal::Wait,
                }
              }
              _ => Signal::Wait,
            }
          }
          SshInputMode::Port => match event.code {
            KeyCode::Enter | KeyCode::Tab => {
              // Save the current port value and return to buttons
              self.input_mode = SshInputMode::Buttons;
              self.port_input.unfocus();
              self.buttons.focus();
              Signal::Wait
            }
            _ => self.port_input.handle_input(event),
          },
        }
      }
    }
  }
}
