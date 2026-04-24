// SPDX-License-Identifier: MPL-2.0

use crate::config::{Config, CornerAction};
use cosmic::app::Task;
use cosmic::cosmic_config::{self, CosmicConfigEntry};
use cosmic::iced::event::listen_with;
use cosmic::iced::event::wayland::{Event as WaylandEvent, OutputEvent};
use cosmic::iced::platform_specific::shell::commands::layer_surface::{
    destroy_layer_surface, get_layer_surface, Anchor, KeyboardInteractivity, Layer,
};
use cosmic::iced::platform_specific::runtime::wayland::layer_surface::{
    IcedOutput, SctkLayerSurfaceSettings,
};
use cosmic::iced::{mouse, Event, Length, Subscription, window};
use cosmic::widget;
use cosmic::ApplicationExt;
use cosmic::Element;
use cosmic::cctk::sctk::reexports::client::protocol::wl_output::WlOutput;
use std::time::Duration;

const CORNER_SIZE: u32 = 10;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Corner {
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
}

impl Corner {
    fn anchor(self) -> Anchor {
        match self {
            Corner::TopLeft => Anchor::TOP.union(Anchor::LEFT),
            Corner::TopRight => Anchor::TOP.union(Anchor::RIGHT),
            Corner::BottomLeft => Anchor::BOTTOM.union(Anchor::LEFT),
            Corner::BottomRight => Anchor::BOTTOM.union(Anchor::RIGHT),
        }
    }

    fn name(self) -> &'static str {
        match self {
            Corner::TopLeft => "Top-Left",
            Corner::TopRight => "Top-Right",
            Corner::BottomLeft => "Bottom-Left",
            Corner::BottomRight => "Bottom-Right",
        }
    }
}

const CORNERS: [Corner; 4] = [
    Corner::TopLeft,
    Corner::TopRight,
    Corner::BottomLeft,
    Corner::BottomRight,
];

pub struct AppModel {
    core: cosmic::Core,
    outputs: Vec<(WlOutput, [(window::Id, Corner); 4])>,
    config: Config,
    pending_generation: u64,
    active_corner: Option<window::Id>,
}

#[derive(Debug, Clone)]
pub enum Message {
    CursorMoved(window::Id),
    CursorLeft(window::Id),
    TriggerCorner(Corner, u64),
    OutputAdded(WlOutput),
    OutputRemoved(WlOutput),
    ConfigUpdated(Config),
    Noop,
}

impl cosmic::Application for AppModel {
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

    fn init(
        core: cosmic::Core,
        _flags: Self::Flags,
    ) -> (Self, Task<Message>) {
        let config = cosmic_config::Config::new(Self::APP_ID, Config::VERSION)
            .map(|ctx| match Config::get_entry(&ctx) {
                Ok(c) => c,
                Err((_, c)) => c,
            })
            .unwrap_or_default();

        (
            AppModel {
                core,
                outputs: Vec::new(),
                config,
                pending_generation: 0,
                active_corner: None,
            },
            Task::none(),
        )
    }

    fn view(&self) -> Element<'_, Message> {
        widget::Space::new().into()
    }

    fn view_window(&self, _id: window::Id) -> Element<'_, Message> {
        widget::Space::new()
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }

    fn subscription(&self) -> Subscription<Message> {
        let pointer_events = listen_with(|event, _status, window_id| match event {
            Event::Mouse(mouse::Event::CursorMoved { .. }) => Some(Message::CursorMoved(window_id)),
            Event::Mouse(mouse::Event::CursorLeft) => Some(Message::CursorLeft(window_id)),
            Event::PlatformSpecific(cosmic::iced::event::PlatformSpecific::Wayland(
                WaylandEvent::Output(evt, output),
            )) => match evt {
                OutputEvent::Created(_) => Some(Message::OutputAdded(output)),
                OutputEvent::Removed => Some(Message::OutputRemoved(output)),
                _ => None,
            },
            _ => None,
        });

        let config_watch = self
            .watch_config::<Config>(Self::APP_ID)
            .map(|update| Message::ConfigUpdated(update.config));

        Subscription::batch([pointer_events, config_watch])
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::OutputAdded(output) => {
                eprintln!("[hot-corners] OutputAdded! Creating 4 surfaces...");
                let surfaces: [(window::Id, SctkLayerSurfaceSettings); 4] = CORNERS.map(|corner| {
                    let id = window::Id::unique();
                    let settings = SctkLayerSurfaceSettings {
                        id,
                        layer: Layer::Overlay,
                        keyboard_interactivity: KeyboardInteractivity::None,
                        input_zone: None,
                        anchor: corner.anchor(),
                        size: Some((Some(CORNER_SIZE), Some(CORNER_SIZE))),
                        exclusive_zone: -1,
                        output: IcedOutput::Output(output.clone()),
                        namespace: String::from("hot-corners"),
                        ..Default::default()
                    };
                    (id, settings)
                });

                let corner_ids: [(window::Id, Corner); 4] =
                    std::array::from_fn(|i| (surfaces[i].0, CORNERS[i]));

                self.outputs.push((output, corner_ids));

                let tasks: Vec<Task<Message>> = surfaces
                    .into_iter()
                    .map(|(_, s)| get_layer_surface(s))
                    .collect();
                eprintln!("[hot-corners] {} surfaces created.", tasks.len());
                return Task::batch(tasks);
            }
            Message::OutputRemoved(output) => {
                if let Some(pos) = self.outputs.iter().position(|(o, _)| *o == output) {
                    let (_, corner_ids) = self.outputs.remove(pos);
                    let tasks: Vec<Task<Message>> = corner_ids
                        .iter()
                        .map(|(id, _): &(window::Id, Corner)| destroy_layer_surface(*id))
                        .collect();
                    return Task::batch(tasks);
                }
            }
            Message::ConfigUpdated(config) => {
                self.config = config;
            }
            Message::CursorMoved(id) => {
                eprintln!("[hot-corners] CursorMoved on {:?}, active={:?}, outputs={}", id, self.active_corner, self.outputs.len());
                if self.active_corner == Some(id) {
                    return Task::none();
                }
                let corner = self
                    .outputs
                    .iter()
                    .flat_map(|(_, ids): &(WlOutput, [(window::Id, Corner); 4])| ids.iter())
                    .find(|(cid, _)| *cid == id)
                    .map(|(_, c)| *c);
                if corner.is_none() {
                    if self.active_corner.is_some() {
                        self.active_corner = None;
                        self.pending_generation += 1;
                    }
                    return Task::none();
                }
                let corner = corner.unwrap();
                self.active_corner = Some(id);
                self.pending_generation += 1;
                let trigger_gen = self.pending_generation;
                let delay = Duration::from_millis(self.config.delay_ms);
                return cosmic::task::future(async move {
                    tokio::time::sleep(delay).await;
                    Message::TriggerCorner(corner, trigger_gen)
                });
            }
            Message::CursorLeft(id) => {
                let known = self
                    .outputs
                    .iter()
                    .flat_map(|(_, ids): &(WlOutput, [(window::Id, Corner); 4])| ids.iter())
                    .any(|(cid, _)| *cid == id);
                if known {
                    self.active_corner = None;
                    self.pending_generation += 1;
                }
            }
            Message::TriggerCorner(corner, trigger_gen) => {
                if trigger_gen == self.pending_generation && self.config.enabled {
                    let action = match corner {
                        Corner::TopLeft => &self.config.top_left,
                        Corner::TopRight => &self.config.top_right,
                        Corner::BottomLeft => &self.config.bottom_left,
                        Corner::BottomRight => &self.config.bottom_right,
                    };
                    println!("[hot-corners] Triggering {} → {:?}", corner.name(), action);
                    return execute_action(action);
                }
            }
            Message::Noop => {}
        }
        Task::none()
    }
}

fn execute_action(action: &CornerAction) -> Task<Message> {
    match action {
        CornerAction::Disabled => Task::none(),
        CornerAction::ShowWorkspaces => cosmic::task::future(async {
            let _ = dbus_show_workspaces().await;
            Message::Noop
        }),
        CornerAction::OpenLauncher => cosmic::task::future(async {
            let _ = dbus_open_launcher().await;
            Message::Noop
        }),
        CornerAction::RunCommand(cmd) => {
            let _ = std::process::Command::new("sh").args(["-c", cmd]).spawn();
            Task::none()
        }
    }
}

async fn dbus_show_workspaces() -> zbus::Result<()> {
    let conn = zbus::Connection::session().await?;
    conn.call_method(
        Some("com.system76.CosmicWorkspaces"),
        "/com/system76/CosmicWorkspaces",
        Some("com.system76.CosmicWorkspaces"),
        "Show",
        &(),
    )
    .await?;
    Ok(())
}

async fn dbus_open_launcher() -> zbus::Result<()> {
    let conn = zbus::Connection::session().await?;
    let args: std::collections::HashMap<&str, zbus::zvariant::Value<'_>> =
        std::collections::HashMap::new();
    conn.call_method(
        Some("com.system76.CosmicLauncher"),
        "/com/system76/CosmicLauncher",
        Some("org.freedesktop.DbusActivation"),
        "Activate",
        &args,
    )
    .await?;
    Ok(())
}
