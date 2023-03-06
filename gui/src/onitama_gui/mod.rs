use iced::{
    executor, theme,
    widget::{
        button, column, container,
        pane_grid::{self, Axis, Pane},
        row, scrollable, text, PaneGrid,
    },
    Alignment, Application, Color, Command, Element, Length, Theme,
};

use self::message::Message;

pub mod message;

pub struct OnitamaGui {
    /// IDs of the panes used in the GUI
    panes: pane_grid::State<(pane_grid::State<i32>, Pane)>,
}

impl Application for OnitamaGui {
    type Executor = executor::Default;

    type Message = Message;

    type Theme = Theme;

    type Flags = ();

    fn new(_flags: Self::Flags) -> (Self, iced::Command<Self::Message>) {
        // let panes = (0..4)
        //     .into_iter()
        //     .map(|x| pane_grid::State::new(x).0)
        //     .collect::<Vec<pane_grid::State<usize>>>();
        let pane0 = pane_grid::State::new(0);
        let pane1 = pane_grid::State::new(1);
        let pane2 = pane_grid::State::new(2);
        let pane3 = pane_grid::State::new(3);

        let split1 = pane_grid::Configuration::Split {
            axis: Axis::Vertical,
            ratio: 0.5,
            a: Box::new(pane_grid::Configuration::Pane(pane0)),
            b: Box::new(pane_grid::Configuration::Pane(pane1)),
        };

        let split2 = pane_grid::Configuration::Split {
            axis: Axis::Horizontal,
            ratio: 0.5,
            a: Box::new(pane_grid::Configuration::Pane(pane2)),
            b: Box::new(pane_grid::Configuration::Pane(pane3)),
        };

        let conf = pane_grid::Configuration::Split {
            axis: Axis::Vertical,
            ratio: 0.7,
            a: Box::new(split1),
            b: Box::new(split2),
        };

        let panes = pane_grid::State::with_configuration(conf);

        // let (mut panes, _) = pane_grid::State::new(0usize);
        (OnitamaGui { panes }, Command::none())
    }

    fn title(&self) -> String {
        String::from("Onitama")
    }

    fn update(&mut self, message: Self::Message) -> iced::Command<Self::Message> {
        match message {
            Message::Split(x) => todo!(),
        }
    }

    fn view(&self) -> iced::Element<'_, Self::Message, iced::Renderer<Self::Theme>> {
        let pane_grid = PaneGrid::new(&self.panes, |id, pane, is_maximized| {
            pane_grid::Content::new(view_content(id)).style(style::ChessBoardStyle)
        })
        .width(Length::Fill)
        .height(Length::Fill)
        .spacing(10);

        container(pane_grid)
            .width(Length::Fill)
            .height(Length::Fill)
            .padding(10)
            .into()
    }
}

fn view_content<'a>(pane: pane_grid::Pane) -> Element<'a, Message> {
    let content = column![text(format!("x")).size(24),]
        .width(Length::Fill)
        .spacing(10)
        .align_items(Alignment::Center);

    container(scrollable(content))
        .width(Length::Fill)
        .height(Length::Fill)
        .padding(5)
        .center_y()
        .into()
}

mod style {

    use iced::{widget::container, Color, Theme};

    #[derive(Default, Clone)]
    pub struct ChessBoardStyle;

    impl From<ChessBoardStyle> for iced::theme::Container {
        fn from(tuple: ChessBoardStyle) -> Self {
            iced::theme::Container::Custom(Box::new(tuple))
        }
    }

    impl container::StyleSheet for ChessBoardStyle {
        type Style = Theme;

        fn appearance(&self, _style: &Self::Style) -> container::Appearance {
            container::Appearance {
                border_color: iced::Color::BLACK,
                border_width: 10.0,
                border_radius: 0.0,
                ..Default::default()
            }
        }
    }
}

// Taken from https://github.com/adam-mcdaniel/chess-engine/blob/main/examples/chess-gui/src/lib.rs#L70
macro_rules! rgb {
    ($r:expr, $g:expr, $b:expr) => {
        iced::Color::from_rgb($r as f32 / 255.0, $g as f32 / 255.0, $b as f32 / 255.0)
    };
}
