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
    name: String,
    pub map: HashMap<String, usize>,
}

impl Charset {
    // ==================================== CONSTRUCTOR =======================================
    pub fn new() -> Charset {
        // TODO pass name argument
        let char_map: Charset = Charset {
            name: "Regular".to_string(),
            map: HashMap::new(),
        };
        char_map
    }
    // ==================================== PUBLIC FUNCTIONS =======================================

    pub fn init_map(&mut self) {
        self.map.insert(String::from("A"), 10);
        self.map.insert(String::from("B"), 100);
        self.map.insert(String::from("C"), 100);
    }
    pub fn get_image(&mut self, string: String) {
        let value = self.map.get(&string);

        if let temp = Some(value) {
            println!("Print out {:?}", value);
        };
    }
}
