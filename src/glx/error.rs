use sdl2::video;

use std::error;
use std::fmt;

pub type Result<T> = std::result::Result<T, Error>;

pub fn get_error() -> Result<()> {
    unsafe {
        match gl::GetError() {
            0 => Ok(()),
            u => Err(Error::OpenGL(format!("enum: {}", u))),
        }
    }
}

#[derive(Debug)]
pub enum Error {
    OpenGL(String),
    OutOfMemory,
    Program(String),
    Sdl(String),
    Shader(String),
}

impl error::Error for Error {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> std::result::Result<(), fmt::Error> {
        match self {
            Self::OpenGL(e) => write!(f, "OpenGL error: {}", e),
            Self::OutOfMemory => write!(f, "out of memory"),
            Self::Program(e) => write!(f, "program error: {}", e),
            Self::Sdl(e) => write!(f, "SDL error: {}", e),
            Self::Shader(e) => write!(f, "shader error: {}", e),
        }
    }
}

impl From<video::WindowBuildError> for Error {
    fn from(err: video::WindowBuildError) -> Self {
        Self::Sdl(format!("WindowBuildError: {}", err))
    }
}
