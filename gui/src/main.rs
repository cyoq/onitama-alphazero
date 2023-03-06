use iced::{
    widget::{column, Button, Column, Container, Text},
    Sandbox, Settings,
};

pub struct Hello {
    count: i32,
}

#[derive(Debug, Clone, Copy)]
pub enum CounterMessage {
    Increment,
    Decrement,
}

impl Sandbox for Hello {
    type Message = CounterMessage;

    fn new() -> Self {
        Hello { count: 0 }
    }

    fn title(&self) -> String {
        String::from("Counter App")
    }

    fn update(&mut self, message: Self::Message) {
        match message {
            CounterMessage::Increment => self.count += 1,
            CounterMessage::Decrement => self.count -= 1,
        }
    }

    fn view(&self) -> iced::Element<'_, Self::Message> {
        let label = Text::new(format!("Count: {}", self.count));
        let incr = Button::new("Increment").on_press(CounterMessage::Increment);
        let decr = Button::new("Decrement").on_press(CounterMessage::Decrement);
        let col = Column::new().push(incr).push(label).push(decr);
        Container::new(col)
            .center_x()
            .center_y()
            .width(iced::Length::Fill)
            .height(iced::Length::Fill)
            .into()
    }
}

fn main() -> iced::Result {
    env_logger::builder().format_timestamp(None).init();

    Hello::run(Settings::default())

    // GameOfLife::run(Settings {
    //     antialiasing: true,
    //     window: window::Settings {
    //         position: window::Position::Centered,
    //         ..window::Settings::default()
    //     },
    //     ..Settings::default()
    // })
}
