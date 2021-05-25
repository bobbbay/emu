use minifb::{Key, KeyRepeat, Scale, Window, WindowOptions};
use std::process::exit;

#[derive(Debug)]
pub struct PPU {
    pub buffer: Vec<u32>,
    pub window: Window,
    pub width: usize,
    pub height: usize,
}

impl PPU {
    pub fn new() -> Self {
        let width = 32;
        let height = 32;

        let window = Window::new(
            "MaxEmu 2021",
            width,
            height,
            WindowOptions {
                scale: Scale::X16,
                ..WindowOptions::default()
            },
        )
        .unwrap();

        Self {
            buffer: vec![0; width * height],
            window,
            width,
            height,
        }
    }

    pub fn update_keys(&self, mut memory: [u8; 0xFFFF]) -> [u8; 0xFFFF] {
        let keys = self.window.get_keys_pressed(KeyRepeat::No);
        keys.map(|keys| {
            for t in keys {
                match t {
                    Key::W => memory[0x0100] = 1,
                    Key::A => memory[0x0100] = 2,
                    Key::S => memory[0x0100] = 3,
                    Key::D => memory[0x0100] = 4,
                    _ => memory[0x0100] = 0,
                }
            }
        });
        memory
    }

    pub fn render(&mut self, memory: [u8; 0xFFFF]) {
        for (i, j) in self.buffer.iter_mut().enumerate() {
            *j = (memory[0x0200 + i] as u32).pow(4);
        }

        if !self.window.is_key_down(minifb::Key::Escape) {
            self.window
                .update_with_buffer(&*self.buffer, self.width, self.height)
                .unwrap();
        } else {
            exit(0);
        }
    }
}
