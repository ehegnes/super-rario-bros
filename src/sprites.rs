use sdl2::pixels::Color::RGB;
use sdl2::rect::Rect;
use sdl2::render::{Renderer, Texture};
use sdl2::scancode::ScanCode;
use sdl2::surface::Surface;

use std::collections::HashMap;
use std::path::Path;

type ScanCodes = HashMap<ScanCode, bool>;

use mapgen::map_to_rects;

static WIN_X: i32 = 254;
static WIN_Y: i32 = 224;
static TILE_SIZE: i32 = 16;
static GRAVITY: f32 = 9.80665/2.0; // pixels per second ;)

pub fn load_image(filename: &str, renderer: &Renderer) -> Texture {
    let surface = Surface::from_bmp(Path::new(filename)).unwrap();
    surface.set_color_key(true, RGB(255, 0, 255)).unwrap();
    renderer.create_texture_from_surface(&surface).unwrap()
}

pub fn load_map(filename: &str, renderer: &Renderer) -> Texture {
    let map = map_to_rects(filename);
    let mut surface: Surface = Surface::from_bmp(Path::new("res/world1-1.bmp")).unwrap();
    surface.fill_rects(&map, RGB(0, 0, 0)).unwrap();
    renderer.create_texture_from_surface(&surface).unwrap()
}

pub trait Sprite {
    fn texture(&self) -> &Texture;
    fn rect(&self) -> Rect;
    fn x(&self) -> f32;
    fn vx(&self) -> f32;
    #[allow(dead_code)]
    fn y(&self) -> f32;
    fn vy(&self) -> f32;
    fn falling(&self) -> bool;
    fn set_falling(&mut self, f: bool);
    fn set_x(&mut self, x: f32);
    fn set_y(&mut self, y: f32);
    fn set_vx(&mut self, vx: f32);
    fn set_vy(&mut self, vy: f32);
    fn jump(&mut self);
    fn update(&mut self, kb_state: ScanCodes);
    fn move_dir(&mut self, dir: i32) {
        let vx = self.vx();
        self.set_vx(vx + (dir as f32) * 0.02);
        let max_speed = 1.0;
        if vx > max_speed  { self.set_vx(max_speed); }
        if vx < -max_speed { self.set_vx(-max_speed); }
    }
    fn handle_coll(&mut self, dir: &str, coll_rect: Rect) {
        let vx = self.vx();
        let vy = self.vy();
        if dir == "x" {
            if vx > 0.0 { self.set_x((coll_rect.x-TILE_SIZE) as f32); }
            if vx < 0.0 { self.set_x((coll_rect.x + coll_rect.w) as f32); }
            self.set_vx(0.0);
        } else if dir == "y" && self.falling() {
            if vy > 0.0 { // moving downward
                self.set_y((coll_rect.y-TILE_SIZE) as f32);
                self.set_falling(false);
            } else { // moving upward
                self.set_y((coll_rect.y+coll_rect.h) as f32);
            }
            self.set_vy(0.0);
        }
    }
    fn move_mutate(&mut self, dir: &str) {
        if dir == "x" {
            let x  = self.x();
            let vx = self.vx();
            self.set_x(x + vx);
        } else if dir == "y" && self.falling() {
            let y  = self.y();
            let vy = self.vy();
            self.set_y(y + vy);
            self.set_vy(vy + GRAVITY/60.0);
        }
    }
}

pub struct Enemy {
    texture: Texture,
    x: f32,
    y: f32,
    vx: f32,
    vy: f32,
    falling: bool,
}

impl Enemy {
    pub fn new(path: &str, renderer: &Renderer) -> Enemy {
        Enemy {
            texture: load_image(path, renderer),
            x: 0.0,
            y: 0.0,
            vx: 0.0,
            vy: 0.0,
            falling: false,
        }
    }
}

impl Sprite for Enemy {
    fn texture(&self) -> &Texture { &self.texture }
    fn rect(&self) -> Rect { Rect::new(self.x as i32, self.y as i32, TILE_SIZE, TILE_SIZE) }
    fn x(&self) -> f32                 { self.x }
    fn vx(&self) -> f32                { self.vx }
    #[allow(dead_code)]
    fn y(&self) -> f32                 { self.y }
    fn vy(&self) -> f32                { self.vy }
    fn falling(&self) -> bool          { self.falling }
    fn set_x(&mut self, x: f32)        { self.x = x; }
    fn set_y(&mut self, y: f32)        { self.y = y; }
    fn set_vx(&mut self, vx: f32)      { self.vx = vx; }
    fn set_vy(&mut self, vy: f32)      { self.vy = vy; }
    fn set_falling(&mut self, f: bool) { self.falling = f; }
    fn jump(&mut self) {
        if !self.falling { self.vy = -3.4; self.falling = true; }
    }
    fn handle_coll(&mut self, dir: &str, coll_rect: Rect) {
        let vx = self.vx();
        let vy = self.vy();
        if dir == "x" {
            if vx.is_sign_negative() { // collision on right of sprite
                self.set_x((coll_rect.x + coll_rect.w) as f32);
            } else { // collision on left of sprite
                self.set_x((coll_rect.x - TILE_SIZE) as f32);
            }
            self.set_vx(-vx); // bounce back and forth between objects
        } else if dir == "y" && self.falling() {
            if vy > 0.0 { // moving downward
                self.set_y((coll_rect.y - TILE_SIZE) as f32);
                self.set_falling(false);
            } else { // moving upward
                self.set_y((coll_rect.y + coll_rect.h) as f32);
            }
            self.set_vy(0.0);
        }
    }
    #[allow(unused_variables)]
    fn update(&mut self, kb_state: ScanCodes) {
        // Reset self.falling
        self.falling = true;
    }
}

pub struct Mario {
    texture: Texture,
    x: f32,
    y: f32,
    vx: f32,
    vy: f32,
    falling: bool,
}

impl Mario {
    pub fn new(path: &str, renderer: &Renderer) -> Mario {
        Mario {
            texture: load_image(path, renderer),
            x: 0.0,
            y: 0.0,
            vx: 0.0,
            vy: 0.0,
            falling: false,
        }
    }
}

impl Sprite for Mario {
    fn texture(&self) -> &Texture { &self.texture }
    fn rect(&self) -> Rect { Rect::new(self.x as i32, self.y as i32, TILE_SIZE, TILE_SIZE) }
    fn x(&self) -> f32                 { self.x }
    fn vx(&self) -> f32                { self.vx }
    #[allow(dead_code)]
    fn y(&self) -> f32                 { self.y }
    fn vy(&self) -> f32                { self.vy }
    fn falling(&self) -> bool          { self.falling }
    fn set_x(&mut self, x: f32)        { self.x = x; }
    fn set_y(&mut self, y: f32)        { self.y = y; }
    fn set_vx(&mut self, vx: f32)      { self.vx = vx; }
    fn set_vy(&mut self, vy: f32)      { self.vy = vy; }
    fn set_falling(&mut self, f: bool) { self.falling = f; }
    fn jump(&mut self) {
        if !self.falling { self.vy = -3.4; self.falling = true; }
    }
    fn update(&mut self, kb_state: ScanCodes) {
        // Friction
        if !(kb_state[&ScanCode::Left] || kb_state[&ScanCode::Right] || self.falling) {
            self.vx -= 0.2;
            if self.vx.abs_sub(0.2) < 0.2 {
                self.vx = 0.0;
            }
        }
        // bounds checking
        if (self.x as i32) < 0               { self.set_x(0.0); self.set_vx(0.0); }
        if (self.x as i32) > WIN_X-TILE_SIZE { self.set_x((WIN_X-TILE_SIZE) as f32); }
        // Reset self.falling
        self.falling = true;
        // GAME OVER!
        if self.y as i32 > WIN_Y { panic!("GAME OVER!"); }
    }
}
