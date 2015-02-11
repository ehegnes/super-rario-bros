
/* s = s_0 + v_0 * t + 1/2 * a * t^2
   s : distance fallen
   s_0 : initial distance
   v_0 : initial velocity
   a : acceleration due to gravity
   t : time taken

   F = G * m1 * m2 / r^2
 */

use std::num::Float;

static G: f32 = 6.67384;
static MASS_EARTH: f32 = 5.972;
static RADIUS_EARTH: f32 = 6371.0;

pub fn force (massBody: f32) -> f32 {
  let g = G * 10.0.powi(-11);
  let mass_earth = MASS_EARTH * 10.0.powi(24);
  let radius_earth = RADIUS_EARTH * 10.0.powi(3);
  g * mass_earth * massBody / radius_earth.powi(2)
}

pub fn distance(s: f32, v: f32, a: f32, t: f32) -> f32 {
  s + v*t + 0.5 * a * t.powi(2)
}

pub fn velocity(v: f32, a: f32, t: f32) -> f32 {
  v + a*t
}

