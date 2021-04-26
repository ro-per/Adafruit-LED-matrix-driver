// ==================================== PROJECT IMPORTS =======================================
use super::pixel::Pixel;
use crate::{COLUMNS,ROWS};

// ===========================================================================
// This is a representation of the frame we're currently rendering
// ===========================================================================

pub struct Frame {
    pos: usize,
    pixels: Vec<Vec<Pixel>>
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

    // ==================================== PRIVATE FUNCTIONS =======================================
    fn next_image_frame(&mut self) {
        for row in 0..ROWS {
            for col in 0..COLUMNS {
                let _image_position = (self.pos) as usize;
    
            }
        }

    }


    fn clear_frame(self:&mut Frame){
        self.pixels=vec![vec![Pixel::new(); COLUMNS as usize]; ROWS as usize];
    }
}