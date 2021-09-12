use crate::glx::{types::*, FontMonkey};

use rand::prelude::*;

use std::mem;

fn to_vec(list: &str) -> Vec<String> {
    list.rsplit('\n').filter(|u| !u.is_empty()).map(|u| u.to_owned()).collect()
}

fn rng_color<R: Rng>(rng: &mut R) -> Rgb {
    let h: f32 = rng.gen_range(0.0..360.0);
    let s: f32 = rng.gen_range(0.0..1.0);
    let v = 1.0;

    let f = (h / 60.0).fract();
    let p = v * (1.0 - s);
    let q = v * (1.0 - s * f);
    let t = v * (1.0 - s * (1.0 - f));
    match (h / 60.0) as u32 {
        0 => Rgb::new(v, t, p),
        1 => Rgb::new(q, v, p),
        2 => Rgb::new(p, v, t),
        3 => Rgb::new(p, q, v),
        4 => Rgb::new(t, p, v),
        _ => Rgb::new(v, p, q),
    }
}

pub struct DynWords {
    words_u: Vec<DynWord>,
    words_v: Vec<DynWord>,
    head: Vec<String>,
    tail: Vec<String>,
    frame_hi: u32,
    frame_lo: u32,
    cap: usize,
    width: u32,
    height: u32,
    p: f64,
}

impl DynWords {
    pub fn new(
        word_cap: usize,
        width: u32,
        height: u32,
        frame_hi: u32,
        frame_lo: u32,
        p: f64,
    ) -> Self {
        let head = to_vec(include_str!("../assets/words/moods.txt"));
        let tail = to_vec(include_str!("../assets/words/animals.txt"));
        Self {
            words_u: Vec::with_capacity(word_cap),
            words_v: Vec::with_capacity(word_cap),
            head,
            tail,
            frame_hi,
            frame_lo,
            cap: word_cap,
            width,
            height,
            p,
        }
    }

    pub fn push<R: Rng>(&mut self, monkey: &mut FontMonkey, rng: &mut R) {
        debug_assert!(self.words_v.is_empty());
        for mut word in self.words_u.drain(..) {
            if word.push(monkey) {
                self.words_v.push(word);
            }
        }
        mem::swap(&mut self.words_u, &mut self.words_v);
        self.gen_p(rng);
    }

    fn gen_p<R: Rng>(&mut self, rng: &mut R) {
        let mut p = self.p;
        while self.words_u.len() != self.cap {
            if p >= 1.0 {
                self.gen(rng);
                p -= 1.0;
            } else {
                if rng.gen_bool(p) {
                    self.gen(rng);
                }
                break;
            }
        }
    }

    fn gen<R: Rng>(&mut self, rng: &mut R) {
        let head = &self.head[rng.gen_range(0..self.head.len())];
        let tail = &self.tail[rng.gen_range(0..self.tail.len())];
        let word = format!("{} {}", head, tail);
        let frames = rng.gen_range(self.frame_lo..=self.frame_hi);
        self.words_u.push(DynWord::new(&word, rng, self.width, self.height, frames));
    }
}

pub struct DynWord {
    chars: Vec<DynChar>,
    color: Rgb,
    frames: u32,
    x: f32,
    y: f32,
    t: f32,
    ts: f32,
}

impl DynWord {
    pub fn new<R: Rng>(str: &str, rng: &mut R, width: u32, height: u32, frames: u32) -> Self {
        let color = rng_color(rng);
        let x = (rng.gen_range(0..width) as f32 + 0.5) / width as f32;
        let y = (rng.gen_range(0..height) as f32 + 0.5) / height as f32;
        let x = x * 2.0 - 1.0;
        let y = y * 2.0 - 1.0;
        let chars = str
            .chars()
            .map(|c| {
                let k = rng.gen_range(-2.0..2.0);
                DynChar { k, c }
            })
            .collect();
        let t = -1.0;
        let ts = 2.0 / frames as f32;
        Self { x, y, color, chars, frames, t, ts }
    }

    pub fn push(&mut self, monkey: &mut FontMonkey) -> bool {
        if self.frames == 0 {
            return false;
        }
        let alpha = 1.0 - self.t.abs();
        let color = self.color.into_rgba(alpha);
        let mut x = self.x;
        let y = self.t * self.t * self.t.signum();
        for c in &mut self.chars {
            x += monkey.push_char(c.c, P2::new(x, self.y + y * c.k), color);
        }
        self.t += self.ts;
        self.frames -= 1;
        true
    }
}

pub struct DynChar {
    k: f32,
    c: char,
}
