pub mod config;
pub mod core;
pub mod logger;
#[cfg(unix)]
pub mod serial;
pub mod ui;
pub mod msg {

    include!(concat!(env!("OUT_DIR"), "/msg.rs"));
    impl MsgType {
        pub fn new(tp: msg_type::Type) -> Self {
            Self { r#type: tp as i32 }
        }
    }
    impl From<msg_type::Type> for MsgType {
        fn from(tp: msg_type::Type) -> Self {
            Self::new(tp)
        }
    }
    impl rate_list::Rate {
        pub fn new(idx: i32, state: String) -> Self {
            Self { idx, state }
        }
    }
    impl Port {
        pub fn new(path: String) -> Self {
            Self { path }
        }
    }
    impl RateList {
        pub fn new(rates: Vec<rate_list::Rate>) -> Self {
            Self { rates }
        }
    }
}
