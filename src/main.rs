// ==================================== PROJECT IMPORTS =======================================
pub mod util;
use util::pixel::Pixel;
use util::image::Image;
use util::frame::Frame;
// ==================================== CRATES =======================================
extern crate libc;
extern crate time;
extern crate ctrlc;
#[macro_use] extern crate simple_error;
extern crate shuteye;
extern crate mmap;
extern crate nix;
extern crate rand;
// ==================================== USE =======================================

use std::io::{Error, ErrorKind,Read, Cursor,Seek,SeekFrom};
use std::path::Path;
use std::fs::File;
use byteorder::ReadBytesExt;
//use sdl2::pixels::Color;
//use sdl2::rect::Rect;
use shuteye::sleep;
use std::time::{Duration};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::{fs::OpenOptions, os::unix::fs::OpenOptionsExt};
use std::os::unix::io::AsRawFd;
use std::io::prelude::*;
use mmap::{MemoryMap, MapOption};
// ==================================== CONST =======================================
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

// Use this bitmask for sanity checks // Convenience macro for creating bitmasks. See comment above "impl GPIO" below
macro_rules! GPIO_BIT {
    ($bit:expr) => {
        1 << $bit
    };
}
const VALID_BITS: u64 = GPIO_BIT!(PIN_OE) | GPIO_BIT!(PIN_CLK) | GPIO_BIT!(PIN_LAT) |
    GPIO_BIT!(PIN_A)  | GPIO_BIT!(PIN_B)  | GPIO_BIT!(PIN_C)   | GPIO_BIT!(PIN_D)   | GPIO_BIT!(PIN_E) |
    GPIO_BIT!(PIN_R1) | GPIO_BIT!(PIN_G1) | GPIO_BIT!(PIN_B1)  |
    GPIO_BIT!(PIN_R2) | GPIO_BIT!(PIN_G2) | GPIO_BIT!(PIN_B2);

    // own consts
const SUB_PANELS_: usize = 2;
const TIMER_OVERFLOW: u32 =4294967295;
const COLUMNS: usize = 32;
const ROWS: usize = 16;



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




// Use this struct to implement high-precision nanosleeps
struct Timer {
    _timemap: Option<MemoryMap>,
    timereg: *mut u32 // a raw pointer to the 1Mhz timer register (see section 2.5 in the assignment)
}




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
    fn write_masked_bits(self: &mut GPIO,value: u32,mask: u32) {
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
            // ALLES HIERONDER ZAL NOG GEINITIALISEERD MOETEN WORDEN
            gpio_set_bits_: 0 as *mut u32,
            gpio_clr_bits_: 0 as *mut u32,
            gpio_read_bits_: 0 as *mut u32,
            row_mask: 0,
            // DIT ZAL JE NODIG HEBBEN IN HET PROJECT, MAAR NIET IN LABO 4
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
        //DONE???
        // TODO: Implement this yourself.
        let tijd:u32 = std::ptr::read_volatile(self.timereg);
        tijd
    }

    fn new() -> Timer {
        //DONE???
        // TODO: Implement this yourself.

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
    fn nanosleep( self: &Timer,  mut nanos: u32) {
        //DONE???
        // TODO: Implement this yourself.

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






// ============================================================================
// MAIN FUNCTION
// ============================================================================


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


// ============================================================================
// PPM PARSER (paht in args[1])
// ============================================================================

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
    };

    //Image::show_image(&image); // requires sdl2 import (but takes long to build)

// ============================================================================
// TODO00000000: Initialize the GPIO struct and the Timer struct
// ============================================================================
    let mut io = GPIO::new(1);
    let _timer = Timer::new();
    let _frame = Frame::new();

    // This code sets up a CTRL-C handler that writes "true" to the 
    // interrupt_received bool.
    let int_recv = interrupt_received.clone();
    ctrlc::set_handler(move || {
        int_recv.store(true, Ordering::SeqCst);
    }).unwrap();


// ============================================================================
// RENDERING PIXELS ON THE MATRIX
// ============================================================================
    let parent_method = "main:";
    println!("{} Showing on matrix ...",parent_method);

    while interrupt_received.load(Ordering::SeqCst) == false {

        let mut color_clk_mask : gpio_bits_t = 0;
        color_clk_mask |= GPIO_BIT!(PIN_R1) | GPIO_BIT!(PIN_G1) | GPIO_BIT!(PIN_B1) | GPIO_BIT!(PIN_R2) | GPIO_BIT!(PIN_G2) | GPIO_BIT!(PIN_B2) | GPIO_BIT!(PIN_CLK);
        let mut row_mask : gpio_bits_t = 0;
        row_mask |= GPIO_BIT!(PIN_A) | GPIO_BIT!(PIN_B) | GPIO_BIT!(PIN_C) | GPIO_BIT!(PIN_CLK);


        /* STEP 1. LOOP EACH (DOUBLE) ROW */
        for row in 0..ROWS/2 {
            /* STEP 2. LOOP COLOR DEPTH */                      /*  8 Colors that can be mixed with each other */
            for cd in 0..COLOR_DEPTH{                
                /* STEP 3. LOOP EACH COLUMN */
                for col in 0.. COLUMNS{
                    
                    let pixel_top = &image.pixels[row][col];
                    let pixel_bot = &image.pixels[ROWS /2 + row][col];
                    
                    // let pixel_top = Pixel{r: (255 as u8), g:(0 as u8), b: (0 as u8)
                    // };
                    // let pixel_bot = Pixel{r: (0 as u8), g:(0 as u8), b: (255 as u8)
                    // };

                    let plane_bits : u32 = GPIO::get_plane_bits(&mut io, &pixel_top, &pixel_bot, cd as i8);
    

                    /* STEP 4. PUSH COLORS */
                    GPIO::write_masked_bits(&mut io, plane_bits, color_clk_mask);
                    /* STEP 5. SIGNAL MATRIX THAT DATA FOR A SINGLE COLUMN HAS ARRIVED */
                    GPIO::set_bits(&mut io, GPIO_BIT!(PIN_CLK)); // Rising edge: clock color in.
                }
            GPIO::clear_bits(&mut io, color_clk_mask); // clock back to normal.
            
            /* STEP 6. SIGNAL MATRIX THAT DATA FOR A DOUBLE ROW HAS ARRIVED */
            let row_bits : u32 = GPIO::get_row_bits(&mut io, row as u8);
            io.write_masked_bits(row_bits,io.row_mask);
            GPIO::set_bits(&mut io, GPIO_BIT!(PIN_LAT)); //disable
            GPIO::clear_bits(&mut io, GPIO_BIT!(PIN_LAT)); //enable


            /* STEP 7. ENABLE OUTPUT PINS */
            GPIO::clear_bits(&mut io, GPIO_BIT!(PIN_OE));
            }
        }
    }
    if interrupt_received.load(Ordering::SeqCst) == true {
        println!("\n{} Received CTRL-C",parent_method);
    } else {
        println!("{} Timeout reached",parent_method);
    }
    println!("Exiting...");


    // TODO: You may want to reset the board here (i.e., disable all LEDs)
   

    /* STEP 8. TIMEOUT */
    // timer.nanosleep(...);

    /* STEP 9. DISABLE OUTPUT PINS */
    // GPIO::set_bits(&mut io, GPIO_BIT!(PIN_OE));
    // timer.nanosleep(...);
}