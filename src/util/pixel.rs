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
	pub fn average_channels(pixel: Pixel) -> Pixel{
		let rf: usize = 2126;
		let r: usize = pixel.r as usize;

		let gf: usize = 7152;
		let g: usize = pixel.g as usize;

		let bf: usize = 722;
		let b: usize = pixel.b as usize;


		let temp = (rf*r+gf*g+bf*b)/1000;


		let pix: Pixel = Pixel {
			r:temp as u8,
			g:temp as u8,
			b:temp as u8,
		};
		pix



	}


    // ==================================== PRIVATE FUNCTIONS =======================================

}