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
	pub fn toString(&self){
		println!("Pixel: R{} G{} B{}", self.r,self.g,self.b);

	}


    // ==================================== PRIVATE FUNCTIONS =======================================

}