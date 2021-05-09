// ==================================== PROJECT IMPORTS =======================================
use super::pixel::Pixel;
use crate::*;

// ==================================== EXTERN IMPORTS =======================================
use byteorder::ReadBytesExt;
use std::io::{Cursor, Read, Seek, SeekFrom};

// ===========================================================================
// REPRESENTATION OF THE 'RAW' IMAGE
// ===========================================================================
pub struct Image {
    // TODO: SHOULD NOT BE PRIVATE
    pub width: usize,  // 32
    pub height: usize, //16
    pub pixels: Vec<Vec<Pixel>>,
}

impl Image {
    // ==================================== CONSTRUCTOR =======================================
    pub fn new(row_col: Vec<Vec<Pixel>>) -> Image {
        let mut image: Image = Image {
            width: 0,
            height: 0,
            pixels: Vec::new(),
        };
        image.set_pixels(row_col);
        image
    }
    // ==================================== PUBLIC FUNCTIONS =======================================
    pub fn set_pixels(&mut self, pixels: Vec<Vec<Pixel>>) {
        self.pixels = pixels;
        self.width = self.pixels[0].len();
        self.height = self.pixels.len();
    }
    pub fn decode_ppm_image(
        cursor: &mut Cursor<Vec<u8>>,
        scaling: bool,
    ) -> Result<Image, std::io::Error> {
        let parent_method = "Image/decode_ppm_image:";
        println!("{} Decoding ppm image ...", parent_method);
        let mut image = Image {
            width: 0,
            height: 0,
            pixels: vec![],
        };
        /* INLEZEN VAN HET TYPE */
        let mut header: [u8; 2] = [0; 2]; // inlezen van karakters
        cursor.read(&mut header)?; // ? geeft error terug mee met result van de functie
        match &header {
            // & dient voor slice van te maken
            b"P6" => println!("\t P6 image"), // b zorgt ervoor dat je byte string hebt (u8 slice)
            _ => panic!("\t Not an P6 image"), //_ staat voor default branch
        }
        /* INLEZEN VAN BREEDTE EN HOOGTE */
        image.width = Image::read_number(cursor)?;
        image.height = Image::read_number(cursor)?;
        let _colour_range = Image::read_number(cursor)?;
        /* eventuele whitespaces na eerste lijn */
        Image::consume_whitespaces(cursor)?;
        /* body inlezen */

        for _ in 0..image.height {
            let mut row = Vec::new();
            for _ in 0..image.width {
                let red = cursor.read_u8()?;
                let green = cursor.read_u8()?;
                let blue = cursor.read_u8()?;

                let pixel = Pixel {
                    r: red,
                    g: green,
                    b: blue,
                };
                row.push(pixel);
            }
            image.pixels.push(row);
        }

        println!("{} Decoding done !", parent_method);
        if scaling {
            let y_scale = ROWS as f64 / image.height as f64;
            let x_scale = y_scale;
            Image::scale(&mut image, x_scale, y_scale);
        }

        Ok(image)
    }
    pub fn to_grey_scale(&mut self) {
        for row in 0..self.height {
            for col in 0..self.width {
                self.pixels[row][col].to_grey_scale();
            }
        }
    }
    pub fn invert_colors(&mut self) {
        for row in 0..self.height {
            for col in 0..self.width {
                self.pixels[row][col].invert_colors();
            }
        }
    }
    pub fn mirror_vertical(&mut self) {
        self.pixels.reverse();
    }
    pub fn mirror_horizontal(&mut self) {
        for i in 0..self.height {
            self.pixels[i].reverse();
        }
    }
    pub fn gamma_correction(&mut self) {
        for row in 0..self.height {
            for col in 0..self.width {
                self.pixels[row][col].gamma_correction();
            }
        }
    }
    pub fn print_to_console(&mut self) {
        let parent_method = "Image/print_to_console:";
        println!(
            "{} Printing image of size W{}xH{} ...",
            parent_method, self.width, self.height
        );
        let mut print: String = "".to_string();
        for col in 0..self.height {
            for row in 0..self.width {
                let pixel = &self.pixels[col][row];

                print += &pixel.get_primary_color_string();
            }
            print += "\n";
        }
        println!("{}", print);
        println!("{} Printing done ...", parent_method);
    }
    pub fn read_ppm_image(image_path: &String, scaling: bool) -> Image {
        println!("Image path = {}", image_path);
        let path = Path::new(&image_path);
        let display = path.display();
        let mut file = match File::open(&path) {
            Err(why) => panic!("Could not open file: {} (Reason: {})", display, why),
            Ok(file) => file,
        };
        // read the full file into memory. panic on failure
        let mut raw_file = Vec::new();
        file.read_to_end(&mut raw_file).unwrap();
        // construct a cursor so we can seek in the raw buffer
        let mut cursor = Cursor::new(raw_file);
        let image = match Image::decode_ppm_image(&mut cursor, scaling) {
            Ok(img) => img,
            Err(why) => panic!("Could not parse PPM file - Desc: {}", why),
        };
        image
    }
    pub fn transponate_image(&mut self) {
        // save old matrix
        let w = self.width;
        let h = self.height;
        let pixels_old = self.pixels.clone();

        // build new image
        let mut image = Image {
            width: h,
            height: w,
            pixels: vec![],
        };

        for x in 0..image.height {
            let mut row = Vec::new();
            for y in 0..image.width {
                let pixel = pixels_old[x][y];
                row.push(pixel);
            }
            image.pixels.push(row);
        }
    }

    // ==================================== PRIVATE FUNCTIONS =======================================
    fn read_number(cursor: &mut Cursor<Vec<u8>>) -> Result<usize, std::io::Error> {
        let parent_method = "Image/read_number:";
        Image::consume_whitespaces(cursor)?;

        let mut buff: [u8; 1] = [0];
        let mut v = Vec::new(); // vector waar je bytes gaat in steken

        loop {
            cursor.read(&mut buff)?;
            match buff[0] {
                b'0'..=b'9' => v.push(buff[0]),
                b' ' | b'\n' | b'\r' | b'\t' => break,
                _ => panic!("{} Not a valid image", parent_method),
            }
        }
        // byte vector omzetten
        let num_str: &str = std::str::from_utf8(&v).unwrap(); // unwrap gaat ok value er uit halen als het ok is, panic als het niet ok is
        let num = num_str.parse::<usize>().unwrap(); // unwrap dient voor errors
                                                     // return
        Ok(num)
        //return Ok(num); andere mogelijke return
    }

    fn consume_whitespaces(cursor: &mut Cursor<Vec<u8>>) -> Result<(), std::io::Error> {
        //Result<() : de lege haakjes betekend  niks returnen
        let mut buff: [u8; 1] = [0];

        loop {
            cursor.read(&mut buff)?;
            match buff[0] {
                b' ' | b'\n' | b'\r' | b'\t' => println!("\t consumed 1 whitespace"),
                _ => {
                    // je zit eigenlijk al te ver nu !!! zet cursor 1 terug
                    cursor.seek(SeekFrom::Current(-1))?;
                    break;
                }
            }
        }
        Ok(()) // () : de lege haakjes betekend  niks returnen
    }

    fn lin(s: f64, e: f64, t: f64) -> f64 {
        let y = s + (e - s) * t;
        y
    }

    fn bilin(c00: f64, c01: f64, c10: f64, c11: f64, tx: f64, ty: f64) -> u8 {
        let result = Image::lin(Image::lin(c00, c10, tx), Image::lin(c01, c11, tx), ty) as u8;
        result
    }

    fn scale(image: &mut Image, x_scale: f64, y_scale: f64) {
        // Scale the image to the appropriate height of the LED display
        // Kudos to: https://rosettacode.org/wiki/Bilinear_interpolation
        let mut pixels: Vec<Vec<Pixel>> = vec![];
        let new_width = (x_scale * image.width as f64) as u32;
        let new_height = (y_scale * image.height as f64) as u32;

        println!(
            "Scaling: {}x{} -> {}x{}",
            image.height, image.width, new_height, new_width
        );

        for i in 0..new_height {
            let mut pixel_row = Vec::new(); //TODO Nick init met alles zwart

            for j in 0..new_width {
                let gy = i as f64 / new_height as f64 * (image.height - 1) as f64;
                let gx = j as f64 / new_width as f64 * (image.width - 1) as f64;
                let gyi = gy as usize;
                let gxi = gx as usize;

                // surrounding pixels
                let c00 = &image.pixels[gyi][gxi];
                let c01 = &image.pixels[gyi + 1][gxi];
                let c10 = &image.pixels[gyi][gxi + 1];
                let c11 = &image.pixels[gyi + 1][gxi + 1];

                let r = Image::bilin(
                    c00.r as f64,
                    c01.r as f64,
                    c10.r as f64,
                    c11.r as f64,
                    gx - gxi as f64,
                    gy - gyi as f64,
                );
                let g = Image::bilin(
                    c00.g as f64,
                    c01.g as f64,
                    c10.g as f64,
                    c11.g as f64,
                    gx - gxi as f64,
                    gy - gyi as f64,
                );
                let b = Image::bilin(
                    c00.b as f64,
                    c01.b as f64,
                    c10.b as f64,
                    c11.b as f64,
                    gx - gxi as f64,
                    gy - gyi as f64,
                );

                let pixel = Pixel { r: r, g: g, b: b };
                pixel_row.push(pixel.clone());
            }
            pixels.push(pixel_row.clone());
        }

        image.width = new_width as usize;
        image.height = new_height as usize;
        image.pixels = pixels;
    }

    /* fn show_image(image: &Image) {
        let sdl = sdl2::init().unwrap();
        let video_subsystem = sdl.video().unwrap();
        let display_mode = video_subsystem.current_display_mode(0).unwrap();
        let w = match display_mode.w as u32 > image.width {
            true => image.width,
            false => display_mode.w as u32
        };
        let h = match display_mode.h as u32 > image.height {
            true => image.height,
            false => display_mode.h as u32
        };
        let window = video_subsystem
            .window("Image", w, h)
            .build()
            .unwrap();
        let mut canvas = window
            .into_canvas()
            .present_vsync()
            .build()
            .unwrap();
        let black = sdl2::pixels::Color::RGB(0, 0, 0);
        let mut event_pump = sdl.event_pump().unwrap();
        // render image
        canvas.set_draw_color(black);
        canvas.clear();
        for r in 0..image.height {
            for c in 0..image.width {
                let pixel = &image.pixels[r as usize][c as usize];
                canvas.set_draw_color(Color::RGB(pixel.r as u8, pixel.g as u8, pixel.b as u8));
                canvas.fill_rect(Rect::new(c as i32, r as i32, 1, 1)).unwrap();
            }
        }
        canvas.present();

        'main: loop {
            for event in event_pump.poll_iter() {
                match event {
                    sdl2::event::Event::Quit {..} => break 'main,
                    _ => {},
                }
            }
            sleep(Duration::new(0, 250000000));
        }
    } */
}
