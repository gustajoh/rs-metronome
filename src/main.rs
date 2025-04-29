use iced::Application;
mod gui;
mod metronome;

fn main() -> iced::Result {
    let settings = iced::Settings {
        antialiasing: true,
        window: iced::window::Settings {
            decorations: true,
            resizable: false,
            ..iced::window::Settings::default()
        },
        ..iced::Settings::default()
    };
    gui::MetronomeApp::run(settings)
}
