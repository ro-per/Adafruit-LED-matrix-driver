# Adafruit LED matrix driver in rust
```Image``` -> ```Vec<Frame> ``` --> ```RGB Matrix```
# Table of contents
1. [Tasks](I)  
1.A [Funtionality](#IA)  
1.B [Structs](#IB)  
1.C [Effects](#IC)
2. [Authors](#II)
3. [Links](#III)

![Adafruit Medium 16x32 RGB LED matrix panel](https://cdn.shopify.com/s/files/1/1749/9663/products/420-07_800x.jpg?v=1500650577 "Adafruit Medium 16x32 RGB LED matrix panel")

## 1. Tasks <a name="I"></a>
### I.A Funtionality                        <a name="IA"></a>
* *CTRL+C* Interrupt handling

### 1.B Structs                             <a name="IB"></a>
#### 1.B.I **Image:** 
*This is a representation of the "raw" image*  
atrr= {width | height | pixels: Vec<Vec<Pixel>>}
- [ ]  ```fn rescaleTo_32_16();```
- [ ]  ```fn rescaleTo_xx_16();```
- [x]  ```fn ppm_image_parser();``` (lab 2)
    
#### 1.B.II **Frame:**  
*This is a representation of the frame we're currently rendering*  
?????
    
#### 1.B.III **Timer:**
- [ ] ```fn nanosleep(self: &Timer, mut nanos: u32)```

### 1.C Effects                             <a name="IC"></a>
- [ ] --S(croll)M(ode)=
    * A(uto)   : Scrolling automatically
    * M(ouse)  :Scroll with mouse
- [ ] --S(croll)D(ir)=
    * L(eft)
    * R(ight)
- [ ] --T(ext)=<filename>.txt (+ News API ?)
- [ ] Separate bottom/ upper
- [x] ```--Image=<filename>.ppm``` (lab 3)
- [x] ```--colors=grey```
- [x] ```--colors=invert```
- [x] ```--mirror=vertical```
- [x] ```--mirror=horizontal```

## 2. Authors                              <a name="II"></a>
- [Nick Braeckman](https://github.com/NickBraeckman)
- [Cedric Lefevre](https://github.com/Cedric-Lefevre)
- [Romeo Permentier](https://github.com/ro-per)


## 3. Links                               <a name="III"></a>
- [Rust bindings for C++ Library](https://github.com/rust-rpi-led-matrix/rust-rpi-rgb-led-matrix) (see below)
- [C++ Library](https://github.com/hzeller/rpi-rgb-led-matrix) for driving RGB-Matrix
- [Backup project](https://github.com/ro-per/VS-LED_Matrix_Driver_Backup/blob/master/src/main.rs) of last year
- [Lab 1: Debugging C and CPP with GDB](https://github.com/ro-per/VS-Lab1_Debugging_C_CPP_with_GDB)
- [Lab 2: Memory Exploits in C](https://github.com/ro-per/VS-Lab2_Memory_Exploits_in_C)
- [Lab 3: IO and parsing in Rust](https://github.com/ro-per/VS-Lab3_IO_and_Parsing_in_Rust/blob/main/src/main.rs)
- [Lab 4: LED matrix driver](https://github.com/ro-per/VS-Lab4-LED_Matrix_Driver_in_Rust)
