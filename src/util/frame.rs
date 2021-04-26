use super::pixel::Pixel;
use crate::{COLUMNS,ROWS};
// This is a representation of the frame we're currently rendering
pub struct Frame {
    pos: usize,
    pixels: Vec<Vec<Pixel>>
}

// The Frame should contain the pixels that are currently shown
// on the LED board. In most cases, the Frame will have less pixels
// than the input Image!
impl Frame {
    pub fn new() -> Frame {

        let frame: Frame = Frame {
            pos: 0,
            pixels: vec![vec![Pixel::new(); COLUMNS as usize]; ROWS as usize],
        };

        frame
    }

    fn next_image_frame(&mut self) {
        for row in 0..ROWS {
            for col in 0..COLUMNS {
                let _image_position = (self.pos) as usize;
                
                //lijntje hier onder is nog 'raw'
                //self.pixels[row][col] = image.pixels[row][image_position].clone();
            
                //raw -> full
                let raw_color = Pixel::new();

                //rgb waarden naar full color converteren en er dan in zetten (gamma correction)
                self.pixels[row as usize][col as usize].r = self.raw_color_to_full_color(raw_color.r);
                self.pixels[row as usize][col as usize].g = self.raw_color_to_full_color(raw_color.g);
                self.pixels[row as usize][col as usize].b = self.raw_color_to_full_color(raw_color.b);
            
            }
        }

    }

    //voor gamma correction
    fn raw_color_to_full_color(self: &mut Frame, raw_color: u8) -> u8{
        //let full_color = ((raw_color as u32)* ((1<<COLOR_DEPTH) -1)/255) as u16;
        let gammaCorrection : f32 = 1.75;
        
        let _raw_color_float = raw_color as f32;
        let max_value_float = 255 as f32;
        
        let full_color = (max_value_float * (raw_color as f32 / max_value_float).powf(gammaCorrection)) as u8;
        
        full_color
    }

    fn clear_frame(self:&mut Frame){
        self.pixels=vec![vec![Pixel::new(); COLUMNS as usize]; ROWS as usize];
    }
}