use gl;
use gl::types::*;

use super::error;

use std::ffi::CString;

macro_rules! object {
    ($name:ident, $gen:ident, $delete:ident) => {
        #[derive(Debug)]
        #[repr(transparent)]
        pub struct $name(GLuint);

        impl $name {
            pub fn gen() -> Self {
                let mut idx: GLenum = 0;
                unsafe { gl::$gen(1, &mut idx) };
                debug_assert!(error::get_error().is_ok());
                Self(idx)
            }

            #[inline(always)]
            pub fn name(&self) -> GLuint {
                self.0
            }
        }

        impl Drop for $name {
            fn drop(&mut self) {
                unsafe { gl::$delete(1, &self.0) };
                debug_assert!(error::get_error().is_ok());
            }
        }
    };
}

object!(BufferObject, GenBuffers, DeleteBuffers);
object!(FramebufferObject, GenFramebuffers, DeleteFramebuffers);
object!(RenderbufferObject, GenRenderbuffers, DeleteRenderbuffers);
object!(TextureObject, GenTextures, DeleteTextures);
object!(VertexArrayObject, GenVertexArrays, DeleteVertexArrays);

pub struct ProgramBuilder {
    program: ProgramObject,
    shaders: Vec<Shader>,
}

impl ProgramBuilder {
    pub fn new() -> super::Result<Self> {
        let name = unsafe { gl::CreateProgram() };
        match name {
            0 => Err(super::Error::Program("create: failed".to_string())),
            u => Ok(Self {
                program: ProgramObject(u),
                shaders: Vec::default(),
            }),
        }
    }

    pub fn vert(mut self, src: &[u8]) -> super::Result<Self> {
        self.shaders.push(Shader::build(gl::VERTEX_SHADER, src)?);
        Ok(self)
    }

    pub fn frag(mut self, src: &[u8]) -> super::Result<Self> {
        self.shaders.push(Shader::build(gl::FRAGMENT_SHADER, src)?);
        Ok(self)
    }

    pub fn comp(mut self, src: &[u8]) -> super::Result<Self> {
        self.shaders.push(Shader::build(gl::COMPUTE_SHADER, src)?);
        Ok(self)
    }

    pub fn build(self) -> super::Result<ProgramObject> {
        let Self { program, shaders } = self;
        for shader in &shaders {
            unsafe { gl::AttachShader(program.0, shader.name()) };
        }
        unsafe { gl::LinkProgram(program.0) };
        match program.get_program_iv(gl::LINK_STATUS) as GLboolean {
            gl::TRUE => {
                for shader in &shaders {
                    unsafe { gl::DetachShader(program.0, shader.name()) };
                }
                Ok(program)
            }
            gl::FALSE => {
                let info_log_len = program.get_program_iv(gl::INFO_LOG_LENGTH);
                let info_log = program.get_program_info_log(info_log_len);
                Err(super::Error::Program(info_log))
            }
            u => Err(super::Error::Program(format!(
                "build: bad GLboolean: {}",
                u
            ))),
        }
    }
}

#[derive(Debug)]
pub struct ProgramObject(GLuint);

impl ProgramObject {
    pub fn uniform_location(&self, name: &str) -> UniformLocation {
        match unsafe { gl::GetUniformLocation(self.0, c_string(name).as_ptr() as *const GLchar) } {
            -1 => panic!("uniform: not found: {}", name),
            idx => UniformLocation(idx),
        }
    }

    fn get_program_info_log(&self, info_log_len: GLsizei) -> String {
        let mut length = 0;
        let mut info_log = vec![0u8; info_log_len as usize];
        unsafe {
            gl::GetProgramInfoLog(
                self.0,
                info_log_len,
                &mut length,
                info_log.as_mut_ptr() as *mut i8,
            );
        }
        match String::from_utf8(info_log) {
            Ok(u) => u,
            Err(e) => format!("get_program_info_log: utf8 error: {:?}", e),
        }
    }

    fn get_program_iv(&self, pname: GLenum) -> GLint {
        let mut params = 0;
        unsafe { gl::GetProgramiv(self.0, pname, &mut params) };
        params
    }

    #[inline(always)]
    pub fn name(&self) -> GLuint {
        self.0
    }
}

impl Drop for ProgramObject {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteProgram(self.0);
        }
    }
}

#[derive(Debug)]
struct Shader(GLuint);

impl Shader {
    fn build(shader_type: GLenum, src: &[u8]) -> super::Result<Self> {
        let shader = Self::create_shader(shader_type)?;
        let strings = [src.as_ptr() as *const GLchar];
        let lens = [src.len() as GLint];
        unsafe { gl::ShaderSource(shader.0, 1, strings.as_ptr(), lens.as_ptr()) };
        unsafe { gl::CompileShader(shader.0) }
        match shader.get_shader_iv(gl::COMPILE_STATUS) as GLboolean {
            gl::TRUE => Ok(shader),
            gl::FALSE => {
                let info_log_len = shader.get_shader_iv(gl::INFO_LOG_LENGTH);
                let info_log = shader.get_shader_info_log(info_log_len);
                Err(super::Error::Shader(info_log))
            }
            u => Err(super::Error::Shader(format!("build: bad GLboolean: {}", u))),
        }
    }

    fn create_shader(shader_type: GLenum) -> super::Result<Self> {
        let id = unsafe { gl::CreateShader(shader_type as GLenum) };
        match id {
            0 => Err(super::Error::Shader(format!(
                "create: failed: {:?}",
                shader_type
            ))),
            gl::INVALID_ENUM => Err(super::Error::Shader(format!(
                "create: unacceptable shader type: {:?}",
                shader_type
            ))),
            u => Ok(Self(u)),
        }
    }

    fn get_shader_info_log(&self, info_log_len: GLsizei) -> String {
        let mut length = 0;
        let mut info_log = vec![0u8; info_log_len as usize];
        unsafe {
            gl::GetShaderInfoLog(
                self.0,
                info_log_len,
                &mut length,
                info_log.as_mut_ptr() as *mut i8,
            );
        }
        match String::from_utf8(info_log) {
            Ok(u) => u,
            Err(e) => format!("get_shader_info_log: utf8 error: {:?}", e),
        }
    }

    fn get_shader_iv(&self, pname: GLenum) -> GLint {
        let mut params = 0;
        unsafe { gl::GetShaderiv(self.0, pname, &mut params) };
        params
    }

    #[inline(always)]
    pub fn name(&self) -> GLuint {
        self.0
    }
}

impl Drop for Shader {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteShader(self.0);
        }
    }
}

#[derive(Copy, Clone, Debug)]
#[repr(transparent)]
pub struct UniformLocation(GLint);

impl UniformLocation {
    #[inline(always)]
    pub fn location(&self) -> GLint {
        self.0
    }
}

impl UniformLocation {}

fn c_string<T: Into<Vec<u8>>>(t: T) -> CString {
    CString::new(t).expect("invalid C string")
}
