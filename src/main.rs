use iced::{Application, Settings};
use rating::ui;
// void set_right(string &str, int id) {
//     str = {0x5A, (char)id, 0x08, 0x13, (char)(0x5A + id + 0x08 + 0x13)};
//     // 5A 01 08 00 63
//   }
//   void set_query(string &str, int id) {
//     str = {0x5A, 0, 0x03, (char)id, (char)(0x5A + 0x03 + id)};
//     // 5A 01 08 13 76
//   }
//   // 5A 00 03 01 5E
//   bool is_error(int id) {
//     char ebuf[] = {0x5A, (char)id, 0x03, 0x6F, (char)(0x5A + id + 0x03 + 0x6F)};
//     return !strncmp((char *)readbuf, ebuf, 5);
//   }
use std::env;
use std::io::{self, Write};
use std::sync::mpsc;
fn main() {
    rating::logger::init(
        std::fs::File::options()
            .create(true)
            .append(true)
            .open("log.txt")
            .unwrap(),
    );
    rating::logger::set_level(rating::logger::Level::Trace);
    let (tx0, rx0) = mpsc::channel();
    let (tx1, rx1) = mpsc::channel();
    let mut core = rating::core::Core::new(tx1, rx0);

    std::thread::spawn(move || {
        core.run();
    });

    ui::App::run(Settings::with_flags((tx0, rx1)));
    // Counter::run(Settings::default());
    // let ava_ports = serialport::available_ports().expect("Failed to get available ports");

    // let ser = serialport::new("/dev/ttyUSB0", libc::B9600);
    // for i in 0..10 {
    //     let mut str: Vec<u8> = vec![i, i + 1, i + 2, i + 3, i + 4];
    //     ser.write(&str);
    //     ser.read(str[0..5].as_mut());
    //     println!("{:?}", str);
    // }
}
