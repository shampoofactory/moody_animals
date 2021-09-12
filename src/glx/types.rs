//! Our basic data types.

use crate::glx;

use gl::types::*;
use std::mem;

macro_rules! build {
    ($name:ident, $($element: ident: $ty: ty),*) => {
        #[derive(Clone, Copy, PartialEq, Debug, Default)]
        #[repr(C)]
        pub struct $name { $(pub $element: $ty),* }

        impl $name {
            #[inline(always)]
            pub const fn new($($element: $ty),*) -> Self {
                Self { $($element),* }
            }
        }
    }
}

build!(P2, x: f32, y: f32);
build!(P3, x: f32, y: f32, z: f32);
build!(V2, u: f32, v: f32);
build!(V3, u: f32, v: f32, w: f32);
build!(V3Rgb, v: V3, rgb: Rgb);
build!(Rect2, top_left: V3, bottom_right: V3);
build!(Quad, p0: P2, p1: P2, p2: P2, p4: P2);
build!(Rgb, r: f32, g: f32, b: f32);
build!(Rgba, r: f32, g: f32, b: f32, a: f32);

impl Rgb {
    pub const RED: Rgb = Rgb::new(1.0, 0.0, 0.0);
    pub const YELLOW: Rgb = Rgb::new(1.0, 1.0, 0.0);
    pub const GREEN: Rgb = Rgb::new(0.0, 1.0, 0.0);
    pub const CYAN: Rgb = Rgb::new(0.0, 1.0, 1.0);
    pub const BLUE: Rgb = Rgb::new(0.0, 0.0, 1.0);
    pub const MAGENTA: Rgb = Rgb::new(1.0, 0.0, 1.0);
    pub const BLACK: Rgb = Rgb::new(0.0, 0.0, 0.0);
    pub const WHITE: Rgb = Rgb::new(1.0, 1.0, 1.0);

    pub const fn into_rgba(self, a: f32) -> Rgba {
        Rgba::new(self.r, self.g, self.b, a)
    }
}

build!(Blit, pos: P2, tex_coord: P2, color: Rgba);

impl Blit {
    pub unsafe fn init_vao() {
        let stride = mem::size_of::<Self>() as GLsizei;
        let mut pointer = 0;
        gl::VertexAttribPointer(0, 2, gl::FLOAT, gl::FALSE, stride, pointer as *const GLvoid);
        pointer += 2 * mem::size_of::<GLfloat>();
        gl::VertexAttribPointer(1, 2, gl::FLOAT, gl::FALSE, stride, pointer as *const GLvoid);
        pointer += 2 * mem::size_of::<GLfloat>();
        gl::VertexAttribPointer(2, 4, gl::FLOAT, gl::FALSE, stride, pointer as *const GLvoid);
        pointer += 4 * mem::size_of::<GLfloat>();
        assert_eq!(pointer, stride as usize);
        gl::EnableVertexAttribArray(0);
        gl::EnableVertexAttribArray(1);
        gl::EnableVertexAttribArray(2);
        glx::check_debug();
    }
}
