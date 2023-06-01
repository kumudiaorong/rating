// pub struct Serial {
//     dfd: i32,
// }
// #[derive(Debug, Clone)]
// pub enum Error {
//     OpenFail(String),
//     SetFail(String),
//     Convert(String),
// }
// fn i32tobaud(i: i32) -> Result<libc::speed_t, Error> {
//     Ok(match i {
//         0 => libc::B0,
//         50 => libc::B50,
//         75 => libc::B75,
//         110 => libc::B110,
//         134 => libc::B134,
//         150 => libc::B150,
//         200 => libc::B200,
//         300 => libc::B300,
//         600 => libc::B600,
//         1200 => libc::B1200,
//         1800 => libc::B1800,
//         2400 => libc::B2400,
//         4800 => libc::B4800,
//         9600 => libc::B9600,
//         19200 => libc::B19200,
//         38400 => libc::B38400,
//         57600 => libc::B57600,
//         115200 => libc::B115200,
//         230400 => libc::B230400,
//         460800 => libc::B460800,
//         500000 => libc::B500000,
//         576000 => libc::B576000,
//         921600 => libc::B921600,
//         1000000 => libc::B1000000,
//         1152000 => libc::B1152000,
//         1500000 => libc::B1500000,
//         2000000 => libc::B2000000,
//         2500000 => libc::B2500000,
//         3000000 => libc::B3000000,
//         3500000 => libc::B3500000,
//         4000000 => libc::B4000000,
//         _ => return Err(Error::Convert("Invalid baud rate".to_string())),
//     })
// }

// impl Serial {
//     pub fn new(name: &str, baud: i32) -> Result<Serial, Error> {
//         Ok(Self {
//             dfd: unsafe {
//                 // let name = std::ffi::CString::new(name).unwrap();
//                 let c = name.clone();
//                 let fd = libc::open(
//                     format!("{}\0", name).as_ptr() as *const i8,
//                     libc::O_RDWR | libc::O_NOCTTY | libc::O_NDELAY,
//                 );
//                 if fd == -1 {
//                     return Err(Error::OpenFail("Failed to open serial port".to_string()));
//                 }
//                 let mut opt: libc::termios = { std::mem::zeroed() };
//                 libc::tcgetattr(fd, &mut opt);
//                 match i32tobaud(baud) {
//                     Ok(baud) => {
//                         libc::cfsetispeed(&mut opt, baud);
//                         libc::cfsetospeed(&mut opt, baud);
//                     }
//                     Err(e) => return Err(e),
//                 }
//                 opt.c_lflag &= !(libc::ICANON | libc::ECHO | libc::ISIG | libc::IEXTEN);
//                 opt.c_iflag &=
//                     !(libc::BRKINT | libc::ICRNL | libc::INPCK | libc::ISTRIP | libc::IXON);
//                 opt.c_oflag &= !(libc::OPOST);
//                 opt.c_cflag &= !(libc::CSIZE | libc::PARENB);
//                 opt.c_cflag |= libc::CS8;
//                 opt.c_cc[libc::VMIN] = 0xff;
//                 opt.c_cc[libc::VTIME] = 150;
//                 if libc::tcsetattr(fd, libc::TCSANOW, &opt) != 0 {
//                     libc::close(fd);
//                     return Err(Error::SetFail(
//                         "Failed to set attributes of serial port".to_string(),
//                     ));
//                 }
//                 fd
//             },
//         })
//     }
//     pub fn read(&self, buf: &mut [u8]) -> Option<Vec<u8>> {
//         if self.dfd == -1 {
//             panic!("Serial port is not open");
//         }
//         unsafe {
//             let mut pfd = libc::pollfd {
//                 fd: self.dfd,
//                 events: libc::POLLIN,
//                 revents: 0,
//             };
//             let mut all = 0;
//             loop {
//                 let n = libc::poll(&mut pfd, 1, 20);
//                 if n == -1 {
//                     panic!("Failed to poll serial port");
//                 }
//                 if n == 0 {
//                     return None;
//                 }
//                 if (pfd.revents & libc::POLLERR != 0)
//                     || (pfd.revents & libc::POLLHUP != 0)
//                     || (pfd.revents & libc::POLLNVAL != 0)
//                 {
//                     panic!("Error on serial port");
//                 }
//                 if pfd.revents & libc::POLLIN != 0 {
//                     let c = libc::read(
//                         self.dfd,
//                         buf[all..].as_mut_ptr() as *mut libc::c_void,
//                         buf.len(),
//                     );
//                     if c == -1 {
//                         panic!("Failed to read serial port");
//                     }
//                     if c == 0 {
//                         return None;
//                     }
//                     all += c as usize;
//                 }
//                 if all == buf.len() {
//                     return Some(buf.to_vec());
//                 }
//             }
//         }
//     }
//     pub fn write(&self, buf: &[u8]) -> isize {
//         if self.dfd == -1 {
//             panic!("Serial port is not open");
//         }
//         unsafe { libc::write(self.dfd, buf.as_ptr() as *const libc::c_void, buf.len()) }
//     }
// }
// impl Drop for Serial {
//     fn drop(&mut self) {
//         if self.dfd != -1 {
//             unsafe {
//                 libc::close(self.dfd);
//             }
//         }
//     }
// }
