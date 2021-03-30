# Adafruit LED matrix driver in rust
## Tasks
### Funtionality
.PNG -> BPM -> Rescale to ??*16 
* Frame to Display
    * Functions (Clock in, Latch in, RGB1 RGB2, ...)
    * Uses Timer
* * 
* *CTRL+C* Interrupt handling

### Structs
* **Image:** *This is a representation of the "raw" image*
    * atrr= {width | height | pixels: Vec<Vec<Pixel>>}
    - [ ]  ```fn rescaleTo_32_16();```
   - [ ]  ```fn rescaleTo_xx_16();```
   - [ ]  ```fn ppm_image_parser();``` (lab 2)
    
    
* **Frame:**  *This is a representation of the frame we're currently rendering*
class that holds 3 Matrices (or a 3D Matrix)  
    * (x,y) = (32,16)   : correspond to LED's on panel
    * (z)   = (3)       :contains Pixels
    
    
* **Timer:** used to timeout funtions


* **Pixel:** *Holds single RGB value*
    * atrr= {r: u16 | g: u16 | b: u16}
    
    
* **GPIO**
- [ ]R  ```fn clockPulse(); ```
- [ ]R  ```fn latchPulse(); ```
- [ ]R  ``` fn oeEnabled(boolean b);```


### Effects
- [ ] --Fade: Show static Frame that fades in and out (use PulsWidthModulation)
- [ ]  --ScrollMode=
    * A(uto)   : Scrolling automatically
    * M(ouse)  :Scroll with mouse
- [ ]  --ScrollDir=
    * L(eft)
    * R(ight)
- [ ]R  --Image=<filename>.ppm (lab 3)

## Links
- [Rust bindings for C++ Library](https://github.com/rust-rpi-led-matrix/rust-rpi-rgb-led-matrix)
- [C++ Library](https://github.com/hzeller/rpi-rgb-led-matrix)
- [Backup project](https://github.com/ro-per/VS-LED_Matrix_Driver_Backup/blob/master/src/main.rs)
- [Lab 1: Debugging C and CPP with GDB](https://github.com/ro-per/VS-Lab1_Debugging_C_CPP_with_GDB)
- [Lab 2: Memory Exploits in C](https://github.com/ro-per/VS-Lab2_Memory_Exploits_in_C)
- [Lab 3: IO and parsing in Rust](https://github.com/ro-per/VS-Lab3_IO_and_Parsing_in_Rust/blob/main/src/main.rs)
- [Lab 4: LED matrix driver](https://github.com/ro-per/VS-Lab4-LED_Matrix_Driver_in_Rust)
