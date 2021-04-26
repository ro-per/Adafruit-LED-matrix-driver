#[derive(Clone)]
pub struct Pixel {
    // TODO: SHOULD NOT BE PRIVATE
    pub r: u8,
    pub g: u8,
    pub b: u8
}
impl Pixel {
	
	pub fn new() -> Pixel {
		let pixel: Pixel = Pixel {
			r:0,
			g:0,
			b:0,
		};
		pixel
	}
}