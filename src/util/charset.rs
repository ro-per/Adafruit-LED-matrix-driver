// ==================================== PROJECT IMPORTS =======================================
use super::image::Image;
use super::pixel::Pixel;
use crate::NUMBER_SPACES;

// ==================================== IMPORTS =======================================
use rand::prelude::*;
use std::collections::HashMap;

// ===========================================================================
// This is a representation of the frame we're currently rendering
// ===========================================================================

pub struct Charset {
    pub ppm_charset: Image,
    pixel_map: HashMap<char, Vec<usize>>,
}

impl Charset {
    // ==================================== CONSTRUCTOR =======================================
    pub fn new() -> Charset {
        let path = "ppm/octafont.ppm".to_string();
        let char_map: Charset = Charset {
            ppm_charset: Image::read_ppm_image(&path, false), //map: HashMap::new(),
            pixel_map: init_pixel_map(),
        };
        char_map
    }
    // ==================================== PUBLIC FUNCTIONS =======================================
    pub fn show_char_set(&mut self) {
        self.ppm_charset.print_to_console();
    }
    pub fn get_text(&self, text: String, random_rgb: bool) -> Image {
        let mut rand_gen = rand::thread_rng();
        let mut text_matrix = Vec::new();
        let text_lower = text.to_ascii_lowercase();

        // ------------------------------ LOOP OVER EACH LITERAL ------------------------------
        for (_index, lit) in text_lower.chars().enumerate() {
            //println!("index{}, literal{}", index, lit);
            // ------------------------------ GET PIXEL RANGE ------------------------------
            //ROWS
            let a = 10;
            let b = 17;
            //COLS
            let x;
            let y;
            let temp = self.pixel_map.get(&lit);
            match temp {
                Some(vec) => {
                    // println!("Result: {} {}", vec[0], vec[1]);
                    x = vec[0];
                    y = vec[1];
                }
                None => panic!("Character ```{}``` not available!", &lit),
            }

            // ------------------------------ LOOP CHARACTERSET FOR GIVEN RANGE ------------------------------
            let space_x: bool = x == 0;
            let space_y: bool = y == 0;
            if space_x & space_y {
                for _ in 0..NUMBER_SPACES {
                    let mut column = Vec::new();
                    for _ in a..b {
                        let pixel = Pixel { r: 0, g: 0, b: 0 };
                        column.push(pixel);
                    }
                    text_matrix.push(column);
                }
            } else {
                let mut p = Pixel {
                    r: 255,
                    g: 255,
                    b: 255,
                };
                if random_rgb {
                    let x: f64 = rand_gen.gen();
                    if x < 0.33 {
                        p = Pixel { r: 255, g: 0, b: 0 };
                    } else if x > 0.66 {
                        p = Pixel { r: 0, g: 255, b: 0 };
                    } else {
                        p = Pixel { r: 0, g: 0, b: 255 };
                    }
                }
                for col in x..y {
                    let mut column = Vec::new();
                    // ACTUAL TEXT
                    for row in a..b {
                        let pix = self.ppm_charset.pixels[row][col];

                        let mut pixel = Pixel {
                            r: pix.r,
                            g: pix.g,
                            b: pix.b,
                        };
                        pixel.invert_colors();
                        if pixel.is_white() {
                            pixel.set_color(p);
                        }

                        column.push(pixel);
                    }
                    //BLACK PIX //TODO
                    for _ in 0..2 {
                        let pixel = Pixel { r: 0, g: 0, b: 0 };
                        column.push(pixel);
                    }

                    // ACTUAL TEXT
                    for row in a..b {
                        let pix = self.ppm_charset.pixels[row][col];

                        let mut pixel = Pixel {
                            r: pix.r,
                            g: pix.g,
                            b: pix.b,
                        };
                        pixel.invert_colors();
                        if pixel.is_white() {
                            pixel.set_color(p);
                        }

                        column.push(pixel);
                    }

                    text_matrix.push(column);
                }
            }
        }
        // ------------------------------ TRANSPONATE FROM col<row<pixel>> tot row<col<pixel>> ------------------------------
        let text_matrix_transpose = matrix_transpose(text_matrix);
        // ------------------------------ CONVERT INTO AN IMAGE ------------------------------
        let image = Image::new(text_matrix_transpose);
        // ------------------------------ (PRINT) AND RETURN IMAGE ------------------------------
        //image.print_to_console(); //BUG Romeo
        image
    }
    // ==================================== PRIVATE FUNCTIONS =======================================
}
// ==================================== GENERAL FUNCTIONS =======================================
fn init_pixel_map() -> HashMap<char, Vec<usize>> {
    let mut mapping = HashMap::new();

    mapping.insert(' ', vec![0, 0]);

    //Symbols
    mapping.insert('!', vec![1, 2]);
    mapping.insert('"', vec![4, 7]);
    mapping.insert('#', vec![10, 16]);
    mapping.insert('$', vec![19, 24]);
    mapping.insert('%', vec![28, 37]);
    mapping.insert('&', vec![40, 45]);
    mapping.insert('\'', vec![47, 48]);
    mapping.insert('(', vec![50, 53]);
    mapping.insert(')', vec![55, 58]);
    mapping.insert('*', vec![60, 65]);
    mapping.insert('+', vec![68, 73]);
    mapping.insert(',', vec![75, 77]);
    mapping.insert('-', vec![78, 81]);
    mapping.insert('.', vec![82, 83]);
    mapping.insert('/', vec![85, 90]);
    mapping.insert(':', vec![154, 155]);
    mapping.insert(';', vec![157, 159]);
    mapping.insert('<', vec![161, 165]);
    mapping.insert('=', vec![166, 170]);
    mapping.insert('>', vec![171, 175]);
    mapping.insert('?', vec![176, 180]);
    mapping.insert('@', vec![182, 188]);
    mapping.insert('[', vec![370, 372]);
    mapping.insert('\\', vec![374, 379]);
    mapping.insert(']', vec![380, 382]);
    mapping.insert('^', vec![384, 389]);
    mapping.insert('_', vec![390, 395]);
    mapping.insert('`', vec![396, 398]);
    mapping.insert('{', vec![549, 552]);
    mapping.insert('|', vec![554, 555]);
    mapping.insert('}', vec![556, 559]);
    mapping.insert('~', vec![561, 565]);
    mapping.insert('€', vec![614, 619]);

    // Numbers
    mapping.insert('0', vec![91, 95]);
    mapping.insert('1', vec![98, 100]);
    mapping.insert('2', vec![102, 106]);
    mapping.insert('3', vec![109, 113]);
    mapping.insert('4', vec![115, 120]);
    mapping.insert('5', vec![122, 126]);
    mapping.insert('6', vec![128, 132]);
    mapping.insert('7', vec![134, 139]);
    mapping.insert('8', vec![141, 145]);
    mapping.insert('9', vec![148, 152]);

    // Letters (ALL UPPERCASE)
    mapping.insert('a', vec![191, 196]);
    mapping.insert('b', vec![199, 203]);
    mapping.insert('c', vec![206, 210]);
    mapping.insert('d', vec![212, 216]);
    mapping.insert('e', vec![218, 222]);
    mapping.insert('f', vec![224, 228]);
    mapping.insert('g', vec![230, 234]);
    mapping.insert('h', vec![236, 240]);
    mapping.insert('i', vec![243, 244]);
    mapping.insert('j', vec![246, 250]);
    mapping.insert('k', vec![252, 257]);
    mapping.insert('l', vec![259, 263]);
    mapping.insert('m', vec![265, 270]);
    mapping.insert('n', vec![273, 278]);
    mapping.insert('o', vec![281, 285]);
    mapping.insert('p', vec![288, 292]);
    mapping.insert('q', vec![295, 301]);
    mapping.insert('r', vec![303, 307]);
    mapping.insert('s', vec![310, 314]);
    mapping.insert('t', vec![317, 322]);
    mapping.insert('u', vec![324, 328]);
    mapping.insert('v', vec![331, 336]);
    mapping.insert('w', vec![339, 346]);
    mapping.insert('x', vec![348, 353]);
    mapping.insert('y', vec![356, 361]);
    mapping.insert('z', vec![363, 368]);

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
