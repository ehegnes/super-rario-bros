extern crate sdl2;
use sdl2::rect::Rect;

use std::io::{BufReader, BufRead};
use std::fs::File;
use std::path::Path;

static TILE_SIZE: i32 = 16;

pub fn map_to_rects(filename: &str) -> Vec<Option<Rect>> {
    let mut rects: Vec<Option<Rect>> = vec![];
    let path = Path::new(filename);
    let file = BufReader::new(File::open(path).unwrap());

    for (row, line) in file.lines().filter_map(|res| res.ok()).enumerate() {
        for (column, block) in (&line).chars().enumerate() {
            //print!("{}", block);
            if block != '.' {
                rects.push(Some(Rect::new((column as i32)*TILE_SIZE,
                                          (row as i32)*TILE_SIZE+TILE_SIZE/2,
                                          TILE_SIZE, TILE_SIZE)));
            }
        }
    }
    return rects;
}

