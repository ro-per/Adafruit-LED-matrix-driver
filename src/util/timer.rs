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
    _timemap: Option<MemoryMap>,
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
            _timemap: None,
            timereg: 0 as *mut u32,
        };

        match &map {
            &Some(ref map) => unsafe {
                timer.timereg = map.data() as *mut u32;
                timer.timereg.offset(1);
            },
            &None => {}
        };
        timer
    }

    // ==================================== PUBLIC FUNCTIONS =======================================

    // ==================================== PRIVATE FUNCTIONS =======================================
    fn nanosleep(self: &Timer, mut nanos: u32) {
        let k_jitter_allowance = 60 * 1000;

        if nanos > (k_jitter_allowance + 5000) {
            let before: u32 = unsafe { self.read() };
            let sleep_time = Duration::new(0, nanos - k_jitter_allowance);
            sleep(sleep_time);
            let after: u32 = unsafe { self.read() };
            let time_passed = 1000 * (after - before);
            if time_passed > nanos {
                return;
            } else {
                nanos -= time_passed;
            }
        }

        if nanos < 20 {
            return;
        }
    }
}
