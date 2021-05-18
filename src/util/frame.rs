// ==================================== PROJECT IMPORTS =======================================
use super::image::Image;
use super::pixel::Pixel;

use crate::{COLOR_DEPTH, COLUMNS, ROWS};

// ===========================================================================
// This is a representation of the frame we're currently rendering
// ===========================================================================

pub struct Frame {
    pos: usize,
    pub pixels: Vec<Vec<Pixel>>,
}

/* The Frame should contain the pixels that are currently shown on the LED board.
In most cases, the Frame will have less pixels than the input Image!*/
impl Frame {
    // ==================================== CONSTRUCTOR =======================================
    pub fn new() -> Frame {
        let frame: Frame = Frame {
            pos: 0,
            pixels: vec![vec![Pixel::new(); COLUMNS as usize]; ROWS as usize],
        };

        frame
    }
    // ==================================== PUBLIC FUNCTIONS =======================================
    pub fn next_image_frame(&mut self, image: &Image) {
        // let blokgrootte_breedte = image.width/(COLUMNS*3);
        // let blokgrootte_lengte = image.height/ROWS;

        for row in 0..ROWS {
            for col in 0..COLUMNS {
                //let image_position = ((self.pos + col*blokgrootte_breedte) as usize % image.width) as usize;
                let image_position = (self.pos + col) % image.width;

                //let rgb = image.pixels[row*blokgrootte_lengte as usize][image_position].clone();
                let rgb = image.pixels[row][image_position].clone();

                //rgb waarden naar full color converteren en er dan in zetten (gamma correction)
                self.pixels[row as usize][col as usize].r = self.raw_to_full_color(rgb.r);
                self.pixels[row as usize][col as usize].g = self.raw_to_full_color(rgb.g);
                self.pixels[row as usize][col as usize].b = self.raw_to_full_color(rgb.b);
            }
        }

        self.pos = self.pos + 1;
        if self.pos >= image.width as usize {
            self.pos = 0;
        }
    }
    // ==================================== PRIVATE FUNCTIONS =======================================

    fn clear_frame(self: &mut Frame) {
        self.pixels = vec![vec![Pixel::new(); COLUMNS as usize]; ROWS as usize];
    }
    fn raw_to_full_color(&self, raw: u16) -> u16 {
        let full_color = raw * ((1 << COLOR_DEPTH) - 1) / 255;
        full_color
    }
}
