// SPDX-License-Identifier: MPL-2.0

use cosmic::app::Task;
use cosmic::iced::event::listen_with;
use cosmic::iced::platform_specific::shell::commands::layer_surface::{
    get_layer_surface, Anchor, KeyboardInteractivity, Layer,
};
use cosmic::iced::platform_specific::runtime::wayland::layer_surface::SctkLayerSurfaceSettings;
use cosmic::iced::{mouse, Event, Length, Subscription, window};
use cosmic::widget;
use cosmic::Element;

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
    corner_ids: [(window::Id, Corner); 4],
}

#[derive(Debug, Clone)]
pub enum Message {
    CursorEntered(window::Id),
    CursorLeft(window::Id),
    CursorMoved(window::Id, cosmic::iced::Point),
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
        let surfaces: [(window::Id, SctkLayerSurfaceSettings); 4] = CORNERS.map(|corner| {
            let id = window::Id::unique();
            let settings = SctkLayerSurfaceSettings {
                id,
                layer: Layer::Overlay,
                keyboard_interactivity: KeyboardInteractivity::None,
                input_zone: None,
                anchor: corner.anchor(),
                size: Some((Some(CORNER_SIZE), Some(CORNER_SIZE))),
                exclusive_zone: 0,
                namespace: String::from("hot-corners"),
                ..Default::default()
            };
            (id, settings)
        });

        let corner_ids: [(window::Id, Corner); 4] =
            std::array::from_fn(|i| (surfaces[i].0, CORNERS[i]));

        println!("[hot-corners] Iniciando — criando superfícies em 4 cantos...");
        for (id, corner) in &corner_ids {
            println!("  {:?} → {}", id, corner.name());
        }

        let task_vec: Vec<Task<Message>> = surfaces
            .into_iter()
            .map(|(_, s)| get_layer_surface(s))
            .collect();

        (AppModel { core, corner_ids }, Task::batch(task_vec))
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
        listen_with(|event, _status, window_id| match event {
            Event::Mouse(mouse::Event::CursorEntered) => Some(Message::CursorEntered(window_id)),
            Event::Mouse(mouse::Event::CursorLeft) => Some(Message::CursorLeft(window_id)),
            Event::Mouse(mouse::Event::CursorMoved { position }) => {
                Some(Message::CursorMoved(window_id, position))
            }
            _ => None,
        })
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::CursorEntered(id) => {
                if let Some((_, corner)) = self.corner_ids.iter().find(|(cid, _)| *cid == id) {
                    println!("[hot-corners] >>> ENTROU no canto: {}", corner.name());
                }
            }
            Message::CursorLeft(id) => {
                if let Some((_, corner)) = self.corner_ids.iter().find(|(cid, _)| *cid == id) {
                    println!("[hot-corners] <<< SAIU do canto: {}", corner.name());
                }
            }
            Message::CursorMoved(id, pos) => {
                if let Some((_, corner)) = self.corner_ids.iter().find(|(cid, _)| *cid == id) {
                    println!(
                        "[hot-corners] Cursor em {} → ({:.1}, {:.1})",
                        corner.name(),
                        pos.x,
                        pos.y
                    );
                }
            }
        }
        Task::none()
    }
}
