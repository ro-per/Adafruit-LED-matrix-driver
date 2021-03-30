// This is a highly compressed and simplified version of the
// scrolling-text-example.cc code in Henner Zeller's LED-matrix
// library. The original code was licensed under the following terms:

// Copyright (C) 2014 Henner Zeller <h.zeller@acm.org>
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation version 2.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <http://gnu.org/licenses/gpl-2.0.txt>


#include <assert.h>
#include <getopt.h>
#include <limits.h>
#include <math.h>
#include <signal.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>
#include <pthread.h>
#include <stdint.h>
#include <sys/mman.h>
#include <sys/time.h>
#include <fcntl.h>

#define ROWS         16
#define COLUMNS      32
#define SUB_PANELS_  2
#define COLOR_DEPTH  8

// GPIO configuration for the raspberry pi 3
#define BCM2709_PERI_BASE          0x3F000000
#define GPIO_REGISTER_OFFSET         0x200000
#define COUNTER_1Mhz_REGISTER_OFFSET   0x3000
#define REGISTER_BLOCK_SIZE           (4*1024)

// GPIO setup macros. Always use INP_GPIO(x) before using OUT_GPIO(x).
#define INP_GPIO(io, g) *(io->gpio_port_+((g)/10)) &= ~(7<<(((g)%10)*3))
#define OUT_GPIO(io, g) *(io->gpio_port_+((g)/10)) |=  (1<<(((g)%10)*3))
#define GPIO_BIT(b) (1<<(b))

// Bit mapping for the raspberry pi 3
#define PIN_OE  GPIO_BIT(4)
#define PIN_CLK GPIO_BIT(17)
#define PIN_LAT GPIO_BIT(21)
#define PIN_A   GPIO_BIT(22)
#define PIN_B   GPIO_BIT(26)
#define PIN_C   GPIO_BIT(27)
#define PIN_D   GPIO_BIT(20)
#define PIN_E   GPIO_BIT(24)
#define PIN_R1  GPIO_BIT(5)
#define PIN_G1  GPIO_BIT(13)
#define PIN_B1  GPIO_BIT(6)
#define PIN_R2  GPIO_BIT(12)
#define PIN_G2  GPIO_BIT(16)
#define PIN_B2  GPIO_BIT(23)

// Available bits that actually have pins.
const uint32_t kValidBits =
  ((1 <<  0) | (1 <<  1) | // RPi 1 - Revision 1 accessible
   (1 <<  2) | (1 <<  3) | // RPi 1 - Revision 2 accessible
   (1 <<  4) | (1 <<  7) | (1 << 8) | (1 <<  9) |
   (1 << 10) | (1 << 11) | (1 << 14) | (1 << 15)| (1 <<17) | (1 << 18) |
   (1 << 22) | (1 << 23) | (1 << 24) | (1 << 25)| (1 << 27) |
   // support for A+/B+ and RPi2 with additional GPIO pins.
   (1 <<  5) | (1 <<  6) | (1 << 12) | (1 << 13) | (1 << 16) |
   (1 << 19) | (1 << 20) | (1 << 21) | (1 << 26));

typedef uint32_t gpio_bits_t;
static gpio_bits_t row_mask = 0;
int bitplane_timings[COLOR_DEPTH];

struct Pixel {
  uint16_t R, G, B;
};

struct Pixel Frame[ROWS][COLUMNS];

// A raw pixel in a PPM file.
struct PPMPixel {
  uint8_t R, G, B;
};
struct PPMPixel* image = NULL;

int image_width = 0;
int current_position = 0;



struct GPIO {
  uint32_t output_bits_;
  uint32_t input_bits_;
  uint32_t reserved_bits_;
  int slowdown_;
  volatile uint32_t *gpio_port_;
  volatile uint32_t *gpio_set_bits_;
  volatile uint32_t *gpio_clr_bits_;
  volatile uint32_t *gpio_read_bits_;
};

uint32_t *mmap_bcm_register(off_t register_offset) {
  const off_t base = BCM2709_PERI_BASE;

  int mem_fd;
  if ((mem_fd = open("/dev/mem", O_RDWR|O_SYNC) ) < 0) {
    perror("can't open /dev/mem: ");
    return NULL;
  }

  uint32_t *result =
    (uint32_t*) mmap(NULL,                  // Any adddress in our space will do
		     REGISTER_BLOCK_SIZE,   // Map length
		     PROT_READ|PROT_WRITE,  // Enable r/w on GPIO registers.
		     MAP_SHARED,
		     mem_fd,                // File to map
		     base + register_offset // Offset to bcm register
		     );
  close(mem_fd);

  if (result == MAP_FAILED) {
    perror("mmap error: ");
    fprintf(stderr, "Pi3: MMapping from base 0x%lx, offset 0x%lx\n",
	    base, register_offset);
    return NULL;
  }
  return result;
}

// Initialize outputs.
// Returns the bits that were available and could be set for output.
uint32_t GPIO_InitOutputs(struct GPIO* io, uint32_t outputs) {
      
  if (!io || io->gpio_port_ == NULL) {
    fprintf(stderr, "Attempt to init outputs but not yet Init()-ialized.\n");
    return 0;
  }

  // Hack: for the PWM mod, the user soldered together GPIO 18 (new OE)
  // with GPIO 4 (old OE).
  // Since they are connected inside the HAT, want to make extra sure that,
  // whatever the outside system set as pinmux, the old OE is _not_ also
  // set as output so that these GPIO outputs don't fight each other.
  //
  // So explicitly set both of these pins as input initially, so the user
  // can switch between the two modes "adafruit-hat" and "adafruit-hat-pwm"
  // without trouble.
  INP_GPIO(io, 4);
  INP_GPIO(io, 18);
  // Even with PWM enabled, GPIO4 still can not be used, because it is
  // now connected to the GPIO18 and thus must stay an input.
  // So reserve this bit if it is not set in outputs.
  io->reserved_bits_ = (1<<4) & ~outputs;

  outputs &= kValidBits;     // Sanitize: only bits on GPIO header allowed.
  outputs &= ~(io->output_bits_ | io->input_bits_ | io->reserved_bits_);
  for (uint32_t b = 0; b <= 27; ++b) {
    if (outputs & (1 << b)) {
      INP_GPIO(io, b);   // for writing, we first need to set as input.
      OUT_GPIO(io, b);
    }
  }
  io->output_bits_ |= outputs;
  return outputs;
}

// Set the bits that are '1' in the output. Leave the rest untouched.
inline void GPIO_SetBits(struct GPIO* io, uint32_t value) {
  if (!value) return;
  *io->gpio_set_bits_ = value;
  for (int i = 0; i < io->slowdown_; ++i) {
    *io->gpio_set_bits_ = value;
  }
}

// Clear the bits that are '1' in the output. Leave the rest untouched.
inline void GPIO_ClearBits(struct GPIO* io, uint32_t value) {
  if (!value) return;
  *io->gpio_clr_bits_ = value;
  for (int i = 0; i < io->slowdown_; ++i) {
    *io->gpio_clr_bits_ = value;
  }
}

// Write all the bits of "value" mentioned in "mask". Leave the rest untouched.
inline void GPIO_WriteMaskedBits(struct GPIO* io, uint32_t value, uint32_t mask) {
  // Writing a word is two operations. The IO is actually pretty slow, so
  // this should probably  be unnoticable.
  GPIO_ClearBits(io, ~value & mask);
  GPIO_SetBits(io, value & mask);
}

inline void GPIO_Write(struct GPIO* io, uint32_t value) {
  GPIO_WriteMaskedBits(io, value, io->output_bits_);
}

inline uint32_t GPIO_Read(struct GPIO* io) {
  return *io->gpio_read_bits_ & io->input_bits_;
}

struct GPIO* GPIO_New(int slowdown, int rows) {

  struct GPIO* io = (struct GPIO*) malloc(sizeof(struct GPIO));
  io->output_bits_ = 0;
  io->input_bits_ = 0;
  io->reserved_bits_ = 0;
  io->slowdown_ = slowdown;
  io->gpio_port_ = mmap_bcm_register(GPIO_REGISTER_OFFSET);

  if (io->gpio_port_ == NULL) {
    free(io);
    return NULL;
  }

  io->gpio_set_bits_ = io->gpio_port_ + (0x1C / sizeof(uint32_t));
  io->gpio_clr_bits_ = io->gpio_port_ + (0x28 / sizeof(uint32_t));
  io->gpio_read_bits_ = io->gpio_port_ + (0x34 / sizeof(uint32_t));

  // Tell GPIO about all bits we intend to use.
  gpio_bits_t all_used_bits = 0;

  all_used_bits |= PIN_OE | PIN_CLK | PIN_LAT;
  all_used_bits |= PIN_R1 | PIN_G1 | PIN_B1 | PIN_R2 | PIN_G2 | PIN_B2;

  row_mask = PIN_A;
  if (rows / SUB_PANELS_ > 2) row_mask |= PIN_B;
  if (rows / SUB_PANELS_ > 4) row_mask |= PIN_C;
  if (rows / SUB_PANELS_ > 8) row_mask |= PIN_D;
  if (rows / SUB_PANELS_ > 16) row_mask |= PIN_E;
  all_used_bits |= row_mask;

  const uint32_t result = GPIO_InitOutputs(io, all_used_bits);
  assert(result == all_used_bits);

  uint32_t timing_ns = 200;
  for (int b = 0; b < COLOR_DEPTH; ++b) {
    bitplane_timings[b] = timing_ns;
    timing_ns *= 2;
  } 

  return io;
}

static volatile uint32_t *timer1Mhz = NULL;

uint32_t Timer_GetMicrosecondCounter() {
  return timer1Mhz ? *timer1Mhz : 0;
}

static unsigned char Timer_Init() {
  uint32_t *timereg = mmap_bcm_register(COUNTER_1Mhz_REGISTER_OFFSET);
  if (timereg == NULL) 
    return 0;      
  timer1Mhz = timereg + 1;
  return 1;
}

static void Timer_NanoSleep(long nanos) {
  // For smaller durations, we go straight to busy wait.
  // For larger duration, we use nanosleep() to give the operating system
  // a chance to do something else.
  // However, these timings have a lot of jitter, so we do a two way
  // approach: we use nanosleep(), but for some shorter time period so
  // that we can tolerate some jitter.
  //
  // We use the global 1Mhz hardware timer to measure the actual time period
  // that has passed, and then inch forward for the remaining time with
  // busy wait.


  // stijn: originally was 60 but I had to increase this to a much
  // higher value to account for the imprecise timers our kernel is
  // using
  static long kJitterAllowanceNanos = 60 * 1000; 
  if (nanos > kJitterAllowanceNanos + 5000) {
    const uint32_t before = *timer1Mhz;
    struct timespec sleep_time
      = { 0, nanos - kJitterAllowanceNanos };
    nanosleep(&sleep_time, NULL);
    const uint32_t after = *timer1Mhz;
    const long nanoseconds_passed = 1000 * (uint32_t)(after - before);
    if (nanoseconds_passed > nanos) {
      return;  // darn, missed it.
    } else {
      nanos -= nanoseconds_passed; // remaining time with busy-loop
    }
  }

  if (nanos < 20) return;
  // The following loop is determined empirically on a 900Mhz RPi 2
  for (uint32_t i = (nanos - 20) * 100 / 110; i != 0; --i) {
    asm("");
  }
}

volatile unsigned char interrupt_received = 0;

static void InterruptHandler(int signo) {
  interrupt_received = 1;
}

gpio_bits_t GetRowBits(int double_row) {
  gpio_bits_t row_address = (double_row & 0x01) ? PIN_A : 0;
  row_address |= (double_row & 0x02) ? PIN_B : 0;
  row_address |= (double_row & 0x04) ? PIN_C : 0;
  row_address |= (double_row & 0x08) ? PIN_D : 0;
  row_address |= (double_row & 0x10) ? PIN_E : 0;
  return row_address & row_mask;
}

gpio_bits_t GetPlaneBits(struct Pixel* top, struct Pixel* bot, uint8_t plane) {
  gpio_bits_t out = 0;
  if (top->R & (1 << plane)) out |= PIN_R1;
  if (top->G & (1 << plane)) out |= PIN_G1;
  if (top->B & (1 << plane)) out |= PIN_B1;
  if (bot->R & (1 << plane)) out |= PIN_R2;
  if (bot->G & (1 << plane)) out |= PIN_G2;
  if (bot->B & (1 << plane)) out |= PIN_B2;
  return out;
}

uint16_t RawColorToFullColor(uint8_t raw) {
  return raw * ((1 << COLOR_DEPTH) - 1) / 255;
}

void NextFrame() {
  for (int row = 0; row < ROWS; ++row) {
    for (int col = 0; col < COLUMNS; ++col) {
      struct Pixel* pix = &Frame[row][col];

      // select the image column to show in this position
      int pos = (current_position + col) % image_width;
      struct PPMPixel* raw = &image[pos + row * image_width];      

      pix->R = RawColorToFullColor(raw->R);
      pix->G = RawColorToFullColor(raw->G);
      pix->B = RawColorToFullColor(raw->B);      
    }
  }

  if (++current_position >= image_width)
    current_position = 0;
}

char* ReadLine(FILE* f, char* buffer, size_t len) {
  char* result;
  do {
    result = fgets(buffer, len, f);
  } while (result != NULL && result[0] == '#');
  return result;
}

unsigned char LoadPPM(const char *filename) {
  FILE *f = fopen(filename, "r");
  if (f == NULL) {
    if (access(filename, F_OK) == -1)
      fprintf(stderr, "File \"%s\" doesn't exist\n", filename);
    return 0;
  }
  
  char header_buf[256];
  const char *line = ReadLine(f, header_buf, sizeof(header_buf));
  if (sscanf(line, "P6 ") == EOF) {
    fprintf(stderr, "Can only handle P6 as PPM type.\n");
    return 0;
  }
  
  line = ReadLine(f, header_buf, sizeof(header_buf));
  int width, height;
  if (!line || sscanf(line, "%d %d ", &width, &height) != 2) {
    fprintf(stderr, "Width/height expected\n");
    return 0;
  }
  
  int value;
  line = ReadLine(f, header_buf, sizeof(header_buf));
  if (!line || sscanf(line, "%d ", &value) != 1 || value != 255) {    
    fprintf(stderr, "Only 255 for maxval allowed.\n");
    return 0;
  }
  
  const size_t pixel_count = width * height;
  image = (struct PPMPixel*) malloc(pixel_count * sizeof(struct PPMPixel));

  if (fread(image, sizeof(struct PPMPixel), pixel_count, f) != pixel_count) {
    fprintf(stderr, "Not enough pixels read.\n");
    return 0;
  }

  if (height != ROWS) {
    fprintf(stderr, "invalid image dimensions - read height: %d\n", height);
    return 0;
  }

  image_width = width;
  
  fclose(f);
  fprintf(stderr, "Read image '%s' with %dx%d\n", filename,
	  width, height);  
  
  return 1;
}

int main(int argc, char *argv[]) {
  if (getuid() != 0) {
    fprintf(stderr, "Must run as root to be able to access /dev/mem\n"
	    "Prepend 'sudo' to the command\n");
    return 1;
  }

  if (argc < 2) {
    fprintf(stderr, "Syntax: %s [image]\n", argv[0]);
    return 1;
  }

  if (LoadPPM(argv[1]) == 0) {
    return 1;
  }
  
  NextFrame();

  struct GPIO* io = GPIO_New(1, ROWS);
  if (!io) {
    return 1;
  }

  (void) Timer_Init();
  
  // Set up an interrupt handler to be able to stop animations while they go
  // on. Note, each demo tests for while (running() && !interrupt_received) {},
  // so they exit as soon as they get a signal.
  signal(SIGTERM, InterruptHandler);
  signal(SIGINT, InterruptHandler);

  // Now, the image generation runs in the background. We can do arbitrary
  // things here in parallel. In this demo, we're essentially just
  // waiting for one of the conditions to exit.
  printf("Press <CTRL-C> to exit and reset LEDs\n");

  struct timeval prev_frame_time;
  struct timeval current_time;

  gettimeofday(&prev_frame_time, NULL);

  while (!interrupt_received) {
    gpio_bits_t color_clk_mask = 0;  // Mask of bits while clocking in.
    color_clk_mask |= PIN_R1 | PIN_G1 | PIN_B1 | PIN_R2 | PIN_G2 | PIN_B2 | PIN_CLK;

    for (uint8_t row_loop = 0; row_loop < ROWS / 2; ++row_loop) {
      for (unsigned b = 0; b < COLOR_DEPTH; ++b) {
	for (unsigned col = 0; col < 32; ++col) {
	  struct Pixel* top = &Frame[row_loop][col];
	  struct Pixel* bot = &Frame[ROWS / 2 + row_loop][col];
	  GPIO_WriteMaskedBits(io, GetPlaneBits(top, bot, b), color_clk_mask);  // col + reset clock
	  GPIO_SetBits(io, PIN_CLK);               // Rising edge: clock color in.
	}
	GPIO_ClearBits(io, color_clk_mask);    // clock back to normal.
      
	// Setting address and strobing needs to happen in dark time.
	GPIO_WriteMaskedBits(io, GetRowBits(row_loop), row_mask);
      
	GPIO_SetBits(io, PIN_LAT);   // Strobe in the previously clocked in row.
	GPIO_ClearBits(io, PIN_LAT);
	
	GPIO_ClearBits(io, PIN_OE);
	Timer_NanoSleep(bitplane_timings[b]);
	GPIO_SetBits(io, PIN_OE);       
      }
    }

    // see if we should calculate the next frame
    gettimeofday(&current_time, NULL);
    
    int64_t usec_since_prev_frame = (current_time.tv_sec - prev_frame_time.tv_sec) * 1000 * 1000 +
      (current_time.tv_usec - prev_frame_time.tv_usec);
    
    if (usec_since_prev_frame >= 40000) {
      prev_frame_time.tv_sec = current_time.tv_sec;
      prev_frame_time.tv_usec = current_time.tv_usec;
      NextFrame();
    }
  }    
  
  printf("\%s. Exiting.\n",
         interrupt_received ? "Received CTRL-C" : "Timeout reached");
  return 0;
}
