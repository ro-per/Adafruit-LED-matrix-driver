// ==================================== PROJECT IMPORTS =======================================
use super::pixel::Pixel;
use crate::{*};
// ==================================== EXTERN IMPORTS =======================================
use mmap::{MemoryMap, MapOption};

// ============================================================================
// GPIO configuration parameters for the raspberry pi 3

/* NOTE/WARNING 1 : In many cases, particularly those where you need to set or clear multiple bits at once, 
it is convenient to store multiple pin numbers in one bit mask value. 
If you want to simultaneously set PIN_A and PIN_C to high, for example, 
you should probably create a bit mask with the positions of PIN_A and PIN_C set to 1, 
and all other positions set to 0. You can do this using the GPIO_BIT! macro.
In this example, you would do something like:
let pin_mask = GPIO_BIT!(PIN_A) | GPIO_BIT!(PIN_C);
io.set_bits(pin_mask);*/

/* NOTE/WARNING 2 : This method configures one pin at a time. 
The @pin_num argument that is expected here is really a pin number and not a bitmask!
Would be WRONG: io.configure_output_pin(VALID_BITS);
Would be OK: if GPIO_BIT!(PIN_A) & VALID_BITS {io.configure_output_pin(PIN_A);}*/
// ============================================================================

// MACRO FOR CREATING BITMASKS
type gpio_bits_t = u32;
macro_rules! GPIO_BIT {
    ($bit:expr) => {
        1 << $bit
    };
}

const VALID_BITS: u64 = GPIO_BIT!(PIN_OE) | GPIO_BIT!(PIN_CLK) | GPIO_BIT!(PIN_LAT) |
    GPIO_BIT!(PIN_A)  | GPIO_BIT!(PIN_B)  | GPIO_BIT!(PIN_C)   | GPIO_BIT!(PIN_D)   | GPIO_BIT!(PIN_E) |
    GPIO_BIT!(PIN_R1) | GPIO_BIT!(PIN_G1) | GPIO_BIT!(PIN_B1)  |
    GPIO_BIT!(PIN_R2) | GPIO_BIT!(PIN_G2) | GPIO_BIT!(PIN_B2);


pub struct GPIO {
    gpio_map_: Option<MemoryMap>,
    output_bits_: u32,
    input_bits_: u32,
    slowdown_: u32,                         // Please refer to the GPIO_SetBits and GPIO_ClearBits functions in the reference implementation to see how this is used.
    gpio_port_: *mut u32,                   // A raw pointer that points to the base of the GPIO register file
    gpio_set_bits_: *mut u32,               // A raw pointer that points to the pin output register (see section 2.1 in the assignment)
    gpio_clr_bits_: *mut u32,               // A raw pointer that points to the pin output clear register (see section 2.1)
    gpio_read_bits_: *mut u32,              // A raw pointer that points to the pin level register (see section 2.1)
    pub row_mask: u32,
    bitplane_timings: [u32; COLOR_DEPTH]
}

impl GPIO {
    

    // ==================================== CONSTRUCTOR =======================================
    pub fn new(slowdown: u32) -> GPIO {

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
    // ==================================== PUBLIC FUNCTIONS =======================================
    pub fn set_bits(self: &mut GPIO, value: u32) {
        unsafe {
            std::ptr::write_volatile(self.gpio_set_bits_, value);
            for _iter in 0..self.slowdown_ {
                std::ptr::write_volatile(self.gpio_set_bits_, value);
            }
        }
    }

    pub fn clear_bits(self: &mut GPIO, value: u32) {
        unsafe {
            std::ptr::write_volatile(self.gpio_clr_bits_, value);
            for _iter in 0..self.slowdown_ {
                std::ptr::write_volatile(self.gpio_clr_bits_, value);
            }
        }
    }

    // Write all the bits of @value that also appear in @mask. Leave the rest untouched.
    // @value and @mask are bitmasks
    pub fn write_masked_bits(self: &mut GPIO,value: u32,mask: u32) {
        self.clear_bits(!value & mask);
        self.set_bits(value & mask);
    }



    // Calculates the pins we must activate to push the address of the specified double_row
    pub fn get_row_bits(self: &GPIO, double_row: u8) -> u32 {
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
    pub fn get_plane_bits(self: &GPIO, top : &Pixel, bot : &Pixel, plane : i8) -> u32 {
       
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
    // ==================================== PRIVATE FUNCTIONS =======================================
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

   
}