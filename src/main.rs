use iced::{Application, Settings};
use rating::ui;
use std::sync::mpsc;
fn main() {
    rating::logger::with_file("log.txt");
    rating::logger::set_level(rating::logger::Level::Trace);
    let (tx0, rx0) = mpsc::channel();
    let (tx1, rx1) = mpsc::channel();
    std::thread::spawn(move || {
        rating::core::Core::new(tx1, rx0).run();
    });
    let _ = ui::App::run(Settings::with_flags((tx0, rx1)));
}
