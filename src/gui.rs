use iced::{Application, Command, executor, Element, Theme};
use iced::widget::{Button, Slider, Text, Column, Container};
use iced::alignment;
use crate::metronome::Metronome;

#[derive(Debug, Clone)]
pub enum Message {
    BpmChanged(u32),
    ToggleMetronome,
}

pub struct MetronomeApp {
    bpm: u32,
    is_running: bool,
    metronome: Option<Metronome>,
}

impl Application for MetronomeApp {
    type Executor = executor::Default;
    type Message = Message;
    type Theme = iced::Theme;
    type Flags = ();

    fn new(_flags: Self::Flags) -> (Self, Command<Message>) {
        (
            Self {
                bpm: 120,
                is_running: false,
                metronome: None,
            },
            Command::none(),
        )
    }

    fn title(&self) -> String {
        String::from("Rs Metronome")
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::BpmChanged(new_bpm) => {
                self.bpm = new_bpm;
            }
            Message::ToggleMetronome => {
                if self.is_running {
                    if let Some(metro) = self.metronome.take() {
                        metro.stop()
                    }
                    self.is_running = false;
                } else {
                    match Metronome::start(self.bpm) {
                        Ok(metro) => {
                            self.metronome = Some(metro);
                            self.is_running = true;
                        }
                        Err(e) => {
                            println!("Failed to start metronome: {:?}", e)
                        }
                    }
                }
            }
        }
        Command::none()
    }

    fn view(&self) -> Element<Message> {
        let bpm_slider = Slider::new(40..=240, self.bpm, Message::BpmChanged);

        let start_stop_button = if self.is_running {
            Button::new(Text::new("Stop"))
                .on_press(Message::ToggleMetronome)
                .style(iced::theme::Button::Destructive)
        } else {
            Button::new(Text::new("Start"))
                .on_press(Message::ToggleMetronome)
                .style(iced::theme::Button::Primary)
        };

        let visualizer = if self.is_running {
            Text::new("Tick").size(30)
        } else {
            Text::new("Silent").size(30)
        };

        Column::new()
            .spacing(20)
            .align_items(alignment::Alignment::Center)
            .push(Text::new(format!("BPM: {}", self.bpm)).size(24))
            .push(bpm_slider)
            .push(start_stop_button)
            .push(Container::new(visualizer).padding(10))
            .into()
    }

    fn theme(&self) -> Self::Theme {
        Theme::Dark
    }
}
