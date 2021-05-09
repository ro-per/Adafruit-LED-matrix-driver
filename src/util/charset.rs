// ==================================== PROJECT IMPORTS =======================================
use super::image::Image;
use super::pixel::Pixel;

// ==================================== IMPORTS =======================================
use std::collections::HashMap;
use std::str;

// ===========================================================================
// This is a representation of the frame we're currently rendering
// ===========================================================================

pub struct Charset {
    bold: bool,
    ppm_charset: Image,
    //pub map: HashMap<String, usize>, //TODO mapping position
}

impl Charset {
    // ==================================== CONSTRUCTOR =======================================
    pub fn new(bold: bool) -> Charset {
        // TODO pass name argument
        let path = "ppm/octafont.ppm".to_string();
        let char_map: Charset = Charset {
            bold: bold,
            ppm_charset: Image::read_ppm_image(&path, false), //map: HashMap::new(),
        };
        char_map
    }
    // ==================================== PUBLIC FUNCTIONS =======================================
    pub fn set_bold(&mut self, b: bool) {
        self.bold = b;
    }
    pub fn get_text(&self, text: String) -> Image {
        //let mut image: Image; //TODO

        let mut text_matrix = Vec::new();

        // ------------------------------ LOOP OVER EACH LITERAL ------------------------------
        for (index, lit) in text.chars().enumerate() {
            //println!("index{}, literal{}", index, lit);
            // ------------------------------ GET PIXEL RANGE ------------------------------
            let row_start = 0;
            let row_stop = 10;
            let h = row_stop - row_start;
            assert!(h > 0, "negative height");

            let col_start = 0;
            let col_stop = 20;
            let w = col_stop - col_start;
            assert!(w > 0, "negative width");

            // ------------------------------ LOOP CHARACTERSET FOR GIVEN RANGE ------------------------------
            for col in col_start..col_stop {
                let mut column = Vec::new();
                for row in row_start..row_stop {
                    let pix = &self.ppm_charset.pixels[row][col];

                    let pixel = Pixel {
                        r: pix.r,
                        g: pix.g,
                        b: pix.b,
                    };
                    column.push(pixel);
                }
                text_matrix.push(column);
            }
        }
        // ------------------------------ TRANSPONATE FROM col<row<pixel>> tot row<col<pixel>> ------------------------------
        //todo
        // ------------------------------ CONVERT INTO AN IMAGE ------------------------------
        //todo
        // ------------------------------ (PRINT) AND RETURN IMAGE ------------------------------
        let mut image = Image::new(text_matrix);
        //text_matrix
        image.print_to_console();
        image
    }
    // ==================================== PRIVATE FUNCTIONS =======================================
}
