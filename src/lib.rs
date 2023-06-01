pub mod config;
pub mod core;
pub mod logger;
pub mod ui;
pub mod msg {

    include!(concat!(env!("OUT_DIR"), "/msg.rs"));
    impl MsgHeader {
        pub fn new(tp: msg_header::MsgType) -> Self {
            Self { tp: tp as i32 }
        }
    }
    impl From<msg_header::MsgType> for MsgHeader {
        fn from(tp: msg_header::MsgType) -> Self {
            Self::new(tp)
        }
    }
    impl rate_list::Rate {
        pub fn new(addr: i32, score: i32, state: rate_list::State) -> Self {
            Self {
                addr,
                score,
                state: state as i32,
            }
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
