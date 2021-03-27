/* -------------------------------------------- LAB 4 CRATES --------------------------------------------*/
extern crate libc;
extern crate time;
extern crate ctrlc;
#[macro_use] extern crate simple_error;
extern crate shuteye;
extern crate mmap;
extern crate nix;

/* -------------------------------------------- LAB 3 IMPORTS --------------------------------------------*/
use std::io::{Error, ErrorKind,Read, Cursor,Seek,SeekFrom};
use std::path::Path;
use std::fs::File;
pub use byteorder::ReadBytesExt;
//use sdl2::pixels::Color;
//use sdl2::rect::Rect;
use shuteye::sleep;
use std::time::Duration;

/* -------------------------------------------- LAB 4 IMPORTS --------------------------------------------*/
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::{fs::OpenOptions, os::unix::fs::OpenOptionsExt};
use std::os::unix::io::AsRawFd;
use std::io::prelude::*;
use mmap::{MemoryMap, MapOption};

/* -------------------------------------------- LAB 3 STRUCTS --------------------------------------------*/
/* #[derive(Clone)]
struct Pixel {
    r: u8,
    g: u8,
    b: u8
}

struct Image {
    width: u32,
    height: u32,
    pixels: Vec<Vec<Pixel>>
} */

/* -------------------------------------------- LAB 4 STRUCTS --------------------------------------------*/
struct GPIO {
    gpio_map_: Option<MemoryMap>,
    output_bits_: u32,
    input_bits_: u32,
    slowdown_: u32,                         // Please refer to the GPIO_SetBits and GPIO_ClearBits functions in the reference implementation to see how this is used.
    gpio_port_: *mut u32,                   // A raw pointer that points to the base of the GPIO register file
    gpio_set_bits_: *mut u32,               // A raw pointer that points to the pin output register (see section 2.1 in the assignment)
    gpio_clr_bits_: *mut u32,               // A raw pointer that points to the pin output clear register (see section 2.1)
    gpio_read_bits_: *mut u32,              // A raw pointer that points to the pin level register (see section 2.1)
    row_mask: u32,
    bitplane_timings: [u32; COLOR_DEPTH]
}

#[derive(Clone)]
struct Pixel {
    r: u8,
    g: u8,
    b: u8
}

// This is a representation of the "raw" image
struct Image {
    width: usize,
    height: usize,
    pixels: Vec<Vec<Pixel>>
}

// This is a representation of the frame we're currently rendering
struct Frame {
    pos: usize,
    pixels: Vec<Vec<Pixel>>
}

// Use this struct to implement high-precision nanosleeps
struct Timer {
    _timemap: Option<MemoryMap>,
    timereg: *mut u32 // a raw pointer to the 1Mhz timer register (see section 2.5 in the assignment)
}

// ============================================================================
// GPIO configuration parameters for the raspberry pi 3
// ============================================================================

const BCM2709_PERI_BASE: u64 = 0x3F000000;
const GPIO_REGISTER_OFFSET: u64 = 0x200000;
const TIMER_REGISTER_OFFSET: u64 = 0x3000;
const REGISTER_BLOCK_SIZE: u64 = 4096;
const COLOR_DEPTH: usize = 8;

const PIN_OE  : u64 = 4;
const PIN_CLK : u64 = 17;
const PIN_LAT : u64 = 21;
const PIN_A   : u64 = 22;
const PIN_B   : u64 = 26;
const PIN_C   : u64 = 27;
const PIN_D   : u64 = 20;
const PIN_E   : u64 = 24;
const PIN_R1  : u64 = 5;
const PIN_G1  : u64 = 13;
const PIN_B1  : u64 = 6;
const PIN_R2  : u64 = 12;
const PIN_G2  : u64 = 16;
const PIN_B2  : u64 = 23;

// Convenience macro for creating bitmasks. See comment above "impl GPIO" below
macro_rules! GPIO_BIT {
    ($bit:expr) => {
        1 << $bit
    };
}

// Use this bitmask for sanity checks
const VALID_BITS: u64 = GPIO_BIT!(PIN_OE) | GPIO_BIT!(PIN_CLK) | GPIO_BIT!(PIN_LAT) |
    GPIO_BIT!(PIN_A)  | GPIO_BIT!(PIN_B)  | GPIO_BIT!(PIN_C)   | GPIO_BIT!(PIN_D)   | GPIO_BIT!(PIN_E) |
    GPIO_BIT!(PIN_R1) | GPIO_BIT!(PIN_G1) | GPIO_BIT!(PIN_B1)  |
    GPIO_BIT!(PIN_R2) | GPIO_BIT!(PIN_G2) | GPIO_BIT!(PIN_B2);

//************************************ADDED SELF ************************************
const ROWS: usize = 16;
const SUB_PANELS_: usize = 2;
const COLUMNS: usize = 32;
const TIMER_OVERFLOW: u32 =4294967295;

type gpio_bits_t = u32;

// ============================================================================
// mmap_bcm_register - convenience function used to map the GPIO register block
// ============================================================================

fn mmap_bcm_register(register_offset: usize) -> Option<MemoryMap> {

    let mem_file =
        match OpenOptions::new()
            .read(true)
            .write(true)
            .custom_flags(libc::O_SYNC)
            .open("/dev/mem") {
            Err(why) => panic!("couldn't open /dev/mem: {}", why),
            Ok(file) => file
        };

    let mmap_options = &[
        MapOption::MapNonStandardFlags(libc::MAP_SHARED),
        MapOption::MapReadable,
        MapOption::MapWritable,
        MapOption::MapFd(mem_file.as_raw_fd()),
        MapOption::MapOffset(BCM2709_PERI_BASE as usize + register_offset as usize)
    ];

    let result = MemoryMap::new(REGISTER_BLOCK_SIZE as usize, mmap_options).unwrap();

    return match result.data().is_null() {
        true => {
            eprintln!("mmap error: {}", std::io::Error::last_os_error());
            eprintln!("Pi3: MMapping from base 0x{:X}, offset 0x{:X}", BCM2709_PERI_BASE, register_offset);
            None
        },
        false => Some(result)
    };

    // NOTE/WARNING: When a MemoryMap struct is dropped, the mapped 
    // memory region is automatically unmapped!
}

//
// NOTE/WARNING: In many cases, particularly those where you need to set or clear 
// multiple bits at once, it is convenient to store multiple pin numbers in one bit 
// mask value. If you want to simultaneously set PIN_A and PIN_C to high, for example, 
// you should probably create a bit mask with the positions of PIN_A and PIN_C set to 1, 
// and all other positions set to 0. You can do this using the GPIO_BIT! macro.
//
// In this example, you would do something like:
//     let pin_mask = GPIO_BIT!(PIN_A) | GPIO_BIT!(PIN_C);
//     io.set_bits(pin_mask);
//
impl GPIO {

    //
    // configures pin number @pin_num as an output pin by writing to the 
    // appropriate Function Select register (see section 2.1).
    // 
    // NOTE/WARNING: This method configures one pin at a time. The @pin_num argument 
    // that is expected here is really a pin number and not a bitmask!
    //
    // Doing something like:
    //     io.configure_output_pin(VALID_BITS);
    // Would be WRONG! This call would make the program crash.
    //
    // Doing something like:
    //     if GPIO_BIT!(PIN_A) & VALID_BITS {
    //         io.configure_output_pin(PIN_A);
    //     }
    // Would be OK!
    //
    fn configure_output_pin(self: &mut GPIO, pin_num: u64) {
        let register_num = (pin_num / 10) as isize;
        let register_ref = unsafe { self.gpio_port_.offset(register_num) };
        // NOTE/WARNING: When reading from or writing to MMIO memory regions, you MUST 
        // use the std::ptr::read_volatile and std::ptr::write_volatile functions
        let current_val = unsafe { std::ptr::read_volatile(register_ref) };
        // the bit range within the register is [(pin_num % 10) * 3 .. (pin_num % 10) * 3 + 2]
        // we need to set these bits to 001
        let new_val = (current_val & !(7 << ((pin_num % 10)*3))) | (1 << ((pin_num % 10)*3));
        // NOTE/WARNING: When reading from or writing to MMIO memory regions, you MUST 
        // use the std::ptr::read_volatile and std::ptr::write_volatile functions
        unsafe { std::ptr::write_volatile(register_ref, new_val) };
    }

    fn init_outputs(self: &mut GPIO, mut outputs: u32) -> u32 {
        self.configure_output_pin(4 as u64);
        self.configure_output_pin(18 as u64);

        outputs &= !(self.output_bits_ | self.input_bits_);

		let valid_output = outputs & (VALID_BITS as u32);

        for b in 0..28 {
            if (GPIO_BIT!(b) & valid_output) != 0 {
                self.configure_output_pin(b as u64);
            }
        }
        valid_output
    }

    fn set_bits(self: &mut GPIO, value: u32) {
        unsafe {
            std::ptr::write_volatile(self.gpio_set_bits_, value);
            for _iter in 0..self.slowdown_ {
                std::ptr::write_volatile(self.gpio_set_bits_, value);
            }
        }
    }

    fn clear_bits(self: &mut GPIO, value: u32) {
        unsafe {
            std::ptr::write_volatile(self.gpio_clr_bits_, value);
            for _iter in 0..self.slowdown_ {
                std::ptr::write_volatile(self.gpio_clr_bits_, value);
            }
        }
    }

    // Write all the bits of @value that also appear in @mask. Leave the rest untouched.
    // @value and @mask are bitmasks
    fn write_masked_bits(
        self: &mut GPIO,
        value: u32,
        mask: u32
    ) {
        self.clear_bits(!value & mask);
        self.set_bits(value & mask);
    }

    fn new(slowdown: u32) -> GPIO {

        // Map the GPIO register file. See section 2.1 in the assignment for details
        let map = mmap_bcm_register(GPIO_REGISTER_OFFSET as usize);

        // Initialize the GPIO struct with default values
        let mut io: GPIO = GPIO {
            gpio_map_: None,
            output_bits_: 0,
            input_bits_: 0,
            slowdown_: slowdown,
            gpio_port_: 0 as *mut u32,
            gpio_set_bits_: 1 as *mut u32,
            gpio_clr_bits_: 0 as *mut u32,
            gpio_read_bits_: 0 as *mut u32,
            row_mask: 0,
            bitplane_timings: [0; COLOR_DEPTH]
        };

        match &map {
            Some(m) => {
                unsafe {
                    io.gpio_port_ = m.data() as *mut u32;

                    io.gpio_set_bits_ = io.gpio_port_.offset(0x1C / 4); 
                    io.gpio_clr_bits_ = io.gpio_port_.offset(0x28/ 4); 
                    io.gpio_read_bits_ = io.gpio_port_.offset(0x34/4);
                }

                let mut all_used_bits : gpio_bits_t = 0;

                all_used_bits |= GPIO_BIT!(PIN_OE) | GPIO_BIT!(PIN_CLK) | GPIO_BIT!(PIN_LAT);
                all_used_bits |= GPIO_BIT!(PIN_R1) | GPIO_BIT!(PIN_G1) | GPIO_BIT!(PIN_B1) | GPIO_BIT!(PIN_R2) | GPIO_BIT!(PIN_G2) | GPIO_BIT!(PIN_B2);
                
                io.row_mask = GPIO_BIT!(PIN_A);

                if ROWS / SUB_PANELS_ > 2 {
                    io.row_mask |= GPIO_BIT!(PIN_B);
                }
                if ROWS / SUB_PANELS_ > 4 {
                    io.row_mask |= GPIO_BIT!(PIN_C);
                }
                if ROWS / SUB_PANELS_ > 8 {
                    io.row_mask |= GPIO_BIT!(PIN_D);
                }
                if ROWS / SUB_PANELS_ > 16 {
                    io.row_mask |= GPIO_BIT!(PIN_E);
                }
                
                all_used_bits |= io.row_mask;

                let result :u32 = io.init_outputs(all_used_bits);
                assert!(result == all_used_bits);

                let mut timing_ns: u32 = 1000;
                for b in 0..COLOR_DEPTH{
                    io.bitplane_timings[b] = timing_ns;
                    timing_ns *= 2;
                }
            },
            None => {}
        }

        io.gpio_map_ = map;
        io
    }

    // Calculates the pins we must activate to push the address of the specified double_row
    fn get_row_bits(self: &GPIO, double_row: u8) -> u32 {
        let mut pin : u32 = 0;
        if double_row & 0x01 != 0 {
            pin |= GPIO_BIT!(PIN_A);
        }
        if double_row & 0x02 != 0 {
            pin |= GPIO_BIT!(PIN_B);
        }
        if double_row & 0x04 != 0 {
            pin |= GPIO_BIT!(PIN_C);
        }
        pin
    }
    fn get_plane_bits(self: &GPIO, top : &Pixel, bot : &Pixel, plane : i8) -> u32 {
       
        let mut out: gpio_bits_t = 0;
        if top.r & (1<<plane) !=0 {
           out |= GPIO_BIT!(PIN_R1);
        }
        if top.g & (1<<plane) !=0 {
            out |= GPIO_BIT!(PIN_G1);
        }
        if top.b & (1<<plane) !=0 {
            out |= GPIO_BIT!(PIN_B1);
        }
        if bot.r & (1<<plane) !=0 {
            out |= GPIO_BIT!(PIN_R2);
        }
        if bot.g & (1<<plane) !=0 {
            out |= GPIO_BIT!(PIN_G2);
        }
        if bot.b & (1<<plane) !=0 {
            out |= GPIO_BIT!(PIN_B2);
        }

       out
    }
}

impl Timer {
    // Reads from the 1Mhz timer register (see Section 2.5 in the assignment)
    unsafe fn read(self: &Timer) -> u32 {
        let time:u32 = std::ptr::read_volatile(self.timereg);
        time
    }

    fn new() -> Timer {
        let map = mmap_bcm_register(TIMER_REGISTER_OFFSET as usize);

        let mut timer: Timer = Timer {
            _timemap: None,
            timereg: 0 as *mut u32,
        };

        match &map {
            &Some(ref map) => {
                unsafe {
                    timer.timereg = map.data() as *mut u32;
                    timer.timereg.offset(1);
                }
            }
            &None => {}
        };
        timer
    }

    // High-precision sleep function (see section 2.5 in the assignment)
    // NOTE/WARNING: Since the raspberry pi's timer frequency is only 1Mhz, 
    // you cannot reach full nanosecond precision here. You will have to think
    // about how you can approximate the desired precision. Obviously, there is
    // no perfect solution here.
    fn nanosleep(self: &Timer, mut nanos: u32) {
        let k_jitter_allowance = 60 * 1000 + 0;

        if nanos > k_jitter_allowance{
       
           let before:u32= unsafe {self.read()};
           let sleep_time = Duration::new(0, nanos - k_jitter_allowance);
           sleep(sleep_time);
           let after:u32 = unsafe {self.read()};
           let time_passed: u64 ;

           if after > before {
               time_passed = 1000 * (after - before) as u64;
           }
           else{
               time_passed = 1000 * ( TIMER_OVERFLOW - before + after) as u64;
           }
           if time_passed > nanos as u64 {
               return
           }
           else{
               nanos -= time_passed as u32;
           }
       }

       if nanos < 20 {
           return;
       }

       let start_time: u32 = unsafe { self.read() };
       let mut current_time: u32 = start_time;

       while start_time + (nanos * 1000) <= current_time {
           current_time = unsafe { self.read() };
       }
       return;
    }
}

// The Frame should contain the pixels that are currently shown
// on the LED board. In most cases, the Frame will have less pixels
// than the input Image!
impl Frame {
    fn new() -> Frame {
        let frame: Frame = Frame {
            pos: 0,
            pixels: vec![vec![Pixel::new(); COLUMNS as usize]; ROWS as usize],
        };

        frame
    }

    fn next_image_frame(&mut self, image: &Image) {
        
        let blokgrootte_breedte = image.width/(COLUMNS*3);
        let blokgrootte_lengte = image.height/ROWS;
        
        for row in 0..ROWS {
            for col in 0..COLUMNS {
                let image_position = ((self.pos + col*blokgrootte_breedte) as usize % image.width) as usize;
                
                //lijntje hier onder is nog 'raw'
                //self.pixels[row][col] = image.pixels[row][image_position].clone();
            
                //raw -> full
                let raw_color = image.pixels[row*blokgrootte_lengte as usize][image_position].clone();

                //rgb waarden naar full color converteren en er dan in zetten (gamma correction)
                self.pixels[row as usize][col as usize].r = self.raw_color_to_full_color(raw_color.r);
                self.pixels[row as usize][col as usize].g = self.raw_color_to_full_color(raw_color.g);
                self.pixels[row as usize][col as usize].b = self.raw_color_to_full_color(raw_color.b);
            
            }
        }

        self.pos = self.pos + 1;
        if self.pos >= image.width as usize {
            self.pos = 0;
        }
    }

    //voor gamma correction
    fn raw_color_to_full_color(self: &mut Frame, raw_color: u8) -> u8{
        //let full_color = ((raw_color as u32)* ((1<<COLOR_DEPTH) -1)/255) as u16;
        let gammaCorrection : f32 = 1.75;
        
        let raw_color_float = raw_color as f32;
        let max_value_float = 255 as f32;
        
        let full_color = (max_value_float * (raw_color as f32 / max_value_float).powf(gammaCorrection)) as u8;
        
        full_color
    }

    fn clear_frame(self:&mut Frame){
        self.pixels=vec![vec![Pixel::new(); COLUMNS as usize]; ROWS as usize];
    }
}

// TODO: Add your PPM parser here
// NOTE/WARNING: Please make sure that your implementation can handle comments in the PPM file
// You do not need to add support for any formats other than P6
// You may assume that the max_color value is always 255, but you should add sanity checks
// to safely reject files with other max_color values
impl Image {
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
    
    
    fn decode_ppm_image(cursor: &mut Cursor<Vec<u8>>) -> Result<Image, std::io::Error> {
        let mut image = Image { 
            width: 0,
            height: 0,
            pixels: vec![]
        };
    
        /* INLEZEN VAN HET TYPE */
        let mut header: [u8;2]=[0;2]; // inlezen van karakters
        cursor.read(&mut header)?; // ? geeft error terug mee met result van de functie
        match &header{ // & dient voor slice van te maken
            b"P6" => println!("P6 image"),  // b zorgt ervoor dat je byte string hebt (u8 slice)
            _ => panic!("Not an P6 image")  //_ staat voor default branch
        }
    
        /* INLEZEN VAN BREEDTE EN HOOGTE */
        image.width=Image::read_number(cursor)?;
        image.height=Image::read_number(cursor)?;
        let colourRange = Image::read_number(cursor)?;
    
        /* eventuele whitespaces na eerste lijn */
        Image::consume_whitespaces(cursor)?;
    
        /* body inlezen */
    
        for _ in 0.. image.height{
            let mut row = Vec::new();
            for _ in 0..image.width{
                let red = cursor.read_u8()?;
                let green = cursor.read_u8()?;
                let blue = cursor.read_u8()?;
                
                row.push(Pixel{r:red,g:green,b:blue});
            }
            image.pixels.push(row);
        }
    
    
    
    
        // TODO: Parse the image here
    
        Ok(image)
    }
    
    fn read_number(cursor: &mut Cursor<Vec<u8>>)-> Result<usize,std::io::Error>{
        Image::consume_whitespaces(cursor)?;
    
        let mut buff: [u8;1] = [0];
        let mut v = Vec::new(); // vector waar je bytes gaat in steken
    
        loop{
            cursor.read(& mut buff)?;
            match buff[0]{
                b'0'..= b'9' => v.push(buff[0]),
                b' ' | b'\n' | b'\r' | b'\t' => break,
                _ => panic!("Not a valid image")
            }
        }
        // byte vector omzetten
        let num_str: &str = std::str::from_utf8(&v).unwrap(); // unwrap gaat ok value er uit halen als het ok is, panic als het niet ok is
        let num =num_str.parse::<usize>().unwrap(); // unwrap dient voor errors
    
        // return
        Ok(num)
    
        //return Ok(num); andere mogelijke return
    }
    
    fn consume_whitespaces (cursor: &mut Cursor<Vec<u8>>)-> Result<(),std::io::Error>{ //Result<() : de lege haakjes betekend  niks returnen
        let mut buff: [u8;1] = [0];
    
        loop{
            cursor.read(& mut buff)?;
            match buff[0]{
                b' ' | b'\n' | b'\r' | b'\t' => println!("Whitespace"),
                _ => { // je zit eigenlijk al te ver nu !!! zet cursor 1 terug
                    cursor.seek(SeekFrom::Current(-1))?;
                    break;
                }
            }
        }
        Ok(()) // () : de lege haakjes betekend  niks returnen
    
    }
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
pub fn main() {
    let args : Vec<String> = std::env::args().collect();
    let interrupt_received = Arc::new(AtomicBool::new(false));

    // sanity checks
    if nix::unistd::Uid::current().is_root() == false {
        eprintln!("Must run as root to be able to access /dev/mem\nPrepend \'sudo\' to the command");
        std::process::exit(1);
    } else if args.len() < 2 {
        eprintln!("Syntax: {:?} [image]", args[0]);
        // std::process::exit(1);
    }

/*     // TODO00000000: Read the PPM file here. You can find its name in args[1]
    let path = Path::new(&args[1]);
    let display = path.display();

    let mut file = match File::open(&path) {
        Err(why) => panic!("Could not open file: {} (Reason: {})", 
            display, why),
        Ok(file) => file
    };

    // read the full file into memory. panic on failure
    let mut raw_file = Vec::new();
    file.read_to_end(&mut raw_file).unwrap();

    // construct a cursor so we can seek in the raw buffer
    let mut cursor = Cursor::new(raw_file);
    let image = match Image::decode_ppm_image(&mut cursor) {
        Ok(img) => img,
        Err(why) => panic!("Could not parse PPM file - Desc: {}", why),
    }; */

    //Image::show_image(&image);

    
    // TODO00000000: Initialize the GPIO struct and the Timer struct
    let mut io = GPIO::new(1);
    let timer = Timer::new(); //DIT OOK JUIST??
    let mut frame = Frame::new();

    // This code sets up a CTRL-C handler that writes "true" to the 
    // interrupt_received bool.
    let int_recv = interrupt_received.clone();
    ctrlc::set_handler(move || {
        int_recv.store(true, Ordering::SeqCst);
    }).unwrap();

    while interrupt_received.load(Ordering::SeqCst) == false {
        //DONE????
        // TODO: Implement your rendering loop here
		let mut color_clk_mask : gpio_bits_t = 0;
        
        color_clk_mask |= GPIO_BIT!(PIN_R1) | GPIO_BIT!(PIN_G1) | GPIO_BIT!(PIN_B1) | GPIO_BIT!(PIN_R2) | GPIO_BIT!(PIN_G2) | GPIO_BIT!(PIN_B2) | GPIO_BIT!(PIN_CLK);
        
        for row_loop in 0..ROWS/2 {
            for b in 0..COLOR_DEPTH{
                for col in 0..32{

                    let top = &frame.pixels[row_loop][col];
                    let bot = &frame.pixels[ROWS /2 + row_loop][col];
                    
                    let plane_bits : u32 = GPIO::get_plane_bits(&mut io, &top, &bot, b as i8);
                    

                    GPIO::write_masked_bits(&mut io, plane_bits, color_clk_mask);
                    GPIO::set_bits(&mut io, GPIO_BIT!(PIN_CLK));

                }
                
                let row_bits : u32 = GPIO::get_row_bits(&mut io, row_loop as u8);

                GPIO::clear_bits(&mut io, color_clk_mask);
                io.write_masked_bits(row_bits, io.row_mask);
                GPIO::set_bits(&mut io, GPIO_BIT!(PIN_LAT));
                GPIO::clear_bits(&mut io, GPIO_BIT!(PIN_LAT));

                GPIO::clear_bits(&mut io, GPIO_BIT!(PIN_OE));
                
                timer.nanosleep(io.bitplane_timings[b] as u32);
                //nanosleep(io.bitplane_timings[b],&timer);
                GPIO::set_bits(&mut io, GPIO_BIT!(PIN_OE));
            }
        }
    }
    println!("Exiting.");
    if interrupt_received.load(Ordering::SeqCst) == true {
        println!("Received CTRL-C");
    } else {
        println!("Timeout reached");
    }

    // TODO: You may want to reset the board here (i.e., disable all LEDs)
    //GPIO::clear_bits(&mut io, GPIO_BIT!(PIN_OE));
    GPIO::set_bits(&mut io, GPIO_BIT!(PIN_OE));
}