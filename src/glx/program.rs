use crate::glx;

use gl;
use gl::types::*;

use std::ffi::CString;
use std::ptr;

use super::objects::{ProgramBuilder, ProgramObject};

macro_rules! program {
    ($name:ident, $shader:expr $(,$element:ident)*) => {
        #[derive(Debug)]
        pub struct $name {
            object: ProgramObject,
            $($element: UniformLocation,)*
        }

        impl $name {
            pub fn build() -> glx::Result<Self> {
                let object = ProgramBuilder::new()?
                .vert(include_bytes!(concat!("../../assets/shaders/",$shader,".vert")).as_ref())?
                .frag(include_bytes!(concat!("../../assets/shaders/",$shader,".frag")).as_ref())?
                .build()?;
                $(let $element = object.uniform_location(stringify!($element));)*
                Ok(Self { object, $($element, )*})
            }

            #[inline(always)]
            pub fn name(&self) -> GLuint { // TODO rework to program object
                self.object.name()
            }

            $(
            #[inline(always)]
            pub fn $element(&self) -> UniformLocation {
                self.$element
            }
            )*
        }
    };
}

program!(Blit, "blit");

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[repr(u32)]
pub enum ShaderType {
    Vertex = gl::VERTEX_SHADER,
    Fragment = gl::FRAGMENT_SHADER,
    Compute = gl::COMPUTE_SHADER,
}

impl ShaderType {
    pub fn extension(self) -> &'static str {
        match self {
            ShaderType::Vertex => "vert",
            ShaderType::Fragment => "frag",
            ShaderType::Compute => "comp",
        }
    }
}

#[derive(Debug)]
pub struct Shader(GLuint);

impl Shader {
    pub fn build<T: Into<Vec<u8>>>(shader_type: ShaderType, src: T) -> glx::Result<Self> {
        let src = CString::new(src).map_err(|u| glx::Error::Shader(u.to_string()))?;
        let shader = Self::create_shader(shader_type)?;
        unsafe { gl::ShaderSource(shader.0, 1, &src.as_ptr(), ptr::null()) };
        unsafe { gl::CompileShader(shader.0) }
        match shader.get_shader_iv(gl::COMPILE_STATUS) as GLboolean {
            gl::TRUE => Ok(shader),
            gl::FALSE => {
                let info_log_len = shader.get_shader_iv(gl::INFO_LOG_LENGTH);
                let info_log = shader.get_shader_info_log(info_log_len);
                Err(glx::Error::Shader(info_log))
            }
            u => Err(glx::Error::Shader(format!("build: bad GLboolean: {}", u))),
        }
    }

    fn create_shader(shader_type: ShaderType) -> glx::Result<Self> {
        let id = unsafe { gl::CreateShader(shader_type as GLenum) };
        match id {
            0 => Err(glx::Error::Shader(format!("create: failed: {:?}", shader_type))),
            gl::INVALID_ENUM => Err(glx::Error::Shader(format!(
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
