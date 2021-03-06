// ==================================== PROJECT IMPORTS =======================================

// ==================================== EXTERN IMPORTS =======================================

// ===========================================================================
// REPRESENTATION OF A SINGLE PIXEL
// ===========================================================================
#[derive(Copy, Clone)]
pub struct Pixel {
    pub r: u16,
    pub g: u16,
    pub b: u16,
}
impl Pixel {
    // ==================================== CONSTRUCTOR =======================================
    pub fn new() -> Pixel {
        let pixel: Pixel = Pixel { r: 0, g: 0, b: 0 };
        pixel
    }
    // ==================================== PUBLIC FUNCTIONS =======================================
    // normal averaging of channels (https://www.kdnuggets.com/2019/12/convert-rgb-image-grayscale.html)
    pub fn to_grey_scale(&mut self) {
        let mut temp: usize = 0;
        temp += (self.r as usize)*2126;
        temp += (self.g as usize)*7152;
        temp += (self.b as usize)*722;

        let temp2: u8 = (temp / 3000) as u8;

        self.r = temp2 as u16;
        self.g = temp2 as u16;
        self.b = temp2 as u16;
    }
    pub fn invert_colors(&mut self) {
        self.r = 255 - self.r;
        self.g = 255 - self.g;
        self.b = 255 - self.b;
    }
    pub fn print_to_console(&self) {
        print!("R{}G{}B{}", self.r, self.g, self.b);
    }
    pub fn get_primary_color_string(&self) -> String {
        let red = self.r > 254;
        let green = self.g > 254;
        let blue = self.b > 254;
        let white = red & green & blue;
        let message: String;

        if white {
            message = "o".to_string();
        } else if red {
            message = "R".to_string();
        } else if green {
            message = "G".to_string();
        } else if blue {
            message = "B".to_string();
        } else {
            message = " ".to_string();
        }
        //eprint!("{}", message);
        message
    }
    pub fn set_color(&mut self, p: Pixel) {
        self.r = p.r;
        self.g = p.g;
        self.b = p.b;
    }
    pub fn is_white(&self) -> bool {
        let red = self.r > 254;
        let green = self.g > 254;
        let blue = self.b > 254;
        let white = red & green & blue;
        white
    }

    pub fn gamma_correction(&mut self) {
        self.r = self.raw_color_to_full_color(self.r);
        self.g = self.raw_color_to_full_color(self.g);
        self.b = self.raw_color_to_full_color(self.b);
    }
    // ==================================== PRIVATE FUNCTIONS =======================================
    fn raw_color_to_full_color(&self, raw_color: u16) -> u16 {
        //let full_color = ((raw_color as u32)* ((1<<COLOR_DEPTH) -1)/255) as u16;
        let gamma_correction: f32 = 1.75;

        let _raw_color_float = raw_color as f32;
        let max_value_float = 255 as f32;

        let full_color =
            (max_value_float * (raw_color as f32 / max_value_float).powf(gamma_correction)) as u16;

        full_color
    }
}
