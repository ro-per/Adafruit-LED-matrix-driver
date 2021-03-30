# Adafruit LED matrix driver in rust
```Image``` -> ```Vec<Frame> ``` --> ```RGB Matrix```
# Table of contents
1. [Tasks](I)  
I.A [Funtionality](#IA)  
I.B [Structs](#IB)  
I.C [Effects](#IC)
2. [Authors](#II)
3. [Links](#III)

![Adafruit Medium 16x32 RGB LED matrix panel](https://cdn.shopify.com/s/files/1/1749/9663/products/420-07_800x.jpg?v=1500650577 "Adafruit Medium 16x32 RGB LED matrix panel")

## I. Tasks <a name="I"></a>
### I.A Funtionality                        <a name="IA"></a>
* *CTRL+C* Interrupt handling

### I.B Structs                             <a name="IB"></a>
#### I.B.1 **Image:** 
*This is a representation of the "raw" image*  
atrr= {width | height | pixels: Vec<Vec<Pixel>>}
- [ ]  ```fn rescaleTo_32_16();```
- [ ]  ```fn rescaleTo_xx_16();```
- [ ] R  ```fn ppm_image_parser();``` (lab 2)
- [ ]  ```fn mirror();```
    
#### I.B.2 **Frame:**  
*This is a representation of the frame we're currently rendering*  
?????
    
#### I.B.3 **Timer:**
- [ ] ```fn nanosleep(self: &Timer, mut nanos: u32)```

#### I.B.4 **Pixel:** 
*Holds single RGB value*
    * atrr= {r: u16 | g: u16 | b: u16}
- [ ]R  ```fn to_grey_scale();```
- [ ]R  ```fn color_invert();```
   
#### I.B.5 **GPIO**
- [ ]R  ```fn clockPulse(); ```
- [ ]R  ```fn latchPulse(); ```
- [ ]R  ``` fn oeEnabled(boolean b);```


### I.C Effects                             <a name="IC"></a>
- [ ] --Fade: Show static Frame that fades in and out (use PulsWidthModulation)
- [ ]  --ScrollMode=
    * A(uto)   : Scrolling automatically
    * M(ouse)  :Scroll with mouse
- [ ]  --ScrollDir=
    * L(eft)
    * R(ight)
- [ ]R  --Image=<filename>.ppm (lab 3)

## II. Authors                              <a name="II"></a>
- @NickBraeckman
- @Cedric-Lefevre
- @ro-per


## III. Links                               <a name="III"></a>
- [Rust bindings for C++ Library](https://github.com/rust-rpi-led-matrix/rust-rpi-rgb-led-matrix) (see below)
- [C++ Library](https://github.com/hzeller/rpi-rgb-led-matrix) for driving RGB-Matrix
- [Backup project](https://github.com/ro-per/VS-LED_Matrix_Driver_Backup/blob/master/src/main.rs) of last year
- [Lab 1: Debugging C and CPP with GDB](https://github.com/ro-per/VS-Lab1_Debugging_C_CPP_with_GDB)
- [Lab 2: Memory Exploits in C](https://github.com/ro-per/VS-Lab2_Memory_Exploits_in_C)
- [Lab 3: IO and parsing in Rust](https://github.com/ro-per/VS-Lab3_IO_and_Parsing_in_Rust/blob/main/src/main.rs)
- [Lab 4: LED matrix driver](https://github.com/ro-per/VS-Lab4-LED_Matrix_Driver_in_Rust)


<style type="text/css">
    ol { list-style-type: upper-roman; }
</style>
