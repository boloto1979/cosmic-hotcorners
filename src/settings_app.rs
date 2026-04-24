// SPDX-License-Identifier: MPL-2.0

use crate::config::{Config, CornerAction};
use cosmic::app::Task;
use cosmic::cosmic_config::{self, CosmicConfigEntry};
use cosmic::iced::Length;
use cosmic::widget::{self, Row, Column};
use cosmic::{ApplicationExt, Element};

static ACTION_LABELS: &[&str] = &[
    "Disabled",
    "Show Workspaces",
    "Open Launcher",
    "Run Command",
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Corner {
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
}

impl Corner {
    fn label(self) -> &'static str {
        match self {
            Corner::TopLeft => "Top-Left",
            Corner::TopRight => "Top-Right",
            Corner::BottomLeft => "Bottom-Left",
            Corner::BottomRight => "Bottom-Right",
        }
    }

    fn index(self) -> usize {
        match self {
            Corner::TopLeft => 0,
            Corner::TopRight => 1,
            Corner::BottomLeft => 2,
            Corner::BottomRight => 3,
        }
    }
}

fn action_to_index(action: &CornerAction) -> usize {
    match action {
        CornerAction::Disabled => 0,
        CornerAction::ShowWorkspaces => 1,
        CornerAction::OpenLauncher => 2,
        CornerAction::RunCommand(_) => 3,
    }
}

fn action_cmd(action: &CornerAction) -> String {
    match action {
        CornerAction::RunCommand(cmd) => cmd.clone(),
        _ => String::new(),
    }
}

fn index_to_action(index: usize, cmd: &str) -> CornerAction {
    match index {
        1 => CornerAction::ShowWorkspaces,
        2 => CornerAction::OpenLauncher,
        3 => CornerAction::RunCommand(cmd.to_string()),
        _ => CornerAction::Disabled,
    }
}

pub struct SettingsApp {
    core: cosmic::Core,
    config: Config,
    config_ctx: Option<cosmic_config::Config>,
    selected: [usize; 4],
    commands: [String; 4],
    delay_str: String,
}

#[derive(Debug, Clone)]
pub enum Message {
    Toggled(bool),
    ActionSelected(Corner, usize),
    CommandChanged(Corner, String),
    DelayChanged(String),
}

impl cosmic::Application for SettingsApp {
    type Executor = cosmic::executor::Default;
    type Flags = ();
    type Message = Message;

    const APP_ID: &'static str = "io.github.cosmic-hot-corners";

    fn core(&self) -> &cosmic::Core {
        &self.core
    }

    fn core_mut(&mut self) -> &mut cosmic::Core {
        &mut self.core
    }

    fn init(core: cosmic::Core, _flags: ()) -> (Self, Task<Message>) {
        let config_ctx = cosmic_config::Config::new(Self::APP_ID, Config::VERSION).ok();
        let config = config_ctx
            .as_ref()
            .and_then(|h| Config::get_entry(h).ok())
            .unwrap_or_default();

        let actions = [
            &config.top_left,
            &config.top_right,
            &config.bottom_left,
            &config.bottom_right,
        ];
        let selected = actions.map(action_to_index);
        let commands = actions.map(action_cmd);
        let delay_str = config.delay_ms.to_string();

        let mut app = SettingsApp {
            core,
            config,
            config_ctx,
            selected,
            commands,
            delay_str,
        };

        let title_task = app.set_header_title("Hot Corners".to_string());

        (app, title_task.into())
    }

    fn view(&self) -> Element<'_, Message> {
        let spacing = cosmic::theme::spacing();

        let enable_row = widget::settings::item(
            "Enable Hot Corners",
            widget::toggler(self.config.enabled).on_toggle(Message::Toggled),
        );

        let delay_row = widget::settings::item(
            "Activation delay (ms)",
            widget::text_input("300", &self.delay_str)
                .on_input(Message::DelayChanged)
                .width(Length::Fixed(100.0)),
        );

        let top_row = Row::new()
            .push(self.corner_section(Corner::TopLeft))
            .push(self.corner_section(Corner::TopRight))
            .spacing(spacing.space_m)
            .width(Length::Fill);

        let bottom_row = Row::new()
            .push(self.corner_section(Corner::BottomLeft))
            .push(self.corner_section(Corner::BottomRight))
            .spacing(spacing.space_m)
            .width(Length::Fill);

        Column::new()
            .push(widget::list_column().add(enable_row).add(delay_row))
            .push(top_row)
            .push(bottom_row)
            .spacing(spacing.space_m)
            .padding(spacing.space_l)
            .into()
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Toggled(v) => {
                self.config.enabled = v;
                self.flush();
            }
            Message::ActionSelected(corner, index) => {
                self.selected[corner.index()] = index;
                self.apply_corner(corner);
            }
            Message::CommandChanged(corner, cmd) => {
                self.commands[corner.index()] = cmd;
                if self.selected[corner.index()] == 5 {
                    self.apply_corner(corner);
                }
            }
            Message::DelayChanged(s) => {
                self.delay_str = s.clone();
                if let Ok(ms) = s.parse::<u64>() {
                    self.config.delay_ms = ms;
                    self.flush();
                }
            }
        }
        Task::none()
    }
}

impl SettingsApp {
    fn corner_section(&self, corner: Corner) -> Element<'_, Message> {
        let i = corner.index();

        let mut section = widget::list_column().add(widget::settings::item(
            "Action",
            widget::dropdown(
                ACTION_LABELS,
                Some(self.selected[i]),
                move |idx| Message::ActionSelected(corner, idx),
            ),
        ));

        if self.selected[i] == 3 {
            section = section.add(widget::settings::item(
                "Command",
                widget::text_input("sh -c ...", &self.commands[i])
                    .on_input(move |s| Message::CommandChanged(corner, s)),
            ));
        }

        Column::new()
            .push(widget::text::heading(corner.label()))
            .push(section)
            .spacing(cosmic::theme::spacing().space_xs)
            .width(Length::Fill)
            .into()
    }

    fn apply_corner(&mut self, corner: Corner) {
        let i = corner.index();
        let action = index_to_action(self.selected[i], &self.commands[i]);
        match corner {
            Corner::TopLeft => self.config.top_left = action,
            Corner::TopRight => self.config.top_right = action,
            Corner::BottomLeft => self.config.bottom_left = action,
            Corner::BottomRight => self.config.bottom_right = action,
        }
        self.flush();
    }

    fn flush(&self) {
        if let Some(ctx) = &self.config_ctx {
            let _ = self.config.write_entry(ctx);
        }
    }
}
