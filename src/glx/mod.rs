//! OpenGL helper objects and functions.

pub mod types;

mod error;
mod font;
mod objects;
mod program;

pub use error::{Error, Result};
pub use font::FontMonkey;
pub use objects::*;
pub use program::*;

pub fn check_debug() {
    #[cfg(debug_assertions)]
    {
        check().unwrap();
    }
}

pub fn check() -> error::Result<()> {
    unsafe {
        match gl::GetError() {
            gl::NO_ERROR => Ok(()),
            gl::INVALID_ENUM => Err(error::Error::OpenGL("invalid enum".to_owned())),
            gl::INVALID_VALUE => Err(error::Error::OpenGL("invalid value".to_owned())),
            gl::INVALID_OPERATION => Err(error::Error::OpenGL("invalid operation".to_owned())),
            gl::INVALID_FRAMEBUFFER_OPERATION => {
                Err(error::Error::OpenGL("invalid framebuffer operation".to_owned()))
            }
            gl::OUT_OF_MEMORY => Err(error::Error::OpenGL("out of memory".to_owned())),
            gl::STACK_UNDERFLOW => Err(error::Error::OpenGL("stack underflow".to_owned())),
            gl::STACK_OVERFLOW => Err(error::Error::OpenGL("stack overflow".to_owned())),
            u => Err(error::Error::OpenGL(format!("error: {}", u))),
        }
    }
}
