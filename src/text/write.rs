use std::ops::{Index, IndexMut};

use cosmic_text::{
    Align, Attrs, AttrsList, Buffer, BufferLine, Color, Family, FontSystem, LineEnding, Metrics,
    Shaping, SwashCache,
};

pub struct Canvas<'a> {
    canvas_buffer: &'a mut [u8],
    height: u32,
    width: u32,
    stride: u32,
}

impl<'a> Canvas<'a> {
    pub fn new(canvas_buffer: &'a mut [u8], width: u32, height: u32) -> Self {
        Self {
            canvas_buffer,
            height,
            width,
            stride: width * 4,
        }
    }
    pub fn get_pixel(&'a mut self, row: u32, column: u32) -> &'a mut [u8] {
        &mut self[row][column as usize * 4..((column * 4) + 1) as usize]
    }

    pub fn set_pixel_color(&mut self, color: Color, x: i32, y: i32) {
        let stride = self.stride;

        let line = stride as i32 * y;
        let chunk_start = (x * 4) + line;
        if chunk_start as usize > self.canvas_buffer.len() - 4 {
            return;
        }
        let slice: &mut [u8] =
            &mut self.canvas_buffer[chunk_start as usize..(chunk_start + 4) as usize];

        // Alpha Blending
        let text_a = color.a();
        let mut text_r = color.r();
        let mut text_g = color.g();
        let mut text_b = color.b();

        let bg_r = slice[2];
        let bg_g = slice[1];
        let bg_b = slice[0];

        let alpha_percent = text_a as f32 / 255.0;

        text_r = ((bg_r as f32 * (1.0 - alpha_percent))
            + (text_r as f32 * alpha_percent).clamp(0.0, 255.0)) as u8;
        text_g = ((bg_g as f32 * (1.0 - alpha_percent))
            + (text_g as f32 * alpha_percent).clamp(0.0, 255.0)) as u8;
        text_b = ((bg_b as f32 * (1.0 - alpha_percent))
            + (text_b as f32 * alpha_percent).clamp(0.0, 255.0)) as u8;

        slice[2] = text_r;
        slice[1] = text_g;
        slice[0] = text_b;
    }

    pub fn set_background(&mut self, color: Color) -> &mut Self {
        self.canvas_buffer
            .chunks_exact_mut(4)
            .enumerate()
            .for_each(|(index, chunk)| {
                let _x = (index % self.width as usize) as u32;
                let _y = (index / self.width as usize) as u32;

                let array: &mut [u8; 4] = chunk.try_into().unwrap();
                *array = [color.b(), color.g(), color.r(), color.a()]; // Little Endian
            });

        self
    }

    pub fn write_text(&mut self, text: &str, align: Align) {
        let Canvas { width, height, .. } = *self;
        const TEXT_COLOR: Color = Color::rgb(0xFF, 0xFF, 0xFF);
        let mut font_system = FontSystem::new();
        let mut swash_cache = SwashCache::new();

        let font_size: f32 = height as f32 * 0.8;
        let line_height: f32 = font_size * 1.2;

        let metrics = Metrics::new(font_size, line_height);
        let mut buffer = Buffer::new_empty(metrics);

        let mut buffer = buffer.borrow_with(&mut font_system);

        buffer.set_size(Some(width as f32), Some(height as f32));

        let attrs = Attrs::new();
        let attrs = attrs.family(Family::Name("JetBrainsMono Nerd Font Mono"));
        let attrs_list = AttrsList::new(attrs);

        let mut bufferline = BufferLine::new(text, LineEnding::None, attrs_list, Shaping::Advanced);
        bufferline.set_align(Some(align));
        buffer.lines.push(bufferline);
        buffer.shape_until_scroll(true);

        buffer.draw(&mut swash_cache, TEXT_COLOR, |x, y, w, h, color| {
            if color.a() == 0
                || x < 0
                || x >= width as i32
                || y < 0
                || y >= height as i32
                || w != 1
                || h != 1
            {
                return;
            }
            self.set_pixel_color(color, x, y)
        });
    }
}

impl<'a> Index<u32> for Canvas<'a> {
    type Output = [u8];

    fn index(&self, index: u32) -> &Self::Output {
        &self.canvas_buffer[(index * self.stride) as usize..((index + 1) * self.stride) as usize]
    }
}

impl<'a> IndexMut<u32> for Canvas<'a> {
    fn index_mut(&mut self, index: u32) -> &mut Self::Output {
        &mut self.canvas_buffer
            [(index * self.stride) as usize..((index + 1) * self.stride) as usize]
    }
}
