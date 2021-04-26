// ==================================== PROJECT IMPORTS =======================================
use crate::{TIMER_REGISTER_OFFSET,TIMER_OVERFLOW};
use crate::mmap_bcm_register;
// ==================================== EXTERN IMPORTS =======================================
use mmap::{MemoryMap, MapOption};
use shuteye::sleep;
use std::time::{Duration};

// ===========================================================================
// IMPLEMENTATION OF HIGH-PRECISION NANOSLEEPS (see section 2.5 in the assignment)
/* NOTE/WARNING: Since the raspberry pi's timer frequency is only 1Mhz, you cannot reach full nanosecond precision here. 
    You will have to think about how you can approximate the desired precision. 
    Obviously, there is no perfect solution here. */
// ===========================================================================
pub struct Timer {
    _timemap: Option<MemoryMap>,
    timereg: *mut u32 // a raw pointer to the 1Mhz timer register (see section 2.5 in the assignment)
}

impl Timer {
    // ==================================== UNSAFE =======================================

    // Reads from the 1Mhz timer register (see Section 2.5 in the assignment)
    unsafe fn read(self: &Timer) -> u32 {
        let time:u32 = std::ptr::read_volatile(self.timereg);
        time
    }
    // ==================================== CONSTRUCTOR =======================================
    pub fn new() -> Timer {
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


    // ==================================== PUBLIC FUNCTIONS =======================================

    // ==================================== PRIVATE FUNCTIONS =======================================    
    fn nanosleep( self: &Timer,  mut nanos: u32) {
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