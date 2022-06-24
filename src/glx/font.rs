//! Font helper objects and functions.

use crate::glx;
use crate::glx::objects::*;
use crate::glx::types::*;

use bmfont_rs::Char;
use bmfont_rs::Font;
use gl::types::*;
use image::imageops;
use image::{GrayImage, ImageFormat};

use std::error::Error;
use std::fs::{self, File};
use std::io::BufReader;
use std::mem;
use std::path::Path;
use std::ptr;
use std::result::Result;

pub struct FontMonkey {
    buffer: Vec<Blit>,
    chars: [CharLite; 128],
    cap: usize,
    vao: VertexArrayObject,
    vbo: BufferObject,
    texture: TextureObject,
}

impl FontMonkey {
    pub fn load(
        folder: impl AsRef<Path>,
        font: impl AsRef<Path>,
        screen_width: u32,
        screen_height: u32,
        cap: usize,
    ) -> Result<Self, Box<dyn Error>> {
        let folder: &Path = folder.as_ref();
        let font: &Path = font.as_ref();

        let rdr = File::open(folder.join(font))?;
        let font = bmfont_rs::text::from_reader(rdr)?;

        if font.pages.len() != 1 {
            return Err("single page support only".into());
        }
        let page = &font.pages[0];

        let image_data = fs::read(folder.join(page))?;

        Self::load_static(font, &image_data, screen_width, screen_height, cap)
    }

    pub fn load_static(
        font: Font,
        image_data: &[u8],
        screen_width: u32,
        screen_height: u32,
        cap: usize,
    ) -> Result<Self, Box<dyn Error>> {
        let mut image = image::load_from_memory_with_format(image_data, ImageFormat::Png)
            .map(|u| u.into_luma8())?;

        if image.width() != font.common.scale_w as u32
            || image.height() != font.common.scale_h as u32
        {
            return Err("incongruent texture dimensions".into());
        }
        imageops::flip_vertical_in_place(&mut image);

        let texture = build_texture(&image)?;

        let x_k = 2.0 / screen_width as f32;
        let y_k = 2.0 / screen_height as f32;
        let u_k = 1.0 / font.common.scale_w as f32;
        let v_k = 1.0 / font.common.scale_h as f32;

        let chars = chars_lossy(u_k, v_k, x_k, y_k, &font.chars);

        let Vaos { vao, vbo } = Vaos::build(cap)?;

        let buffer = Vec::with_capacity(cap);

        Ok(Self { buffer, chars, cap, vao, vbo, texture })
    }

    pub fn clear(&mut self) {
        self.buffer.clear();
    }

    pub fn moo(&mut self) {
        let color = Rgb::BLUE.into_rgba(1.0);
        let v0 = Blit::new(P2::new(0.0, 0.0), P2::new(0.0, 0.0), color);
        let v1 = Blit::new(P2::new(0.0, 1.0), P2::new(0.0, 1.0), color);
        let v2 = Blit::new(P2::new(1.0, 1.0), P2::new(1.0, 1.0), color);
        let v3 = Blit::new(P2::new(1.0, 0.0), P2::new(1.0, 0.0), color);
        self.buffer.push(v1);
        self.buffer.push(v0);
        self.buffer.push(v2);
        self.buffer.push(v3);
        self.buffer.push(v2);
        self.buffer.push(v0);
    }

    /// Blit type program must be bound.
    pub fn draw(&mut self) {
        self.bind_texture();
        self.buffer_data();
        self.draw_arrays();
    }

    fn bind_texture(&self) {
        unsafe {
            gl::BindTexture(gl::TEXTURE_2D, self.texture.name());
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::REPEAT as GLint);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::REPEAT as GLint);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR as GLint);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as GLint);
        }
        glx::check_debug();
    }

    fn buffer_data(&mut self) {
        unsafe {
            gl::BindBuffer(gl::ARRAY_BUFFER, self.vbo.name());
            if self.buffer.len() > self.cap {
                self.cap = self.buffer.capacity();
                gl::BufferData(
                    gl::ARRAY_BUFFER,
                    (self.cap * mem::size_of::<Blit>()) as GLsizeiptr,
                    self.buffer.as_ptr() as *const GLvoid,
                    gl::DYNAMIC_DRAW,
                );
                glx::check_debug();
            } else {
                gl::BufferSubData(
                    gl::ARRAY_BUFFER,
                    0,
                    mem::size_of_val(self.buffer.as_slice()) as GLsizeiptr,
                    self.buffer.as_ptr() as *const GLvoid,
                );
                glx::check_debug();
            }
        }
    }

    fn draw_arrays(&mut self) {
        unsafe {
            gl::BindVertexArray(self.vao.name());
            gl::DrawArrays(gl::TRIANGLES, 0, self.buffer.len() as GLsizei);
        }
        glx::check_debug()
    }

    pub fn push_str(&mut self, string: &str, mut pos: P2, color: Rgba) -> f32 {
        for c in string.chars() {
            pos.x += self.push_char(c, pos, color);
        }
        pos.x
    }

    pub fn push_char(&mut self, c: char, pos: P2, color: Rgba) -> f32 {
        if (c as usize) < 128 {
            let CharLite { u, v, us, vs, x, y, xs, ys, a } = self.chars[c as usize];
            let x = pos.x + x;
            let y = pos.y + y;
            let v0 = Blit::new(P2::new(x, y), P2::new(u, v), color);
            let v1 = Blit::new(P2::new(x, y + ys), P2::new(u, v + vs), color);
            let v2 = Blit::new(P2::new(x + xs, y + ys), P2::new(u + us, v + vs), color);
            let v3 = Blit::new(P2::new(x + xs, y), P2::new(u + us, v), color);
            self.buffer.push(v1);
            self.buffer.push(v0);
            self.buffer.push(v2);
            self.buffer.push(v3);
            self.buffer.push(v2);
            self.buffer.push(v0);
            a
        } else {
            0.0
        }
    }
}

struct Vaos {
    vao: VertexArrayObject,
    vbo: BufferObject,
}

impl Vaos {
    fn build(cap: usize) -> glx::Result<Self> {
        // VAO gen
        let vao = VertexArrayObject::gen();
        glx::check()?;
        unsafe {
            gl::BindVertexArray(vao.name());
        }
        glx::check()?;
        // VBO
        let vbo = BufferObject::gen();
        unsafe {
            gl::BindBuffer(gl::ARRAY_BUFFER, vbo.name());
            gl::BufferData(
                gl::ARRAY_BUFFER,
                (cap * mem::size_of::<Blit>()) as GLsizeiptr,
                ptr::null(),
                gl::DYNAMIC_DRAW,
            );
        }
        glx::check()?;
        unsafe { Blit::init_vao() };
        // Done
        unsafe {
            gl::BindVertexArray(0);
            gl::BindBuffer(gl::ARRAY_BUFFER, 0);
            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, 0);
        }
        Ok(Self { vao, vbo })
    }
}

#[derive(Copy, Clone, Debug, Default)]
pub struct CharLite {
    u: f32,  // texture coord u
    v: f32,  // texture coord v
    us: f32, // texture coord u size/ width
    vs: f32, // texture coord v size/ height
    x: f32,  // screen coord x
    y: f32,  // screen coord y
    xs: f32, // screen coord x size/ width
    ys: f32, // screen coord y size/ height
    a: f32,  // screen advance
}

impl CharLite {
    pub const fn new(
        u: f32,
        v: f32,
        us: f32,
        vs: f32,
        x: f32,
        y: f32,
        xs: f32,
        ys: f32,
        a: f32,
    ) -> Self {
        Self { u, v, us, vs, x, y, xs, ys, a }
    }
}

fn chars_lossy(u_k: f32, v_k: f32, x_k: f32, y_k: f32, chars: &[Char]) -> [CharLite; 128] {
    let mut char_lites = [CharLite::default(); 128];
    for c in chars.iter() {
        let index = c.id as usize;
        if index < 128 {
            char_lites[index] = CharLite::new(
                c.x as f32 * u_k,
                1.0 - c.y as f32 * v_k,
                c.width as f32 * u_k,
                -(c.height as f32) * v_k,
                c.xoffset as f32 * x_k,
                -(c.yoffset as f32) * y_k,
                c.width as f32 * x_k,
                -(c.height as f32) * y_k,
                c.xadvance as f32 * x_k,
            );
        }
    }
    char_lites
}

fn build_texture(src: &GrayImage) -> glx::Result<TextureObject> {
    let txo = TextureObject::gen();
    glx::check()?;
    unsafe {
        gl::BindTexture(gl::TEXTURE_2D, txo.name());
        gl::TexImage2D(
            gl::TEXTURE_2D,
            0,
            gl::R8 as GLint,
            src.width() as GLsizei,
            src.height() as GLsizei,
            0,
            gl::RED,
            gl::UNSIGNED_BYTE,
            src.as_ptr() as *const GLvoid,
        );
        gl::BindTexture(gl::TEXTURE_2D, 0);
    }
    glx::check()?;
    Ok(txo)
}
