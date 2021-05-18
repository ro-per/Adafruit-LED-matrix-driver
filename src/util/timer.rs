// ==================================== PROJECT IMPORTS =======================================
use crate::mmap_bcm_register;
use crate::*;
// ==================================== EXTERN IMPORTS =======================================
use mmap::MemoryMap;
use shuteye::sleep;
use std::time::Duration;

// ===========================================================================
// IMPLEMENTATION OF HIGH-PRECISION NANOSLEEPS (see section 2.5 in the assignment)
/* NOTE/WARNING: Since the raspberry pi's timer frequency is only 1Mhz, you cannot reach full nanosecond precision here.
You will have to think about how you can approximate the desired precision.
Obviously, there is no perfect solution here. */
// ===========================================================================
pub struct Timer {
    _timermap: Option<MemoryMap>,
    timereg: *mut u32, // a raw pointer to the 1Mhz timer register (see section 2.5 in the assignment)
}

impl Timer {
    // ==================================== UNSAFE =======================================

    // Reads from the 1Mhz timer register (see Section 2.5 in the assignment)
    unsafe fn read(self: &Timer) -> u32 {
        let time: u32 = std::ptr::read_volatile(self.timereg);
        time
    }
    // ==================================== CONSTRUCTOR =======================================
    pub fn new() -> Timer {
        let map = mmap_bcm_register(TIMER_REGISTER_OFFSET as usize);

        let mut timer: Timer = Timer {
            _timermap: None,
            timereg: 0 as *mut u32,
        };

        match &map {
            Some(m) => unsafe {
                timer.timereg = (m.data() as *mut u32).offset(1);
            },
            None => {}
        };
        timer._timermap = map;
        timer
    }

    // ==================================== PUBLIC FUNCTIONS =======================================
    pub fn nanosleep(self: &Timer, mut nanos: u32) {
        // account for jitter on larger time periods
        let k_jitter_allowance = 60000;

        if nanos > k_jitter_allowance + 5000 {
            //println!("{}",nanos);
            let before: u32 = unsafe { self.read() };
            let sleep_time = Duration::new(0, nanos - k_jitter_allowance);
            sleep(sleep_time);
            let after: u32 = unsafe { self.read() };
            let time_passed: u64;
            // account for timer overflow
            if after > before {
                time_passed = 1000 * (after - before) as u64;
            } else {
                time_passed = 1000 * (TIMER_OVERFLOW - before + after) as u64;
            }

            if time_passed > nanos as u64 {
                return;
            } else {
                nanos -= time_passed as u32;
            }
        }

        if nanos < 20 {
            return;
        }

        // busy wait for shorter timing periods
        let start: u32 = unsafe { self.read() };
        let mut cur: u32 = start;
        let mut tresshold = start + nanos / 1000;

        //println!("start: {}", start);
        //println!("cur: {}", cur);
        //println!("nanos: {}", nanos);
        //println!("tresshold:{}", tresshold);
        while tresshold >= cur {
            cur = unsafe { self.read() };
            //println!("start: {}", start);
            //println!("cur: {}", cur);
            //println!("tresshold: {}", tresshold);
            if cur < start {
                //println!("start: {}", start);
                //println!("cur: {}", cur);
                //println!("tresshold: {}", tresshold);
                tresshold = TIMER_OVERFLOW - start + nanos / 1000;
                //println!("tresshold: {}", tresshold);
            }
        }
        return;
    }
    // ==================================== PRIVATE FUNCTIONS =======================================
}
