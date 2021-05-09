// ==================================== PROJECT IMPORTS =======================================
use super::image::Image;
use super::pixel::Pixel;

// ==================================== IMPORTS =======================================
use std::collections::HashMap;
use std::ops::Range;
use std::str;

// ===========================================================================
// This is a representation of the frame we're currently rendering
// ===========================================================================

pub struct Charset {
    bold: bool,
    ppm_charset: Image,
    pixel_map: HashMap<char, Vec<usize>>,
}

impl Charset {
    // ==================================== CONSTRUCTOR =======================================
    pub fn new(bold: bool) -> Charset {
        let path = "ppm/octafont.ppm".to_string();
        let char_map: Charset = Charset {
            bold: bold,
            ppm_charset: Image::read_ppm_image(&path, false), //map: HashMap::new(),
            pixel_map: init_pixel_map(),
        };
        char_map
    }
    // ==================================== PUBLIC FUNCTIONS =======================================
    pub fn set_bold(&mut self, b: bool) {
        self.bold = b;
    }
    pub fn get_text(&self, text: String) -> Image {
        let mut text_matrix = Vec::new();

        // ------------------------------ LOOP OVER EACH LITERAL ------------------------------
        for (index, lit) in text.chars().enumerate() {
            //println!("index{}, literal{}", index, lit);
            // ------------------------------ GET PIXEL RANGE ------------------------------
            //FIXME get range dynamically
            //ROWS
            let a = 0;
            let b = 9;
            //COLS
            let x;
            let y;
            let temp = self.pixel_map.get(&lit);
            match temp {
                // The division was valid
                Some(vec) => {
                    println!("Result: {} {}", vec[0], vec[1]);
                    x = vec[0];
                    y = vec[1];
                }
                // The division was invalid
                None => panic!("No col range found !"),
            }
            //let Some(col_vec) = self.pixel_map.get_key_value(&lit);

            // ------------------------------ LOOP CHARACTERSET FOR GIVEN RANGE ------------------------------
            for col in x..y {
                let mut column = Vec::new();
                for row in a..b {
                    let pix = self.ppm_charset.pixels[row][col];

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
        let text_matrix_transpose = matrix_transpose(text_matrix);
        // ------------------------------ CONVERT INTO AN IMAGE ------------------------------
        let image = Image::new(text_matrix_transpose);
        // ------------------------------ (PRINT) AND RETURN IMAGE ------------------------------
        //image.print_to_console();
        image
    }
    // ==================================== PRIVATE FUNCTIONS =======================================
}
// ==================================== GENERAL FUNCTIONS =======================================
fn init_pixel_map() -> HashMap<char, Vec<usize>> {
    let mut mapping = HashMap::new();

    mapping.insert('0', vec![90, 96]);
    mapping.insert('1', vec![96, 102]);

    return mapping;
}

fn matrix_transpose(m: Vec<Vec<Pixel>>) -> Vec<Vec<Pixel>> {
    let mut t = vec![Vec::with_capacity(m.len()); m[0].len()];
    for r in m {
        for i in 0..r.len() {
            t[i].push(r[i]);
        }
    }
    t
}
