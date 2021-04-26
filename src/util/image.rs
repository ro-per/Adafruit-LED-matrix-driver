// ==================================== PROJECT IMPORTS =======================================
use super::pixel::Pixel;
// ==================================== EXTERN IMPORTS =======================================
use byteorder::ReadBytesExt;
use std::io::{Read, Cursor,Seek,SeekFrom};

// ===========================================================================
// REPRESENTATION OF THE 'RAW' IMAGE
// ===========================================================================
pub struct Image {
    // TODO: SHOULD NOT BE PRIVATE
    width: usize,   // 32
    height: usize, //16
    pub pixels: Vec<Vec<Pixel>>
}

impl Image {
    // ==================================== CONSTRUCTOR =======================================

    // ==================================== PUBLIC FUNCTIONS =======================================
    pub fn decode_ppm_image(cursor: &mut Cursor<Vec<u8>>) -> Result<Image, std::io::Error> {
        let parent_method = "Image/decode_ppm_image:";
        println!("{} Decoding ppm image ...",parent_method);
        let mut image = Image { 
            width: 0,
            height: 0,
            pixels: vec![]
        };
    
        /* INLEZEN VAN HET TYPE */
        let mut header: [u8;2]=[0;2]; // inlezen van karakters
        cursor.read(&mut header)?; // ? geeft error terug mee met result van de functie
        match &header{ // & dient voor slice van te maken
            b"P6" => println!("\t P6 image"),  // b zorgt ervoor dat je byte string hebt (u8 slice)
            _ => panic!("\t Not an P6 image")  //_ staat voor default branch
        }
    
        /* INLEZEN VAN BREEDTE EN HOOGTE */
        image.width=Image::read_number(cursor)?;
        image.height=Image::read_number(cursor)?;
        let _colourRange = Image::read_number(cursor)?;
    
        /* eventuele whitespaces na eerste lijn */
        Image::consume_whitespaces(cursor)?;
    
        /* body inlezen */
    
        for _ in 0.. image.height{
            let mut row = Vec::new();
            for _ in 0..image.width{
                let red = cursor.read_u8()?;
                let green = cursor.read_u8()?;
                let blue = cursor.read_u8()?;

                let pixel = Pixel {
                    r:red,
                    g:green,
                    b:blue,
                };

                //row.push(Pixel::average_channels(pixel));
                row.push(pixel);

            }
            image.pixels.push(row);
        }
    
        println!("{} Decoding done !",parent_method);
    
        Ok(image)
    }
    // ==================================== PRIVATE FUNCTIONS =======================================
    fn read_number(cursor: &mut Cursor<Vec<u8>>)-> Result<usize,std::io::Error>{
        let parent_method = "Image/read_number:";
        Image::consume_whitespaces(cursor)?;

        let mut buff: [u8;1] = [0];
        let mut v = Vec::new(); // vector waar je bytes gaat in steken
    
        loop{
            cursor.read(& mut buff)?;
            match buff[0]{
                b'0'..= b'9' => v.push(buff[0]),
                b' ' | b'\n' | b'\r' | b'\t' => break,
                _ => panic!("{} Not a valid image",parent_method)
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
                b' ' | b'\n' | b'\r' | b'\t' => println!("\t consumed 1 whitespace"),
                _ => { // je zit eigenlijk al te ver nu !!! zet cursor 1 terug
                    cursor.seek(SeekFrom::Current(-1))?;
                    break;
                }
            }
        }
        Ok(()) // () : de lege haakjes betekend  niks returnen
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