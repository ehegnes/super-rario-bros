use sdl2;

pub struct RateLimiter {
    fps: u32,
    last_ticks: u32
}

impl RateLimiter {
    pub fn new(fps: u32) -> RateLimiter {
        RateLimiter {
            fps: fps,
            last_ticks: 0
        }
    }

    pub fn limit(&mut self) {
        let ticks = sdl2::timer::get_ticks();
        let adjusted_ticks = ticks - self.last_ticks;
        if adjusted_ticks < 1000 / self.fps {
            sdl2::timer::delay((1000 / self.fps) - adjusted_ticks);
        }
        self.last_ticks = ticks;
    }
}
