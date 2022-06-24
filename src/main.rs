pub mod dyn_words;
pub mod glx;

use dyn_words::DynWords;

use clap::{crate_version, App, Arg, ArgMatches};
use rand::prelude::*;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::video::{GLContext, GLProfile, SwapInterval, Window};
use sdl2::{Sdl, VideoSubsystem};

use std::error::Error;
use std::result::Result;
use std::str::FromStr;

const TITLE: &str = "Demo";

const CHAR_CAP: usize = 0x0400;
const WORD_CAP: usize = 0x0100;

pub struct Param {
    width: u32,
    height: u32,
    density: u32,
    speed: u32,
    fullscreen: bool,
}

impl Param {
    fn build(args: ArgMatches<'static>) -> Self {
        Self {
            width: get_u32(&args, "width"),
            height: get_u32(&args, "height"),
            density: get_u32(&args, "density"),
            speed: get_u32(&args, "speed"),
            fullscreen: args.is_present("fullscreen"),
        }
    }
}

pub struct State {
    sdl: Sdl,
    _video_subsystem: VideoSubsystem,
    window: Window,
    _context: GLContext,
}

impl State {
    pub fn windowed(title: &str, width: u32, height: u32) -> Result<Self, Box<dyn Error>> {
        Self::build(|vs| vs.window(title, width, height).opengl().build().unwrap())
    }

    pub fn fullscreen(title: &str) -> Result<Self, Box<dyn Error>> {
        Self::build(|vs| vs.window(title, 0, 0).fullscreen_desktop().opengl().build().unwrap())
    }

    fn build<W>(window_builder: W) -> Result<Self, Box<dyn Error>>
    where
        W: FnOnce(&VideoSubsystem) -> Window,
    {
        let sdl = sdl2::init().unwrap();
        let video_subsystem = sdl.video().unwrap();

        let gl_attr = video_subsystem.gl_attr();
        gl_attr.set_context_profile(GLProfile::Core);
        gl_attr.set_context_version(3, 3);

        let window = window_builder(&video_subsystem);

        let context = window.gl_create_context().unwrap();
        gl::load_with(|name| video_subsystem.gl_get_proc_address(name) as *const _);

        debug_assert_eq!(gl_attr.context_profile(), GLProfile::Core);
        debug_assert_eq!(gl_attr.context_version(), (3, 3));

        video_subsystem.gl_set_swap_interval(SwapInterval::VSync)?;
        Ok(Self { sdl, _video_subsystem: video_subsystem, window, _context: context })
    }
}

pub struct Demo {
    width: u32,
    height: u32,
    state: State,
}

impl Demo {
    pub fn new(state: State) -> Result<Self, Box<dyn Error>> {
        let mode = state.window.display_mode()?;
        Ok(Self { width: mode.w as u32, height: mode.h as u32, state })
    }

    pub fn execute(&mut self, p: f64, frame_hi: u32, frame_lo: u32) -> Result<(), Box<dyn Error>> {
        let font = bmfont_rs::text::from_str(include_str!("../assets/fonts/anton_latin.fnt"))?;
        let image_data = include_bytes!("../assets/fonts/anton_latin_0.png");

        let mut monkey =
            glx::FontMonkey::load_static(font, image_data, self.width, self.height, CHAR_CAP)?;
        let mut words = DynWords::new(WORD_CAP, self.width, self.height, frame_hi, frame_lo, p);
        let blit = glx::Blit::build()?;
        unsafe {
            gl::UseProgram(blit.name());
            gl::Enable(gl::BLEND);
            gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
        }
        let mut rng = thread_rng();
        let mut event_pump = self.state.sdl.event_pump().unwrap();
        'running: loop {
            unsafe {
                gl::ClearColor(0.0, 0.0, 0.0, 1.0);
                gl::Clear(gl::COLOR_BUFFER_BIT);
            }
            words.push(&mut monkey, &mut rng);
            monkey.draw();
            monkey.clear();
            self.state.window.gl_swap_window();
            for event in event_pump.poll_iter() {
                match event {
                    Event::Quit { .. } | Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                        break 'running
                    }
                    _ => {}
                }
            }
        }
        Ok(())
    }
}

pub fn main() -> Result<(), Box<dyn Error>> {
    let param = Param::build(arg_matches());
    let state = if param.fullscreen {
        State::fullscreen(TITLE)?
    } else {
        State::windowed(TITLE, param.width, param.height)?
    };
    // TODO modulate frames by frame rate, these assume 60Hz.
    // TODO modulate density by resolution
    let frame_lo = (101 - param.speed) * 20;
    let frame_hi = frame_lo + (frame_lo / 5);
    let p = 0.01 + param.density as f64 * 10.0;
    let p = p / frame_lo as f64;
    let mut demo = Demo::new(state)?;
    demo.execute(p, frame_hi, frame_lo)?;
    Ok(())
}

fn arg_matches() -> ArgMatches<'static> {
    App::new("moody_animals")
        .version(crate_version!())
        .author("Vin Singh <github.com/shampoofactory>")
        .about("Simple OpenGL bitmap font demo")
        .arg(
            Arg::with_name("width")
                .short("w")
                .long("width")
                .help("screen width")
                .takes_value(true)
                .default_value("1024")
                .validator(|u| is_u32_filter(&u, |_| true))
                .value_name("PIXELS"),
        )
        .arg(
            Arg::with_name("height")
                .short("h")
                .long("height")
                .help("screen height")
                .takes_value(true)
                .default_value("768")
                .validator(|u| is_u32_filter(&u, |_| true))
                .value_name("PIXELS"),
        )
        .arg(
            Arg::with_name("fullscreen")
                .short("f")
                .long("fullscreen")
                .help("fullscreen, overrides width/ height"),
        )
        .arg(
            Arg::with_name("density")
                .short("d")
                .long("density")
                .help("word density")
                .takes_value(true)
                .default_value("5")
                .validator(|u| is_u32_filter(&u, |v| v <= 100))
                .value_name("PERCENT"),
        )
        .arg(
            Arg::with_name("speed")
                .short("s")
                .long("speed")
                .help("animation speed")
                .takes_value(true)
                .default_value("70")
                .validator(|u| is_u32_filter(&u, |v| v <= 100))
                .value_name("PERCENT"),
        )
        .get_matches()
}

fn is_u32_filter<P>(v: &str, predicate: P) -> Result<(), String>
where
    P: Fn(u32) -> bool,
{
    match v.parse() {
        Ok(u) if predicate(u) => Ok(()),
        Ok(u) => Err(format!("invalid value: {}", u)),
        Err(err) => Err(err.to_string()),
    }
}

fn get_u32(args: &ArgMatches, name: &str) -> u32 {
    some_u32(args, name).unwrap_or_else(|| panic!("INTERNAL: default value error: {}", name))
}

fn some_u32(args: &ArgMatches, name: &str) -> Option<u32> {
    args.value_of(name).map(|u| {
        u32::from_str(u).unwrap_or_else(|_| panic!("INTERNAL: parse value error: {}", name))
    })
}
