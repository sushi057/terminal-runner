use std::io::{self, Write};
use std::os::fd::AsRawFd;

#[derive(Clone, Copy, PartialEq)]
enum Cell {
    Empty,
    Char(char),
    Colored(char, Color),
}

impl Cell {
    fn ch(&self) -> char {
        match self {
            Cell::Empty => ' ',
            Cell::Char(c) | Cell::Colored(c, _) => *c,
        }
    }
}

#[derive(Clone, Copy, PartialEq)]
#[allow(dead_code)]
pub enum Color {
    Green,
    Brown,
    Cyan,
    Gray,
    White,
    Yellow,
    Red,
    Blue,
}

pub struct Terminal {
    width: u16,
    height: u16,
    stdin_fd: i32,
    orig_termios: libc::termios,
    prev_buffer: Vec<Vec<Cell>>,
    current_buffer: Vec<Vec<Cell>>,
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

        print!("\x1b[?1049h\x1b[?25l\x1b[2J");
        let _ = io::stdout().flush();

        let (w, h) = Self::terminal_size();
        let (w, h) = (w.max(40), h.max(10));

        Self {
            width: w,
            height: h,
            stdin_fd,
            orig_termios,
            prev_buffer: vec![vec![Cell::Empty; w as usize]; h as usize],
            current_buffer: vec![vec![Cell::Empty; w as usize]; h as usize],
        }
    }

    pub fn width(&self) -> u16 { self.width }
    pub fn height(&self) -> u16 { self.height }

    pub fn poll_key(&self) -> Option<Key> {
        let mut pollfd = libc::pollfd { fd: self.stdin_fd, events: libc::POLLIN, revents: 0 };
        let ret = unsafe { libc::poll(&mut pollfd, 1, 0) };
        if ret <= 0 { return None; }
        let mut buf = [0u8; 16];
        let n = unsafe { libc::read(self.stdin_fd, buf.as_mut_ptr() as *mut libc::c_void, buf.len()) };
        if n <= 0 { return None; }
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
            row.fill(Cell::Empty);
        }
    }

    pub fn set_cell(&mut self, x: u16, y: u16, ch: char) {
        if (x as usize) < self.width as usize && (y as usize) < self.height as usize {
            self.current_buffer[y as usize][x as usize] = Cell::Char(ch);
        }
    }

    pub fn set_colored(&mut self, x: u16, y: u16, ch: char, color: Color) {
        if (x as usize) < self.width as usize && (y as usize) < self.height as usize {
            self.current_buffer[y as usize][x as usize] = Cell::Colored(ch, color);
        }
    }

    pub fn set_str(&mut self, x: u16, y: u16, s: &str) {
        for (i, ch) in s.chars().enumerate() {
            self.set_cell(x + i as u16, y, ch);
        }
    }

    fn emit_sgr(color: Option<Color>) -> String {
        match color {
            None => "\x1b[0m".to_string(),
            Some(Color::Green) => "\x1b[32m".to_string(),
            Some(Color::Brown) => "\x1b[33m".to_string(),
            Some(Color::Yellow) => "\x1b[93m".to_string(),
            Some(Color::Cyan) => "\x1b[36m".to_string(),
            Some(Color::Gray) => "\x1b[90m".to_string(),
            Some(Color::White) => "\x1b[97m".to_string(),
            Some(Color::Red) => "\x1b[31m".to_string(),
            Some(Color::Blue) => "\x1b[34m".to_string(),
        }
    }

    pub fn flush(&mut self) {
        let mut out = String::with_capacity(8192);
        let mut current_color: Option<Color> = None;

        for y in 0..self.height as usize {
            let mut need_color_reset = false;
            for x in 0..self.width as usize {
                if self.current_buffer[y][x] == self.prev_buffer[y][x] {
                    continue;
                }
                let cell = &self.current_buffer[y][x];
                let cell_color = match cell {
                    Cell::Empty | Cell::Char(_) => None,
                    Cell::Colored(_, c) => Some(*c),
                };

                if cell_color != current_color {
                    out.push_str(&Self::emit_sgr(cell_color));
                    current_color = cell_color;
                    if cell_color.is_some() {
                        need_color_reset = true;
                    }
                }

                out.push_str(&format!("\x1b[{};{}H", y + 1, x + 1));
                out.push(cell.ch());
            }
            // Reset color at end of row if we used any color this row
            if need_color_reset && current_color.is_some() {
                out.push_str("\x1b[0m");
                current_color = None;
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
        unsafe { libc::tcsetattr(self.stdin_fd, libc::TCSANOW, &self.orig_termios); }
        print!("\x1b[?1049l\x1b[?25h");
        let _ = io::stdout().flush();
    }
}
