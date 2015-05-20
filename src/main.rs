extern crate sdl2;

use sdl2::event::Event;
use sdl2::keyboard::get_keyboard_state;
use sdl2::rect::Rect;
use sdl2::scancode::ScanCode;

use std::collections::HashMap;

type ScanCodes = HashMap<ScanCode, bool>;

mod ratelimiter;
use ratelimiter::RateLimiter;

mod mapgen;
use mapgen::map_to_rects;

mod sprites;
use sprites::{Mario, Enemy, Sprite, load_image, load_map};

static WIN_X: i32 = 254;
static WIN_Y: i32 = 224;
static NAME: &'static str = "Super Rario Bros";
static GROUND_OFFSET: i32 = 24;
static TILE_SIZE: i32 = 16;

fn scroll_background(x_back: &mut f32, sprites: &mut Vec<&mut Sprite>) {
    let result = sprites[0].x() - 80.0;
    if result > 0.0 {
        *x_back += result;
        sprites[0].set_x(80.0);
        // Move sprite backwards when scrolling background forwards
        for sprite in sprites.iter_mut().skip(1) {
            let x = sprite.x();
            sprite.set_x(x - result);
        }
    }


    // Bounds checking
    if *x_back < 0.0                 { *x_back = 0.0; }
    if *x_back > (3392-WIN_X) as f32 { *x_back = (3392-WIN_X) as f32; }
}

fn main() {
    // Initialize SDL2 subsystems
    //let sdl2_context = sdl2::init(INIT_VIDEO | INIT_EVENTS).unwrap();
    let sdl2_context = sdl2::init().video().events().unwrap();

    // Create main window
    //let window = Window::new(&sdl2_context, NAME, PosCentered, PosCentered, WIN_X, WIN_Y).unwrap();
    let window = sdl2_context.window(NAME, WIN_X as u32, WIN_Y as u32).position_centered().opengl().build().unwrap();

    // Initialize the renderer
    //let mut renderer = Renderer::from_window(window, RenderDriverIndex::Auto, ACCELERATED).unwrap();
    let mut renderer = window.renderer().build().unwrap();

    // Load World and Mario sprites
    let world = load_image("res/world1-1.bmp", &renderer);
    let mut mario = Mario::new("res/mario-walking-right.bmp", &renderer);
    mario.set_y((WIN_Y-GROUND_OFFSET-TILE_SIZE) as f32);
    let mut enemy = Enemy::new("res/mario-death.bmp", &renderer);
    enemy.set_x(100.0);
    enemy.move_dir(-20);

    let mut sprites: Vec<&mut Sprite> = Vec::new();
    sprites.push(&mut mario as &mut Sprite);
    sprites.push(&mut enemy as &mut Sprite);

    // Generate Rects to be drawn on map
    let world_rects_overlay = load_map("res/world1-1.txt", &renderer);
    let world_rects = map_to_rects("res/world1-1.txt");

    // Track the background x-axis scrolling
    let mut x_back = 0f32;

    // Initialize drawer
    let mut drawer = renderer.drawer();
    let _ = drawer.clear();
    let _ = drawer.copy(&world, None, None);
    let _ = drawer.copy(sprites[0].texture(), None, Some(sprites[0].rect()));
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
        scroll_background(&mut x_back, &mut sprites);

        // X and Y collision handling
        // TODO: Consider revision, specifically the way in which the 'x' and 'y' strings are
        //       passed to `move_mutate()` and `handle_coll()`. Is this the best way to handle x
        //       and y collision separately?
        for dir in ["x", "y"].iter() {
            for sprite in sprites.iter_mut() {
                sprite.move_mutate(dir);
                for rect in &*world_rects {
                    let rect = rect.unwrap();
                    let rect = Rect::new(rect.x - (x_back as i32), rect.y, TILE_SIZE, TILE_SIZE);
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

        // TODO: Consolidate this as a single loop (or map/closure) spanning all sprites
        sprites[0].update(kb_state);
        for sprite in sprites.iter_mut().skip(1) { sprite.set_falling(true); }

        // Begin drawing
        drawer.clear();
        drawer.copy(&world,               Some(Rect::new(x_back as i32, 0, WIN_X, WIN_Y)), None);
        drawer.copy(&world_rects_overlay, Some(Rect::new(x_back as i32, 0, WIN_X, WIN_Y)), None);
        // Draw all sprites
        // TODO: Skip the drawing of sprites that are offscreen
        for sprite in sprites.iter_mut().rev() {
            drawer.copy(sprite.texture(), None, Some(sprite.rect()));
        }
        drawer.present();
    }
}
