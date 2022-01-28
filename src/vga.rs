use core::fmt;
use lazy_static::lazy_static;
use spin::Mutex;
use volatile::Volatile;
use x86_64::instructions::port::PortWrite;

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::vga::_print(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}

#[doc(hidden)]
pub fn _print(args: fmt::Arguments) {
    use core::fmt::Write;
    WRITER.lock().write_fmt(args).unwrap();
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Color {
    Black = 0,
    Blue = 1,
    Green = 2,
    Cyan = 3,
    Red = 4,
    Magenta = 5,
    Brown = 6,
    LightGray = 7,
    DarkGray = 8,
    LightBlue = 9,
    LightGreen = 10,
    LightCyan = 11,
    LightRed = 12,
    Pink = 13,
    Yellow = 14,
    White = 15,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
struct ColorCode(u8);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
struct ScreenChar {
    ascii_character: u8,
    color_code: ColorCode,
}

#[repr(transparent)]
struct Buffer {
    chars: [[Volatile<ScreenChar>; BUFFER_WIDTH]; BUFFER_HEIGHT],
}

pub struct Writer {
    column_position: usize,
    color_code: ColorCode,
    buffer: &'static mut Buffer,
}

lazy_static! {
    pub static ref WRITER: Mutex<Writer> = Mutex::new(Writer {
        column_position: 0,
        color_code: ColorCode::new(Color::Green, Color::Black),
        buffer: unsafe { &mut *(0xb8000 as *mut Buffer) },
    });
}

const BUFFER_HEIGHT: usize = 25;
const BUFFER_WIDTH: usize = 80;

impl ColorCode {
    fn new(foreground: Color, background: Color) -> ColorCode {
        ColorCode((background as u8) << 4 | (foreground as u8))
    }
}

impl Writer {
    pub fn write_byte(&mut self, byte: u8) {
        let row = BUFFER_HEIGHT - 1;
        match byte {
            b'\n' => self.column_position = BUFFER_WIDTH,
            byte => {
                if self.column_position >= BUFFER_WIDTH {
                    self.new_line();
                }

                let col = self.column_position;

                let color_code = self.color_code;
                self.buffer.chars[row][col].write(ScreenChar {
                    ascii_character: byte,
                    color_code,
                });
                self.column_position += 1;
            }
        }
        self.set_cursor();
    }

    fn new_line(&mut self) {
        for row in 1..BUFFER_HEIGHT {
            for col in 0..BUFFER_WIDTH {
                let character = self.buffer.chars[row][col].read();
                self.buffer.chars[row - 1][col].write(character);
            }
        }
        let blank = ScreenChar {
            ascii_character: b' ',
            color_code: self.color_code,
        };
        for col in 0..BUFFER_WIDTH {
            self.buffer.chars[BUFFER_HEIGHT - 1][col].write(blank);
        }
        self.column_position = 0;
    }

    fn set_cursor(&self) {
        let row = BUFFER_HEIGHT - 1;
        let index_port = 0x3d4;
        let data_port = 0x3d5;
        let pos = row * BUFFER_WIDTH + self.column_position;
        let low: u8 = (pos & 0xff).try_into().unwrap();
        let high: u8 = (pos >> 8).try_into().unwrap();
        unsafe {
            // Select index 14 / CURSOR_HIGH register
            u8::write_to_port(index_port, 14);
            // Write high part of value to selected register (cursor high)
            u8::write_to_port(data_port, high);

            // Select index 15 / CURSOR_LOW register
            u8::write_to_port(index_port, 15);
            // Write low part of value to selected register (cursor low)
            u8::write_to_port(data_port, low);
        }
    }

    pub fn write_string(&mut self, s: &str) {
        for byte in s.bytes() {
            self.write_byte(byte);
            // match byte {
            //     // printable ASCII byte or newline
            //     0x20..=0x7e | b'\n' => self.write_byte(byte),
            //     // not part of printable ASCII range
            //     _ => self.write_byte(0xfe),
            // }
        }
    }
}

impl fmt::Write for Writer {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.write_string(s);
        Ok(())
    }
}
