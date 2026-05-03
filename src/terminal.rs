use std::io::{self, Write};
use std::os::fd::AsRawFd;

pub struct Terminal {
    width: u16,
    height: u16,
    stdin_fd: i32,
    orig_termios: libc::termios,
    prev_buffer: Vec<Vec<char>>,
    current_buffer: Vec<Vec<char>>,
}

#[derive(Clone, Copy, PartialEq)]
pub enum Key {
    Space,
    Up,
    Quit,
    Unknown,
}

impl Terminal {
    pub fn init() -> Self {
        let stdin_fd = io::stdin().as_raw_fd();
        let orig_termios = unsafe {
            let mut termios: libc::termios = std::mem::zeroed();
            libc::tcgetattr(stdin_fd, &mut termios);
            termios
        };

        let mut raw = orig_termios;
        unsafe {
            libc::cfmakeraw(&mut raw);
            libc::tcsetattr(stdin_fd, libc::TCSANOW, &raw);
        }

        // Enter alternate screen, hide cursor, clear
        print!("\x1b[?1049h\x1b[?25l\x1b[2J");
        let _ = io::stdout().flush();

        let (w, h) = Self::terminal_size();
        let (w, h) = (w.max(40), h.max(10));

        Self {
            width: w,
            height: h,
            stdin_fd,
            orig_termios,
            prev_buffer: vec![vec![' '; w as usize]; h as usize],
            current_buffer: vec![vec![' '; w as usize]; h as usize],
        }
    }

    pub fn width(&self) -> u16 {
        self.width
    }

    pub fn height(&self) -> u16 {
        self.height
    }

    pub fn poll_key(&self) -> Option<Key> {
        let mut pollfd = libc::pollfd {
            fd: self.stdin_fd,
            events: libc::POLLIN,
            revents: 0,
        };
        let ret = unsafe { libc::poll(&mut pollfd, 1, 0) };
        if ret <= 0 {
            return None;
        }
        let mut buf = [0u8; 16];
        let n = unsafe { libc::read(self.stdin_fd, buf.as_mut_ptr() as *mut libc::c_void, buf.len()) };
        if n <= 0 {
            return None;
        }
        match &buf[..n as usize] {
            b"\x1b" => Some(Key::Quit),
            b" " => Some(Key::Space),
            b"\x1b[A" | b"w" | b"k" => Some(Key::Up),
            b"q" => Some(Key::Quit),
            _ => Some(Key::Unknown),
        }
    }

    pub fn clear(&mut self) {
        for row in &mut self.current_buffer {
            row.fill(' ');
        }
    }

    pub fn set_cell(&mut self, x: u16, y: u16, ch: char) {
        let x = x as usize;
        let y = y as usize;
        if x < self.width as usize && y < self.height as usize {
            self.current_buffer[y][x] = ch;
        }
    }

    pub fn set_str(&mut self, x: u16, y: u16, s: &str) {
        for (i, ch) in s.chars().enumerate() {
            self.set_cell(x + i as u16, y, ch);
        }
    }

    pub fn flush(&mut self) {
        let mut out = String::with_capacity(4096);
        for y in 0..self.height as usize {
            for x in 0..self.width as usize {
                if self.current_buffer[y][x] != self.prev_buffer[y][x] {
                    out.push_str(&format!("\x1b[{};{}H", y + 1, x + 1));
                    out.push(self.current_buffer[y][x]);
                }
            }
        }
        out.push_str("\x1b[?25l");
        let _ = io::stdout().write_all(out.as_bytes());
        let _ = io::stdout().flush();
        std::mem::swap(&mut self.current_buffer, &mut self.prev_buffer);
    }

    fn terminal_size() -> (u16, u16) {
        let mut ws: libc::winsize = unsafe { std::mem::zeroed() };
        let ret = unsafe { libc::ioctl(0, libc::TIOCGWINSZ, &mut ws) };
        if ret == 0 && ws.ws_col > 0 && ws.ws_row > 0 {
            (ws.ws_col, ws.ws_row)
        } else {
            (80, 24)
        }
    }
}

impl Drop for Terminal {
    fn drop(&mut self) {
        unsafe {
            libc::tcsetattr(self.stdin_fd, libc::TCSANOW, &self.orig_termios);
        }
        // Exit alternate screen, show cursor
        print!("\x1b[?1049l\x1b[?25h");
        let _ = io::stdout().flush();
    }
}
