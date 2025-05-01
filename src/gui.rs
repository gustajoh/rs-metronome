use iced::{Application, Command, executor, Element, Theme};
use iced::widget::{Button, Slider, Text, Column, Container, PickList};
use iced::alignment;
use iced::widget::canvas::{Canvas, Program, Frame, Path, Stroke, Fill};
use iced::{Color, Renderer, Point, Rectangle, Length};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use atomic_float::AtomicF32;

use crate::metronome::Metronome;



#[derive(Debug, Clone)]
pub enum Message {
    BpmChanged(u32),
    ToggleMetronome,
    TimeSignatureChanged(TimeSignature),
    TickUpdate,
    VolumeChanged(i32),
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
    is_active: bool,
    shared_beat_index: Option<Arc<AtomicUsize>>,
    shared_tick_active: Option<Arc<AtomicBool>>,
    volume: Arc<AtomicF32>,
}

pub struct BeatVisualizer {
    pub current_beat: usize,
    pub is_active: bool,
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
        let y = bounds.height / 2.0;
        let x = bounds.width / 2.0;

        let center = Point::new(x.round(), y.round());

        let path = Path::circle(center, radius);
        let color = if self.is_active {
            if self.current_beat == 0 {
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
                is_active: false,
                shared_beat_index: None,
                shared_tick_active: None,
                volume: Arc::new(AtomicF32::new(0.5)),
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
                    match Metronome::start(self.bpm, self.time_signature, self.volume.clone()) {
                        Ok((metro, index, tick_active)) => {
                            self.metronome = Some(metro);
                            self.shared_beat_index = Some(index);
                            self.shared_tick_active = Some(tick_active);
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
                    match Metronome::start(self.bpm, self.time_signature, self.volume.clone()) {
                        Ok((metro, index, tick_active)) => {
                            self.metronome = Some(metro);
                            self.shared_beat_index = Some(index);
                            self.shared_tick_active = Some(tick_active);
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
                    self.current_beat = shared.load(Ordering::Relaxed);
                }
                if let Some(shared) = &self.shared_tick_active {
                    self.is_active = shared.load(Ordering::Relaxed);
                }
            }
            Message::VolumeChanged(v) => {
                self.volume.store(v as f32 / 100.0, Ordering::Relaxed);
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
            is_active: self.is_active,
        })
        .width(Length::Fill)
        .height(Length::Fixed(40.0));

        let volume_slider = Slider::new(0..=100, (self.volume.load(Ordering::Relaxed) * 100.0) as i32, Message::VolumeChanged)
        .width(Length::Fixed(200.0));

        Column::new()
            .spacing(20)
            .align_items(alignment::Alignment::Center)
            .push(Text::new(format!("BPM: {}", self.bpm)).size(24))
            .push(bpm_slider)
            .push(start_stop_button)
            .push(time_sig)
            .push(beat_canvas)
            .push(Text::new(format!("Volume: {}%", (self.volume.load(Ordering::Relaxed) * 100.0) as u32)))
            .push(volume_slider)            
            .into()
    }

    fn theme(&self) -> Self::Theme {
        Theme::Dark
    }
}
