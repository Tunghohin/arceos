use lazy_static::*;
use spinlock::SpinNoIrq;
use x86_64::instructions::port::{Port, PortReadOnly, PortWriteOnly};

const BUFFER_SIZE: usize = 128;

const KEYBOARD_DATA: u16 = 0x60;
const KEYBOARD_STATUS: u16 = 0x64;
const KEYBOARD_COMMAND: u16 = 0x64;

const KEYBOARD_IRQ: u16 = 0x21;

struct RingBuffer {
    head: usize,
    tail: usize,
    inner: [u8; BUFFER_SIZE as usize],
}

impl RingBuffer {
    const fn new() -> Self {
        Self {
            head: 0,
            tail: 0,
            inner: [0; BUFFER_SIZE as usize],
        }
    }
    fn read(&mut self) -> Option<u8> {
        if self.head == self.tail {
            None
        } else {
            let tmp_pos = self.head;
            self.head = (self.head + 1) % BUFFER_SIZE;
            Some(self.inner[tmp_pos])
        }
    }

    fn write(&mut self, data: u8) {
        let tmp_pos = (self.tail + 1) % BUFFER_SIZE;
        if tmp_pos == self.head {
            return;
        }
        self.inner[self.tail] = data;
        self.tail = tmp_pos;
    }
}

struct KeyBoard {
    buffer: RingBuffer,
    port: PortReadOnly<u8>,
    is_capslock: bool,
    is_shifted_l: bool,
    is_shifted_r: bool,
}


impl KeyBoard {
    const fn new() -> Self {
        Self {
            buffer: RingBuffer::new(),
            port: PortReadOnly::new(KEYBOARD_DATA),
            is_capslock: false,
            is_shifted_l: false,
            is_shifted_r: false,
        }
    }

    fn is_shifted(&self) -> bool {
        self.is_shifted_l | self.is_shifted_r
    }

    fn is_capslock(&self) -> bool {
        self.is_capslock
    }

    fn check_status_n_change(&mut self, scancode: u8) {
        match scancode {
            0x2a => {
                self.is_shifted_l = true;
            }
            0x36 => {
                self.is_shifted_r = true;
            }
            0x3a => {
                self.is_capslock = !self.is_capslock;
            }
            0xaa => {
                self.is_shifted_l = false;
            }
            0x36 => {
                self.is_shifted_r = false;
            }
            _ => {}
        }
    }

    fn getchar(&mut self) -> Option<u8> {
        self.buffer.read()
    }
}

struct KeyCode {
    ascii1: u8,
    ascii2: u8,
    scode: u8,
    kcode: u8,
}

static KEY_MAP: [KeyCode; 94] = [
/* 0x00 - none*/	KeyCode{ ascii1:0, ascii2:0, scode: 0,kcode:0},
/*0x01-ESC*/	KeyCode{ascii1:0, ascii2:0, scode: 0x01,kcode:0x1B},
/*0x02-'1'*/	KeyCode{ascii1:'1' as u8 , ascii2:'!' as u8 , scode: 0x02,kcode:0x31},
/*0x03-'2'*/	KeyCode{ascii1:'2' as u8 , ascii2:'@' as u8 , scode: 0x03,kcode:0x32},
/*0x04-'3'*/	KeyCode{ascii1:'3' as u8 , ascii2:'#' as u8 , scode: 0x04,kcode:0x33},
/*0x05-'4'*/	KeyCode{ascii1:'4' as u8 , ascii2:'$' as u8 , scode: 0x05,kcode:0x34},
/*0x06-'5'*/	KeyCode{ascii1:'5' as u8 , ascii2:'%' as u8 , scode: 0x06,kcode:0x35},
/*0x07-'6'*/	KeyCode{ascii1:'6' as u8 , ascii2:'^' as u8 , scode: 0x07,kcode:0x36},
/*0x08-'7'*/	KeyCode{ascii1:'7' as u8 , ascii2:'&' as u8 , scode: 0x08,kcode:0x37},
/*0x09-'8'*/	KeyCode{ascii1:'8' as u8 , ascii2:'*' as u8 , scode: 0x09,kcode:0x38},
/*0x0A-'9'*/	KeyCode{ascii1:'9' as u8 , ascii2:'(' as u8 , scode: 0x0A,kcode:0x39},
/*0x0B-'0'*/	KeyCode{ascii1:'0' as u8 , ascii2:')' as u8 , scode: 0x0B,kcode:0x30},
/*0x0C-'-'*/	KeyCode{ascii1:'-' as u8 , ascii2:'_' as u8 , scode: 0x0C,kcode:0xBD},
/*0x0D-'='*/	KeyCode{ascii1:'=' as u8 , ascii2:'+' as u8 , scode: 0x0D,kcode:0xBB},
/*0x0E-BS*/	KeyCode{ascii1:b'\x08' as u8, ascii2:b'\x08' as u8, scode: 0x0E,kcode:0x08},
/*0x0F-TAB*/	KeyCode{ascii1:0, ascii2:0, scode: 0x0F,kcode:0x09},
/*0x10-'q'*/	KeyCode{ascii1:'q' as u8 , ascii2:'Q' as u8 , scode: 0x10,kcode:0x51},
/*0x11-'w'*/	KeyCode{ascii1:'w' as u8 , ascii2:'W' as u8 , scode: 0x11,kcode:0x57},
/*0x12-'e'*/	KeyCode{ascii1:'e' as u8 , ascii2:'E' as u8 , scode: 0x12,kcode:0x45},
/*0x13-'r'*/	KeyCode{ascii1:'r' as u8 , ascii2:'R' as u8 , scode: 0x13,kcode:0x52},
/*0x14-'t'*/	KeyCode{ascii1:'t' as u8 , ascii2:'T' as u8 , scode: 0x14,kcode:0x54},
/*0x15-'y'*/	KeyCode{ascii1:'y' as u8 , ascii2:'Y' as u8 , scode: 0x15,kcode:0x59},
/*0x16-'u'*/	KeyCode{ascii1:'u' as u8 , ascii2:'U' as u8 , scode: 0x16,kcode:0x55},
/*0x17-'i'*/	KeyCode{ascii1:'i' as u8 , ascii2:'I' as u8 , scode: 0x17,kcode:0x49},
/*0x18-'o'*/	KeyCode{ascii1:'o' as u8 , ascii2:'O' as u8 , scode: 0x18,kcode:0x4F},
/*0x19-'p'*/	KeyCode{ascii1:'p' as u8 , ascii2:'P' as u8 , scode: 0x19,kcode:0x50},
/*0x1A-'['*/	KeyCode{ascii1:'[' as u8 ,ascii2:'{' as u8 , scode: 0x1A,kcode:0xDB},
/*0x1B-']'*/	KeyCode{ascii1:']' as u8 ,ascii2:'}' as u8 , scode: 0x1B,kcode:0xDD},
/*0x1C-CR/LF*/	KeyCode{ascii1:b'\n' as u8, ascii2:b'\n' as u8, scode: 0x1C,kcode:0x0D},
/*0x1D-l.Ctrl*/	KeyCode{ascii1:0, ascii2:0, scode: 0x1D,kcode:0x11},
/*0x1E-'a'*/	KeyCode{ascii1:'a' as u8 , ascii2:'A' as u8 , scode: 0x1E,kcode:0x41},
/*0x1F-'s'*/	KeyCode{ascii1:'s' as u8 , ascii2:'S' as u8 , scode: 0x1F,kcode:0x53},
/*0x20-'d'*/	KeyCode{ascii1:'d' as u8 , ascii2:'D' as u8 , scode: 0x20,kcode:0x44},
/*0x21-'f'*/	KeyCode{ascii1:'f' as u8 , ascii2:'F' as u8 , scode: 0x21,kcode:0x46},
/*0x22-'g'*/	KeyCode{ascii1:'g' as u8 , ascii2:'G' as u8 , scode: 0x22,kcode:0x47},
/*0x23-'h'*/	KeyCode{ascii1:'h' as u8 , ascii2:'H' as u8 , scode: 0x23,kcode:0x48},
/*0x24-'j'*/	KeyCode{ascii1:'j' as u8 , ascii2:'J' as u8 , scode: 0x24,kcode:0x4A},
/*0x25-'k'*/	KeyCode{ascii1:'k' as u8 , ascii2:'K' as u8 , scode: 0x25,kcode:0x4B},
/*0x26-'l'*/	KeyCode{ascii1:'l' as u8 , ascii2:'L' as u8 , scode: 0x26,kcode:0x4C},
/*0x27-';'*/	KeyCode{ascii1:';' as u8 , ascii2:':' as u8 , scode: 0x27,kcode:0xBA},
/*0x28-'\''*/	KeyCode{ascii1:'\'' as u8 , ascii2:'\"' as u8 , scode: 0x28,kcode:0xDE},
/*0x29-'`'*/	KeyCode{ascii1:'`' as u8 , ascii2:'~' as u8 , scode: 0x29,kcode:0xC0},
/*0x2A-l.SHIFT*/	KeyCode{ascii1:0, ascii2:0, scode: 0x2A,kcode:0x10},
/*0x2B-'\'*/	KeyCode{ascii1:'\\' as u8 , ascii2:'|' as u8 , scode: 0x2B,kcode:0xDC},
/*0x2C-'z'*/	KeyCode{ascii1:'z' as u8 , ascii2:'Z' as u8 , scode: 0x2C,kcode:0x5A},
/*0x2D-'x'*/	KeyCode{ascii1:'x' as u8 , ascii2:'X' as u8 , scode: 0x2D,kcode:0x58},
/*0x2E-'c'*/	KeyCode{ascii1:'c' as u8 , ascii2:'C' as u8 , scode: 0x2E,kcode:0x43},
/*0x2F-'v'*/	KeyCode{ascii1:'v' as u8 , ascii2:'V' as u8 , scode: 0x2F,kcode:0x56},
/*0x30-'b'*/	KeyCode{ascii1:'b' as u8 , ascii2:'B' as u8 , scode: 0x30,kcode:0x42},
/*0x31-'n'*/	KeyCode{ascii1:'n' as u8 , ascii2:'N' as u8 , scode: 0x31,kcode:0x4E},
/*0x32-'m'*/	KeyCode{ascii1:'m' as u8 , ascii2:'M' as u8 , scode: 0x32,kcode:0x4D},
/*0x33-' as u8 ,'*/	KeyCode{ascii1:',' as u8 , ascii2:'<' as u8 , scode: 0x33,kcode:0xBC},
/*0x34-'.'*/	KeyCode{ascii1:'.' as u8 , ascii2:'>' as u8 , scode: 0x34,kcode:0xBE},
/*0x35-'/'*/	KeyCode{ascii1:'/' as u8 , ascii2:'?' as u8 , scode: 0x35,kcode:0xBF},
/*0x36-r.SHIFT*/	KeyCode{ascii1:0, ascii2:0, scode: 0x36,kcode:0x10},
/*0x37-'*'*/	KeyCode{ascii1:'*' as u8 , ascii2:'*' as u8 , scode: 0x37,kcode:0x6A},
/*0x38-ALT*/	KeyCode{ascii1:0, ascii2:0, scode: 0x38,kcode:0x12},
/*0x39-''*/	KeyCode{ascii1:' ' as u8 ,ascii2:0, scode: 0x39,kcode:0x20},
/*0x3A-CapsLock*/	KeyCode{ascii1:0, ascii2:0, scode: 0x3A,kcode:0x14},
/*0x3B-F1*/	KeyCode{ascii1:0, ascii2:0, scode: 0x3B,kcode:0x70},
/*0x3C-F2*/	KeyCode{ascii1:0, ascii2:0, scode: 0x3C,kcode:0x71},
/*0x3D-F3*/	KeyCode{ascii1:0, ascii2:0, scode: 0x3D,kcode:0x72},
/*0x3E-F4*/	KeyCode{ascii1:0, ascii2:0, scode: 0x3E,kcode:0x73},
/*0x3F-F5*/	KeyCode{ascii1:0, ascii2:0, scode: 0x3F,kcode:0x74},
/*0x40-F6*/	KeyCode{ascii1:0, ascii2:0, scode: 0x40,kcode:0x75},
/*0x41-F7*/	KeyCode{ascii1:0, ascii2:0, scode: 0x41,kcode:0x76},
/*0x42-F8*/	KeyCode{ascii1:0, ascii2:0, scode: 0x42,kcode:0x77},
/*0x43-F9*/	KeyCode{ascii1:0, ascii2:0, scode: 0x43,kcode:0x78},
/*0x44-F10*/	KeyCode{ascii1:0, ascii2:0, scode: 0x44,kcode:0x79},
/*0x45-NumLock*/	KeyCode{ascii1:0, ascii2:0, scode: 0x45,kcode:0x90},
/*0x46-ScrLock*/	KeyCode{ascii1:0, ascii2:0, scode: 0x46,kcode:0x91},
/*0x47-Home*/	KeyCode{ascii1:0, ascii2:0, scode: 0x47,kcode:0x24},
/*0x48-Up*/	KeyCode{ascii1:0, ascii2:0, scode: 0x48,kcode:0x26},
/*0x49-PgUp*/	KeyCode{ascii1:0, ascii2:0, scode: 0x49,kcode:0x21},
/*0x4A-'-'*/	KeyCode{ascii1:0, ascii2:0, scode: 0x4A,kcode:0x6D},
/*0x4B-Left*/	KeyCode{ascii1:0, ascii2:0, scode: 0x4B,kcode:0x25},
/*0x4C-MID*/	KeyCode{ascii1:0, ascii2:0, scode: 0x4C,kcode:0x0C},
/*0x4D-Right*/	KeyCode{ascii1:0, ascii2:0, scode: 0x4D,kcode:0x27},
/*0x4E-'+'*/	KeyCode{ascii1:0, ascii2:0, scode: 0x4E,kcode:0x6B},
/*0x4F-End*/	KeyCode{ascii1:0, ascii2:0, scode: 0x4F,kcode:0x23},
/*0x50-Down*/	KeyCode{ascii1:0, ascii2:0, scode: 0x50,kcode:0x28},
/*0x51-PgDown*/	KeyCode{ascii1:0, ascii2:0, scode: 0x51,kcode:0x22},
/*0x52-Insert*/	KeyCode{ascii1:0, ascii2:0, scode: 0x52,kcode:0x2D},
/*0x53-Del*/	KeyCode{ascii1:b'\x7f' as u8, ascii2:b'\x7f' as u8, scode: 0x53,kcode:0x2E},
/*0x54-Enter*/	KeyCode{ascii1:b'\n' as u8, ascii2:b'\n' as u8, scode: 0x54,kcode:0x0D},
/*0x55-???*/	KeyCode{ascii1:0, ascii2:0, scode: 0,kcode:0},
/*0x56-???*/	KeyCode{ascii1:0, ascii2:0, scode: 0,kcode:0},
/*0x57-F11*/	KeyCode{ascii1:0, ascii2:0, scode: 0x57,kcode:0x7A},
/*0x58-F12*/	KeyCode{ascii1:0, ascii2:0, scode: 0x58,kcode:0x7B},
/*0x59-???*/	KeyCode{ascii1:0, ascii2:0, scode: 0,kcode:0},
/*0x5A-???*/	KeyCode{ascii1:0, ascii2:0, scode: 0,kcode:0},
/*0x5B-LeftWin*/	KeyCode{ascii1:0, ascii2:0, scode: 0x5B,kcode:0x5B},
/*0x5C-RightWin*/	KeyCode{ascii1:0, ascii2:0, scode: 0x5C,kcode:0x5C},
/*0x5D-Apps*/	KeyCode{ascii1:0, ascii2:0, scode: 0x5D,kcode:0x5D}
];


lazy_static! {
    static ref KEYBOARD: SpinNoIrq<KeyBoard> = SpinNoIrq::new(KeyBoard::new());
}

fn decode(scancode: u8) -> Option<u8> {
    let is_shifted = KEYBOARD.lock().is_shifted();
    let is_capslock = KEYBOARD.lock().is_capslock();
    match scancode {
        0..=0x5D => {
            if is_shifted ^ is_capslock {
                Some(KEY_MAP[scancode as usize].ascii2)
            } else {
                Some(KEY_MAP[scancode as usize].ascii1)
            }
        },
        _ => None,
    }
}

fn keyboard_intrrupt_handler() {
    use x86_64::instructions::port::Port;
    let mut port = Port::new(0x60);
    let scancode: u8 = unsafe { port.read() };

    //change status
    KEYBOARD.lock().check_status_n_change(scancode);

    if let Some(c) = decode(scancode) {
        KEYBOARD.lock().buffer.write(c);
    }
}

pub fn getchar() -> Option<u8> {
    KEYBOARD.lock().getchar()
}

pub(super) fn init() {
    #[cfg(feature = "irq")]
    crate::irq::register_handler(KEYBOARD_IRQ.into(), keyboard_intrrupt_handler);
}
