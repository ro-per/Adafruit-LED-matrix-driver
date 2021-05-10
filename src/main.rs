// ==================================== PROJECT IMPORTS =======================================
pub mod util;
// import structs
use util::charset::Charset;
use util::frame::Frame;
use util::gpio::GPIO;
use util::image::Image;
use util::timer::Timer;
// import function
use util::mmap_bcm_register::*;
// ==================================== CRATES =======================================
extern crate ctrlc;
extern crate libc;
extern crate time;
#[macro_use]
extern crate simple_error;
extern crate mmap;
extern crate nix;
extern crate rand;
extern crate shuteye;

// ==================================== USE =======================================
use std::fs::File;
use std::io::prelude::*;
use std::io::{Cursor, Error, ErrorKind, Read, Seek, SeekFrom};
use std::path::Path;
use std::time::{Duration, SystemTime};
use time::Timespec;
//use sdl2::pixels::Color;
//use sdl2::rect::Rect;

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

// ==================================== CONST =======================================
const BCM2709_PERI_BASE: u64 = 0x3F000000;
const GPIO_REGISTER_OFFSET: u64 = 0x200000;
const TIMER_REGISTER_OFFSET: u64 = 0x3000;
const REGISTER_BLOCK_SIZE: u64 = 4096;
const COLOR_DEPTH: usize = 8;

const PIN_OE: u64 = 4;
const PIN_CLK: u64 = 17;
const PIN_LAT: u64 = 21;
const PIN_A: u64 = 22;
const PIN_B: u64 = 26;
const PIN_C: u64 = 27;
const PIN_D: u64 = 20;
const PIN_E: u64 = 24;
const PIN_R1: u64 = 5;
const PIN_G1: u64 = 13;
const PIN_B1: u64 = 6;
const PIN_R2: u64 = 12;
const PIN_G2: u64 = 16;
const PIN_B2: u64 = 23;

const SUB_PANELS_: usize = 2;
const TIMER_OVERFLOW: u32 = 4294967295;
const COLUMNS: usize = 32;
const ROWS: usize = 16;

const NUMBER_SPACES: usize = 1;

// MACRO FOR CREATING BITMASKS
type gpio_bits_t = u32;
macro_rules! GPIO_BIT {
    ($bit:expr) => {
        1 << $bit
    };
}

// ==================================== MAIN =======================================
pub fn main() {
    let args: Vec<String> = std::env::args().collect();
    let interrupt_received = Arc::new(AtomicBool::new(false));

    // ------------------------------------ SANITY CHECKS ------------------------------------
    if nix::unistd::Uid::current().is_root() == false {
        eprintln!(
            "Must run as root to be able to access /dev/mem\nPrepend \'sudo\' to the command"
        );
        std::process::exit(1);
    } else if args.len() < 2 {
        eprintln!("Syntax: {:?} [image]", args[0]);
        // std::process::exit(1);
    }

    // ------------------------------------ PPM PARSER (paht in args[1]) ------------------------------------
    let mut image = Image::read_ppm_image(&args[1], true);
    //image.print_to_console();
    //Image::show_image(&image); // requires sdl2 import (but takes long to build)

    // ------------------------------------ INIT CHARSET ------------------------------------
    let character_set_regular = Charset::new();

    // ------------------------------------ GET TEXT ------------------------------------
    let mut file = File::open(&args[2]).expect("Unable to open the file");
    let mut contents = String::new();
    file.read_to_string(&mut contents)
        .expect("Unable to read the file");

    let text = String::from(contents); //FIXME Romeo read from file
    let mut text_image = character_set_regular.get_text(text, true); // True means random RGB value per letter
    text_image.print_to_console();

    // ------------------------------------ CHECK FOR FEATURES ------------------------------------
    for arg in args.iter() {
        match arg.as_str() {
            "--colors=grey" => image.to_grey_scale(),
            "--colors=invert" => image.invert_colors(),
            "--mirror=vertical" => image.mirror_vertical(),
            "--mirror=horizontal" => image.mirror_horizontal(),
            "--g" => image.gamma_correction(),
            _ => (),
        }
    }

    // ------------------------------------ INIT GPIO ------------------------------------
    let mut io = GPIO::new(1); //Slowdown = 2 mag je mee spelen

    // ------------------------------------ INIT TIMER ------------------------------------
    let mut timer = Timer::new();

    // ------------------------------------ INIT FRAME ------------------------------------
    let mut frame = Frame::new();
    //Image inladen in het frame
    frame.next_image_frame(&image); //FIXME CEDRIC (mag wss weg)
                                    //Clock starten
    let mut begin = time::get_time();
    let mut current_time: Timespec;

    // ------------------------------------ CTRL+C HANDLER ------------------------------------
    let int_recv = interrupt_received.clone();
    ctrlc::set_handler(move || {
        int_recv.store(true, Ordering::SeqCst);
    })
    .unwrap();

    // ------------------------------------ PIXEL RENDERING ------------------------------------
    let parent_method = "main:";
    println!("{} Showing on matrix ...", parent_method);

    while interrupt_received.load(Ordering::SeqCst) == false {
        let mut color_clk_mask: gpio_bits_t = 0;
        color_clk_mask |= GPIO_BIT!(PIN_R1)
            | GPIO_BIT!(PIN_G1)
            | GPIO_BIT!(PIN_B1)
            | GPIO_BIT!(PIN_R2)
            | GPIO_BIT!(PIN_G2)
            | GPIO_BIT!(PIN_B2)
            | GPIO_BIT!(PIN_CLK);

        let mut row_mask: gpio_bits_t = 0;
        row_mask |= GPIO_BIT!(PIN_A) | GPIO_BIT!(PIN_B) | GPIO_BIT!(PIN_C) | GPIO_BIT!(PIN_CLK);

        /* STEP 1. LOOP EACH (DOUBLE) ROW */
        for row in 0..ROWS / 2 {
            /* STEP 2. LOOP COLOR DEPTH */
            /*  8 Colors that can be mixed with each other */
            for cd in 0..COLOR_DEPTH {
                /* STEP 3. LOOP EACH COLUMN */
                for col in 0..COLUMNS {
                    let pixel_top = &image.pixels[row][col];
                    let pixel_bot = &image.pixels[ROWS / 2 + row][col];

                    let plane_bits: u32 =
                        GPIO::get_plane_bits(&mut io, &pixel_top, &pixel_bot, cd as i8);
                    /* STEP 4. PUSH COLORS */
                    GPIO::write_masked_bits(&mut io, plane_bits, color_clk_mask);

                    /* STEP 5. SIGNAL MATRIX THAT DATA FOR A SINGLE COLUMN HAS ARRIVED */
                    GPIO::set_bits(&mut io, GPIO_BIT!(PIN_CLK)); // Rising edge: clock color in.
                }
                GPIO::clear_bits(&mut io, color_clk_mask); // clock back to normal.

                /* STEP 6. SIGNAL MATRIX THAT DATA FOR A DOUBLE ROW HAS ARRIVED */
                let row_bits: u32 = GPIO::get_row_bits(&mut io, row as u8);
                io.write_masked_bits(row_bits, io.row_mask);
                GPIO::set_bits(&mut io, GPIO_BIT!(PIN_LAT)); //disable
                GPIO::clear_bits(&mut io, GPIO_BIT!(PIN_LAT)); //enable

                /* STEP 7. ENABLE OUTPUT PINS */
                GPIO::clear_bits(&mut io, GPIO_BIT!(PIN_OE));
            }
        }
    }

    // ------------------------------------ SCROLL FUNCTIONALITY ------------------------------------
    current_time = time::get_time();
    let diff = current_time - begin;

    // snelheid scrollen
    if diff >= time::Duration::milliseconds(10) {
        frame.next_image_frame(&image);
        begin = current_time;
    };

    // ------------------------------------ INTERRUPT HANDLER ------------------------------------
    if interrupt_received.load(Ordering::SeqCst) == true {
        println!("\n{} Received CTRL-C", parent_method);
    } else {
        println!("{} Timeout reached", parent_method);
    }
    println!("Exiting...");

    // ------------------------------------ TIMEOUT ------------------------------------
    // timer.nanosleep(...);

    // ------------------------------------ DISABLE OUTPUT PINS ------------------------------------
    // GPIO::set_bits(&mut io, GPIO_BIT!(PIN_OE));
    // timer.nanosleep(...);
}
