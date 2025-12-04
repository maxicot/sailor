use spin::{Mutex, Once};
use bootloader_api::info::FrameBuffer;
use framebuffer::Writer;
pub use framebuffer::Color;

pub(crate) static FRAMEBUFFER: Once<Mutex<Writer>> = Once::new();

pub(crate) fn init(framebuffer: &'static mut FrameBuffer, color: Color) {
    let info = framebuffer.info();

    let mut framebuffer = FRAMEBUFFER
        .call_once(|| Mutex::new(Writer::new(framebuffer.buffer_mut(), info)))
        .lock();

    framebuffer.clear();
    framebuffer.set_color(color);
}

#[allow(unused_macros)]
macro_rules! print {
    () => {};

    (color = $color:expr, $($t:tt)+) => {{
        let _ = $crate::terminal::FRAMEBUFFER
            .get()
            .map_or((), |f| {
                f.lock().write_fmt_colored($color, format_args!($($t)+)).unwrap();
            });
    }};

    ($($t:tt)+) => {{
        let _ = $crate::terminal::FRAMEBUFFER
            .get()
            .map_or((), |f| {
                f.lock().write_fmt(format_args!($($t)+)).unwrap();
            });
    }};
}

#[allow(unused_macros)]
macro_rules! println {
    () => {{
        let _ = $crate::terminal::FRAMEBUFFER
            .get()
            .map_or((), |f| {
                f.lock().write_char('\n').unwrap();
            });
    }};

    (color = $color:expr, $($t:tt)+) => {{
        let _ = $crate::terminal::FRAMEBUFFER
            .get()
            .map_or((), |f| {
                let mut lock = f.lock();

                lock.write_fmt_colored($color, format_args!($($t)+)).unwrap();
                lock.newline();
            });
    }};

    ($($t:tt)+) => {{
        let _ = $crate::terminal::FRAMEBUFFER
            .get()
            .map_or((), |f| {
                let mut lock = f.lock();

                lock.write_fmt(format_args!($($t)+)).unwrap();
                lock.newline();
            });
    }};
}

mod framebuffer {
    use bootloader_api::info::{FrameBufferInfo, PixelFormat};
    use core::fmt::{self, Write};

    const LINE_SPACING: usize = 2;
    const BORDER_PADDING: usize = 6;
    const CHAR_HEIGHT: usize = 16;
    const DEFAULT_CHAR: char = '�';

    /// Structure representing an RGBA color.
    #[derive(Clone, Copy, Debug)]
    pub struct Color {
        pub r: u8,
        pub g: u8,
        pub b: u8,
        pub a: u8
    }

    impl Color {
        pub const BLACK: Self = Self::new(0, 0, 0, 0);
        pub const WHITE: Self = Self::new(255, 255, 255, 0);
        pub const RED: Self = Self::new(255, 0, 0, 0);
        pub const GREEN: Self = Self::new(0, 255, 0, 0);
        pub const BLUE: Self = Self::new(0, 0, 255, 0);
        pub const CYAN: Self = Self::new(0, 255, 255, 0);
        pub const YELLOW: Self = Self::new(255, 255, 0, 0);
        pub const MAGENTA: Self = Self::new(255, 0, 255, 0);
        pub const INDIGO: Self = Self::new(75, 0, 130, 0);
        pub const PEACH: Self = Self::new(255, 218, 185, 0);
        pub const GREY: Self = Self::new(128, 128, 128, 0);
        pub const ORANGE: Self = Self::new(255, 140, 0, 0);

        pub const fn new(r: u8, g: u8, b: u8, a: u8) -> Self {
            Self {r, g, b, a}
        }
    }

    pub struct Writer {
        framebuffer: &'static mut [u8],
        info: FrameBufferInfo,
        x: usize,
        y: usize,
        color: Color
    }

    impl Writer {
        pub fn new(framebuffer: &'static mut [u8], info: FrameBufferInfo) -> Self {
            Self {
                framebuffer,
                info,
                x: 0,
                y: 0,
                color: Color::WHITE
            }
        }

        pub fn newline(&mut self) {
            let new_y = self.y + CHAR_HEIGHT + LINE_SPACING;

            if new_y >= self.height() {
                self.clear();
                self.y = 0;
            } else {
                self.y = new_y;
            }

            self.carriage_return();
        }

        pub fn carriage_return(&mut self) {
            self.x = BORDER_PADDING;
        }

        pub fn clear(&mut self) {
            self.x = BORDER_PADDING;
            self.y = BORDER_PADDING;
            self.framebuffer.fill(0);
        }

        fn width(&self) -> usize {
            self.info.width
        }

        fn height(&self) -> usize {
            self.info.height
        }

        pub fn write_unifont(&mut self, c: char) {
            match c {
                '\n' => self.newline(),
                '\r' => self.carriage_return(),
                '\0' => {
                    for y in 0..CHAR_HEIGHT {
                        for x in 0..8 {
                            self.write_pixel_colored(self.x + x, self.y + y, Color::BLACK);
                        }
                    }
                },
                _ => {
                    let glyph = unifont::get_glyph(c)
                        .unwrap_or_else(|| unifont::get_glyph(DEFAULT_CHAR).unwrap());
                    let width = glyph.get_width();

                    for y in 0..CHAR_HEIGHT {
                        for x in 0..width {
                            if glyph.get_pixel(x, y) {
                                self.write_pixel(self.x + x, self.y + y);
                            }
                        }
                    }

                    let new_x = self.x + width;

                    if new_x >= self.width() {
                        self.newline();
                    } else {
                        self.x = new_x;
                    }
                }
            }
        }

        pub fn write_pixel(&mut self, x: usize, y: usize) {
            self.write_pixel_colored(x, y, self.color);
        }

        pub fn write_pixel_colored(&mut self, x: usize, y: usize, color: Color) {
            let Color {r, g, b, a} = color;

            let color = match self.info.pixel_format {
                PixelFormat::Rgb => [r, g, b, a],
                PixelFormat::Bgr => [b, g, r, a],
                PixelFormat::U8 => [if r > 200 { 0xf } else { 0 }, 0, 0, 0],
                other => {
                    // fall back to a supported format
                    self.info.pixel_format = PixelFormat::Rgb;
                    println!(color = Color::RED, "Unsupported pixel format: {:?}", other);
                    return;
                }
            };

            let pixel_offset = y * self.info.stride + x;
            let bytes_per_pixel = self.info.bytes_per_pixel;
            let byte_offset = pixel_offset * bytes_per_pixel;

            self.framebuffer[byte_offset..(byte_offset + bytes_per_pixel)]
                .copy_from_slice(&color[..bytes_per_pixel]);
        }

        pub fn get_color(&self) -> Color {
            self.color
        }

        pub fn set_color(&mut self, color: Color) {
            self.color = color;
        }

        pub fn write_fmt_colored(&mut self, color: Color, args: fmt::Arguments<'_>) -> fmt::Result {
            let last_color = self.get_color();
            self.set_color(color);
            let res = self.write_fmt(args);
            self.set_color(last_color);
            res
        }
    }

    impl Write for Writer {
        fn write_str(&mut self, s: &str) -> fmt::Result {
            for c in s.chars() {
                self.write_unifont(c)
            }

            Ok(())
        }
    }
}
