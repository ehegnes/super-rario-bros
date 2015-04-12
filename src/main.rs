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

trait Sprite {
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
    fn move_dir(&mut self, dir: i32);
    fn update(&mut self, kb_state: ScanCodes);
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

struct Mario<'renderer> {
    texture: Texture<'renderer>,
    x: f32,
    y: f32,
    vx: f32,
    vy: f32,
    falling: bool,
}

impl<'renderer> Mario<'renderer> {
    fn new(path: &str, renderer: &'renderer Renderer) -> Mario<'renderer> {
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

impl<'renderer> Sprite for Mario<'renderer> {
    fn texture(&self) -> &Texture { &self.texture }
    fn rect(&self) -> Rect { Rect::new(self.x as i32, self.y as i32, TILE_SIZE, TILE_SIZE) }

    // Setter and getter functions for internal variables
    fn x(&self) -> f32      { self.x }
    fn vx(&self) -> f32         { self.vx }
    #[allow(dead_code)]
    fn y(&self) -> f32      { self.y }
    fn vy(&self) -> f32         { self.vy }
    fn falling(&self) -> bool { self.falling }
    fn set_x(&mut self, x: f32)   { self.x = x; }
    fn set_y(&mut self, y: f32)   { self.y = y; }
    fn set_vx(&mut self, vx: f32) { self.vx = vx; }
    fn set_vy(&mut self, vy: f32) { self.vy = vy; }
    fn set_falling(&mut self, f: bool){ self.falling = f; }

    // Auxillary functions
    fn jump(&mut self) { if !self.falling { self.vy = -3.4; self.falling = true; }}
    fn move_dir(&mut self, dir: i32) {
        self.vx += (dir.to_f32().unwrap())*0.02;
        let max_speed = 1.0;
        if self.vx > max_speed { self.vx = max_speed; }
        if self.vx < -max_speed { self.vx = -max_speed; }
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
    if mario.x() > 80.0 {
        *x_back += mario.x() as i32 - 80;
        mario.set_x(80.0);
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
    let mut mario = Mario::new("res/mario-walking-right.bmp", &renderer);
    mario.set_y((WIN_Y-GROUND_OFFSET-TILE_SIZE) as f32);

    let mut sprites: Vec<&mut Sprite> = Vec::new();
    sprites.push(&mut mario as &mut Sprite);

    // Generate Rects to be drawn on map
    let world_rects_overlay = load_map("res/world1-1.txt", &renderer);
    let world_rects = map_to_rects("res/world1-1.txt");

    // Track the background x-axis scrolling
    let mut x_back = 0;

    // Initialize drawer
    let mut drawer = renderer.drawer();
    let _ = drawer.clear();
    let _ = drawer.copy(&world, None, None);
    let _ = drawer.copy((*sprites[0]).texture(), None, Some((*sprites[0]).rect()));
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
        if kb_state[&ScanCode::Left]  { sprites[0].move_dir(-1); }
        if kb_state[&ScanCode::Right] { sprites[0].move_dir( 1); }

        // Background scrolling
        scroll_background(&mut x_back, &mut *sprites[0]);

        // X and Y collision handling
        // TODO: This needs revision. The method arrays need to be properly initialized as well.
        //       Look in the 'trait-refactor' branch for current work on this subject.
        for dir in ["x", "y"].iter() {
            for sprite in sprites.iter_mut() {
                sprite.move_mutate(dir);
                for rect in &*world_rects {
                    let rect = rect.unwrap();
                    let rect = Rect::new(rect.x - x_back, rect.y, TILE_SIZE, TILE_SIZE);
                    if rect.has_intersection(&sprite.rect()) {
                        let coll_rect = rect.intersection(&sprite.rect()).unwrap();
                        sprite.handle_coll(dir, coll_rect);
                        break;
                    }
                }
            }
        }

        // Process jumping
        if kb_state[&ScanCode::Up] { sprites[0].jump(); }

        // TODO: update() should eventually be mapped to a list containing all Sprites
        sprites[0].update(kb_state);

        drawer.clear();
        drawer.copy(&world,               Some(Rect::new(x_back, 0, WIN_X, WIN_Y)), None);
        drawer.copy(&world_rects_overlay, Some(Rect::new(x_back, 0, WIN_X, WIN_Y)), None);
        drawer.copy(sprites[0].texture(), None, Some(sprites[0].rect()));
        drawer.present();
    }
}
