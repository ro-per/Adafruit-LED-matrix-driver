// ==================================== PROJECT IMPORTS =======================================

// ==================================== EXTERN IMPORTS =======================================

// ===========================================================================
// REPRESENTATION OF A SINGLE PIXEL
// ===========================================================================
#[derive(Clone)]
pub struct Pixel {
    // TODO: SHOULD NOT BE PRIVATE
    pub r: u8,
    pub g: u8,
    pub b: u8
}
impl Pixel {
	// ==================================== CONSTRUCTOR =======================================
	pub fn new() -> Pixel {
		let pixel: Pixel = Pixel {
			r:0,
			g:0,
			b:0,
		};
		pixel
	}
	// ==================================== PUBLIC FUNCTIONS =======================================
	// normal averaging of channels (https://www.kdnuggets.com/2019/12/convert-rgb-image-grayscale.html)
	pub fn to_grey_scale(&mut self){
		let mut temp: usize =0;
		temp +=(self.r as usize)*2126;
		temp +=(self.g as usize)*7152;
		temp +=(self.b as usize)*722;

		let  temp2: u8  = (temp / 3000) as u8;

		self.r = temp2;
		self.g = temp2;
		self.b = temp2;
	}
	pub fn invert_colors(&mut self){
		self.r = 255- self.r;
		self.g = 255- self.g;
		self.b = 255 - self.b;
	}
	pub fn to_string(&self){
		println!("Pixel: R{} G{} B{}", self.r,self.g,self.b);

	}

	pub fn gamma_correction(&mut self){
		self.r = Pixel::raw_color_to_full_color(self.r);
		self.g = Pixel::raw_color_to_full_color(self.g);
		self.b = Pixel::raw_color_to_full_color(self.b);
	}
    // ==================================== PRIVATE FUNCTIONS =======================================
	fn raw_color_to_full_color(raw_color: u8) -> u8{
        //let full_color = ((raw_color as u32)* ((1<<COLOR_DEPTH) -1)/255) as u16;
        let gamma_correction : f32 = 1.75;
        
        let _raw_color_float = raw_color as f32;
        let max_value_float = 255 as f32;
        
        let full_color = (max_value_float * (raw_color as f32 / max_value_float).powf(gamma_correction)) as u8;
        
        full_color
    }



}