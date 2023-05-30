struct Logger {
    to: Box<dyn std::io::Write>,
}
static mut LOGGER: Option<Logger> = None;

impl Logger {
    pub fn log(&mut self, s: &str) {
        let _ = self.to.write(s.as_bytes());
        // self.to.write(b"\n").unwrap();
    }
}
pub fn init(to: impl std::io::Write + 'static) {
    // Box::new(to);
    unsafe {
        LOGGER = Some(Logger { to: Box::new(to) });
    }
}
pub fn info(s: &str) {
    unsafe {
        if let Some(logger) = LOGGER.as_mut() {
            logger.log(s);
        }
    }
}
