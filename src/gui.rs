use iced::{Application, Command, executor, Element, Theme};
use iced::widget::{Button, Slider, Text, Column, Container, PickList};
use iced::alignment;
use iced::widget::canvas::{Canvas, Program, Frame, Path, Stroke, Fill};
use iced::{Color, Renderer, Point, Rectangle, Length};
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

use crate::metronome::Metronome;



#[derive(Debug, Clone)]
pub enum Message {
    BpmChanged(u32),
    ToggleMetronome,
    TimeSignatureChanged(TimeSignature),
    TickUpdate,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TimeSignature {
    pub top: u8,
    pub bottom: u8,
}

pub struct MetronomeApp {
    bpm: u32,
    time_signature: TimeSignature,
    is_running: bool,
    metronome: Option<Metronome>,
    current_beat: usize,
    shared_beat_index: Option<Arc<AtomicUsize>>,
}

pub struct BeatVisualizer {
    pub current_beat: usize,
    pub beats_in_measure: usize,
}

pub const TIME_SIGNATURE_OPTIONS: &[TimeSignature] = &[
    TimeSignature { top: 3, bottom: 4 },
    TimeSignature { top: 4, bottom: 4 },
    TimeSignature { top: 4, bottom: 8 },
    TimeSignature { top: 5, bottom: 8 },
    TimeSignature { top: 6, bottom: 8 },
    TimeSignature { top: 7, bottom: 8 },
];

impl Program<Message> for BeatVisualizer {
    type State = ();

    fn draw(
        &self, 
        _state: &Self::State,
        renderer: &Renderer,
        _theme: &Theme,
        bounds: Rectangle,
        _cursor: iced::mouse::Cursor
    ) -> Vec<iced::widget::canvas::Geometry> {
        let mut frame = Frame::new(renderer, bounds.size());
        let radius = 12.0;
        let spacing = 30.0;
        let y = bounds.height / 2.0;
        let x_start = (bounds.width - (self.beats_in_measure as f32 * spacing)) / 2.0 + spacing / 2.0;

        for i in 0..self.beats_in_measure {
            let x = x_start + i as f32 * spacing;
            let center = Point::new(x.round(), y.round());

            let path = Path::circle(center, radius);

            let color = if i == self.current_beat {
                if i == 0 {
                    Color::from_rgb(1.0, 0.2, 0.2)
                } else {
                    Color::from_rgb(1.0, 1.0, 1.0)
                }
            } else {
                Color::from_rgb(0.6, 0.6, 0.6)
            };

            frame.fill(&path, color);
            frame.stroke(
                &path,
                Stroke {
                    width: 2.0,
                    ..Default::default()
                },
            );
        }

        vec![frame.into_geometry()]
    } 

}


impl ToString for TimeSignature {
    fn to_string(&self) -> String {
        format!("{}/{}", self.top, self.bottom)
    }
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
                time_signature: TimeSignature { top: 4, bottom: 4 },
                current_beat: 0,
                shared_beat_index: None,
            },
            Command::none(),
        )
    }

    fn title(&self) -> String {
        String::from("Rs Metronome")
    }

    fn subscription(&self) -> iced::Subscription<Message> {
        use iced::time;
        use std::time::Duration;
    
        time::every(Duration::from_millis(50)).map(|_| Message::TickUpdate)
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
                    match Metronome::start(self.bpm, self.time_signature) {
                        Ok((metro, index)) => {
                            self.metronome = Some(metro);
                            self.shared_beat_index = Some(index);
                            self.is_running = true;
                        }
                        Err(e) => {
                            println!("Failed to start metronome: {:?}", e)
                        }
                    }
                }
            }
            Message::TimeSignatureChanged(new_sig) => {
                self.time_signature = new_sig;

                if self.is_running {
                    if let Some(metro) = self.metronome.take() {
                        metro.stop();
                    }
                    match Metronome::start(self.bpm, self.time_signature) {
                        Ok((metro, index)) => {
                            self.metronome = Some(metro);
                            self.shared_beat_index = Some(index);
                            self.is_running = true;
                        }
                        Err(_e) => {
                            println!("Failed to restart");
                            self.is_running = false;
                        }
                    }
                }
            }
            Message::TickUpdate => {
                if let Some(shared) = &self.shared_beat_index {
                    self.current_beat = shared.load(Ordering::Relaxed)
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
        let time_sig = PickList::new(TIME_SIGNATURE_OPTIONS, Some(self.time_signature), Message::TimeSignatureChanged);

        let beat_canvas = Canvas::new(BeatVisualizer {
            current_beat: self.current_beat,
            beats_in_measure: self.time_signature.top as usize,
        })
        .width(Length::Fill)
        .height(Length::Fixed(40.0));

        Column::new()
            .spacing(20)
            .align_items(alignment::Alignment::Center)
            .push(Text::new(format!("BPM: {}", self.bpm)).size(24))
            .push(bpm_slider)
            .push(start_stop_button)
            .push(time_sig)
            .push(beat_canvas)
            .into()
    }

    fn theme(&self) -> Self::Theme {
        Theme::Dark
    }
}
