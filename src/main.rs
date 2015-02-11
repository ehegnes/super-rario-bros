#![feature(core)]
extern crate sdl2;

use sdl2::{INIT_VIDEO, INIT_EVENTS};
use sdl2::event::Event;
use sdl2::keyboard::get_keyboard_state;
use sdl2::pixels::Color::RGB;
use sdl2::rect::Rect;
use sdl2::render::{Renderer, RenderDriverIndex, ACCELERATED, Texture};
use sdl2::scancode::ScanCode;
use sdl2::surface::Surface;
use sdl2::video::{Window, OPENGL};
use sdl2::video::WindowPos::PosCentered;

use std::collections::HashMap;
use std::num::{Float, ToPrimitive};
use std::path::Path;

type ScanCodes = HashMap<ScanCode, bool>;

mod ratelimiter;
use ratelimiter::RateLimiter;

mod mapgen;
use mapgen::map_to_rects;

static WIN_X: i32 = 254;
static WIN_Y: i32 = 224;
static NAME: &'static str = "Super Rario Bros";
static GRAVITY: f32 = 9.80665/2.0; // pixels per second ;)
static GROUND_OFFSET: i32 = 24;
static TILE_SIZE: i32 = 16;

struct Sprite<'renderer> {
    texture: Texture<'renderer>,
    x: f32,
    y: f32,
    vx: f32,
    vy: f32,
    falling: bool,
}

impl<'renderer> Sprite<'renderer> {
    pub fn new(path: &str, renderer: &'renderer Renderer) -> Sprite<'renderer> {
        Sprite {
            texture: load_image(path, renderer),
            x: 0.0,
            y: 0.0,
            vx: 0.0,
            vy: 0.0,
            falling: false,
        }
    }

    pub fn texture(&self) -> &Texture { &self.texture }
    pub fn rect(&self) -> Rect { Rect::new(self.x as i32, self.y as i32, TILE_SIZE, TILE_SIZE) }

    // Setter and getter functions for internal variables
    pub fn x<T: ToPrimitive>(&mut self) -> i32      { self.x.to_i32().unwrap() }
    #[allow(dead_code)]
    pub fn y<T: ToPrimitive>(&mut self) -> i32      { self.y.to_i32().unwrap() }
    pub fn set_x<T: ToPrimitive>(&mut self, x: T)   { self.x = x.to_f32().unwrap(); }
    pub fn set_y<T: ToPrimitive>(&mut self, y: T)   { self.y = y.to_f32().unwrap(); }
    pub fn set_vx<T: ToPrimitive>(&mut self, vx: T) { self.vx = vx.to_f32().unwrap(); }
    pub fn set_vy<T: ToPrimitive>(&mut self, vy: T) { self.vy = vy.to_f32().unwrap(); }

    // Auxillary functions
    pub fn jump(&mut self) { if !self.falling { self.vy = -3.4; self.falling = true; }}
    pub fn move_dir<T: ToPrimitive>(&mut self, dir: T) {
        self.vx += (dir.to_f32().unwrap())*0.02;
        let max_speed = 1.0;
        if self.vx > max_speed { self.vx = max_speed; }
        if self.vx < -max_speed { self.vx = -max_speed; }
    }

    pub fn move_mutate(&mut self, dir: &str) {
        if dir == "x" {
            self.x += self.vx;
        } else if dir == "y" && self.falling {
            self.y += self.vy;
            self.vy += GRAVITY/60.0;
        }
    }

    pub fn handle_coll(&mut self, dir: &str, coll_rect: Rect) {
        if dir == "x" {
            if self.vx > 0.0 { self.set_x(coll_rect.x-TILE_SIZE); }
            if self.vx < 0.0 { self.set_x(coll_rect.x + coll_rect.w); }
            self.set_vx(0);
        } else if dir == "y" && self.falling {
            if self.vy > 0.0 { // moving downward
                self.set_y(coll_rect.y-TILE_SIZE);
                self.falling = false;
            } else { // moving upward
                self.set_y(coll_rect.y+coll_rect.h);
            }
            self.set_vy(0);
        }
    }

    pub fn update(&mut self, kb_state: ScanCodes) {
        // Friction
        if !(kb_state[&ScanCode::Left] || kb_state[&ScanCode::Right] || self.falling) {
            self.vx -= 0.2;
            if self.vx.abs_sub(0.2) < 0.2 {
                self.vx = 0.0;
            }
        }

        // bounds checking
        if (self.x as i32) < 0               { self.set_x(0); self.vx = 0.0; }
        if (self.x as i32) > WIN_X-TILE_SIZE { self.set_x(WIN_X-TILE_SIZE); }

        // Reset self.falling
        self.falling = true;

        // GAME OVER!
        if self.y as i32 > WIN_Y { panic!("GAME OVER!"); }
    }
}

fn load_image<'renderer>(filename: &str, renderer: &'renderer Renderer) -> Texture<'renderer> {
    let surface = Surface::from_bmp(Path::new(filename)).unwrap();
    surface.set_color_key(true, RGB(255, 0, 255)).unwrap();
    renderer.create_texture_from_surface(&surface).unwrap()
}

fn load_map<'renderer>(filename: &str, renderer: &'renderer Renderer) -> Texture<'renderer> {
    let map = map_to_rects(filename);
    let mut surface: Surface = Surface::from_bmp(Path::new("res/world1-1.bmp")).unwrap();
    surface.fill_rects(&map, RGB(0, 0, 0)).unwrap();
    renderer.create_texture_from_surface(&surface).unwrap()
}

fn scroll_background(x_back: &mut i32, mario: &mut Sprite) {
    if mario.x::<i32>() > 80 {
        *x_back += mario.x::<i32>() - 80;
        mario.set_x(80);
    }

    // Bounds checking
    if *x_back < 0          { *x_back = 0 }
    if *x_back > 3392-WIN_X { *x_back = 3392-WIN_X }
}

fn main() {
    // Initialize SDL2 subsystems
    let sdl2_context = sdl2::init(INIT_VIDEO | INIT_EVENTS).unwrap();

    // Create main window
    let window = Window::new(NAME, PosCentered, PosCentered, WIN_X, WIN_Y, OPENGL).unwrap();

    // Initialize the renderer
    let renderer = Renderer::from_window(window, RenderDriverIndex::Auto, ACCELERATED).unwrap();

    // Load World and Mario sprites
    let world = load_image("res/world1-1.bmp", &renderer);
    let mut mario = Sprite::new("res/mario-walking-right.bmp", &renderer);
    mario.set_x(0);
    mario.set_y(WIN_Y-GROUND_OFFSET-TILE_SIZE);

    // Generate Rects to be drawn on map
    let world_rects_overlay = load_map("res/world1-1.txt", &renderer);
    let world_rects = map_to_rects("res/world1-1.txt");

    // Track the background x-axis scrolling
    let mut x_back = 0;

    // Initialize drawer
    let mut drawer = renderer.drawer();
    let _ = drawer.clear();
    let _ = drawer.copy(&world, None, None);
    let _ = drawer.copy(mario.texture(), None, Some(mario.rect()));
    let _ = drawer.present();

    // Initialize rate limiter
    let mut rate_limiter = RateLimiter::new(60);

    // Main Loop
    let mut event_pump = sdl2_context.event_pump();
    'event : loop {
        if let Some(Event::Quit{..}) = event_pump.poll_event() { break 'event; }

        rate_limiter.limit();

        // Keyboard input
        let kb_state = get_keyboard_state();
        if kb_state[&ScanCode::Left]  { mario.move_dir(-1); }
        if kb_state[&ScanCode::Right] { mario.move_dir( 1); }

        // Background scrolling
        scroll_background(&mut x_back, &mut mario);

        // X and Y collision handling
        // TODO: This needs revision. The method arrays need to be properly initialized as well.
        //       Look in the 'trait-refactor' branch for current work on this subject.
        let f1fn = [Sprite::move_mutate, Sprite::move_mutate];
        let f2fn = [Sprite::handle_coll, Sprite::handle_coll];
        for (count, (f1, f2)) in f1fn.iter().zip(f2fn.iter()).enumerate() {
            let mut dir;
            if (count+1) % 2 == 0 { dir = "x"; } else { dir = "y"; }
            f1(&mut mario, dir);
            for rect in &*world_rects {
                let rect = rect.unwrap();
                let rect = Rect::new(rect.x - x_back, rect.y, TILE_SIZE, TILE_SIZE);
                if rect.has_intersection(&mario.rect()) {
                    let coll_rect = rect.intersection(&mario.rect()).unwrap();
                    f2(&mut mario, dir, coll_rect);
                    break;
                }
            }
        }

        // Process jumping
        if kb_state[&ScanCode::Up] { mario.jump(); }

        // TODO: update() should eventually be mapped to a list containing all Sprites
        mario.update(kb_state);

        drawer.clear();
        drawer.copy(&world,               Some(Rect::new(x_back, 0, WIN_X, WIN_Y)), None);
        drawer.copy(&world_rects_overlay, Some(Rect::new(x_back, 0, WIN_X, WIN_Y)), None);
        drawer.copy(mario.texture(), None, Some(mario.rect()));
        drawer.present();
    }
}
