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
    inner: [char; BUFFER_SIZE as usize],
}

impl RingBuffer {
    const fn new() -> Self {
        Self {
            head: 0,
            tail: 0,
            inner: ['\0'; BUFFER_SIZE as usize],
        }
    }
    fn read(&mut self) -> Option<char> {
        if self.head == self.tail {
            None
        } else {
            let tmp_pos = self.head;
            self.head = (self.head + 1) % BUFFER_SIZE;
            Some(self.inner[tmp_pos])
        }
    }

    fn write(&mut self, data: char) {
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

    fn getchar(&mut self) -> Option<char> {
        self.buffer.read()
    }
}

lazy_static! {
    static ref KEYBOARD: SpinNoIrq<KeyBoard> = SpinNoIrq::new(KeyBoard::new());
}

fn decode(scancode: u8) -> Option<char> {
    let is_shifted = KEYBOARD.lock().is_shifted();
    match scancode {
        0x02 => {
            if is_shifted {
                Some('!')
            } else {
                Some('1')
            }
        }
        0x03 => {
            if is_shifted {
                Some('@')
            } else {
                Some('2')
            }
        }
        0x04 => {
            if is_shifted {
                Some('#')
            } else {
                Some('3')
            }
        }
        0x05 => {
            if is_shifted {
                Some('$')
            } else {
                Some('4')
            }
        }
        0x06 => {
            if is_shifted {
                Some('%')
            } else {
                Some('5')
            }
        }
        0x07 => {
            if is_shifted {
                Some('^')
            } else {
                Some('6')
            }
        }
        0x08 => {
            if is_shifted {
                Some('&')
            } else {
                Some('7')
            }
        }
        0x09 => {
            if is_shifted {
                Some('*')
            } else {
                Some('8')
            }
        }
        0x0a => {
            if is_shifted {
                Some('(')
            } else {
                Some('9')
            }
        }
        0x0b => {
            if is_shifted {
                Some(')')
            } else {
                Some('0')
            }
        }
        0x0c => {
            if is_shifted {
                Some('_')
            } else {
                Some('-')
            }
        }
        0x0d => {
            if is_shifted {
                Some('+')
            } else {
                Some('=')
            }
        }
        0x0f => Some('\t'),
        0x10 => {
            if is_shifted {
                Some('Q')
            } else {
                Some('q')
            }
        }
        0x11 => {
            if is_shifted {
                Some('W')
            } else {
                Some('w')
            }
        }
        0x12 => {
            if is_shifted {
                Some('E')
            } else {
                Some('e')
            }
        }
        0x13 => {
            if is_shifted {
                Some('R')
            } else {
                Some('r')
            }
        }
        0x14 => {
            if is_shifted {
                Some('T')
            } else {
                Some('t')
            }
        }
        0x15 => {
            if is_shifted {
                Some('Y')
            } else {
                Some('y')
            }
        }
        0x16 => {
            if is_shifted {
                Some('U')
            } else {
                Some('u')
            }
        }
        0x17 => {
            if is_shifted {
                Some('I')
            } else {
                Some('i')
            }
        }
        0x18 => {
            if is_shifted {
                Some('O')
            } else {
                Some('o')
            }
        }
        0x19 => {
            if is_shifted {
                Some('P')
            } else {
                Some('p')
            }
        }
        0x1a => {
            if is_shifted {
                Some('{')
            } else {
                Some('[')
            }
        }
        0x1b => {
            if is_shifted {
                Some('}')
            } else {
                Some(']')
            }
        }
        0x1c => Some('\n'),
        0x1e => {
            if is_shifted {
                Some('A')
            } else {
                Some('a')
            }
        }
        0x1f => {
            if is_shifted {
                Some('S')
            } else {
                Some('s')
            }
        }
        0x20 => {
            if is_shifted {
                Some('D')
            } else {
                Some('d')
            }
        }
        0x21 => {
            if is_shifted {
                Some('F')
            } else {
                Some('f')
            }
        }
        0x22 => {
            if is_shifted {
                Some('G')
            } else {
                Some('g')
            }
        }
        0x23 => {
            if is_shifted {
                Some('H')
            } else {
                Some('h')
            }
        }
        0x24 => {
            if is_shifted {
                Some('J')
            } else {
                Some('j')
            }
        }
        0x25 => {
            if is_shifted {
                Some('K')
            } else {
                Some('k')
            }
        }
        0x26 => {
            if is_shifted {
                Some('L')
            } else {
                Some('l')
            }
        }
        0x27 => {
            if is_shifted {
                Some(':')
            } else {
                Some(';')
            }
        }
        0x28 => {
            if is_shifted {
                Some('"')
            } else {
                Some('\'')
            }
        }
        0x29 => {
            if is_shifted {
                Some('~')
            } else {
                Some('`')
            }
        }
        0x2a => {
            KEYBOARD.lock().is_shifted_l = true;
            None
        }
        0x2b => {
            if is_shifted {
                Some('|')
            } else {
                Some('\\')
            }
        }
        0x2c => {
            if is_shifted {
                Some('Z')
            } else {
                Some('z')
            }
        }
        0x2d => {
            if is_shifted {
                Some('X')
            } else {
                Some('x')
            }
        }
        0x2e => {
            if is_shifted {
                Some('C')
            } else {
                Some('c')
            }
        }
        0x2f => {
            if is_shifted {
                Some('V')
            } else {
                Some('v')
            }
        }
        0x30 => {
            if is_shifted {
                Some('B')
            } else {
                Some('b')
            }
        }
        0x31 => {
            if is_shifted {
                Some('N')
            } else {
                Some('n')
            }
        }
        0x32 => {
            if is_shifted {
                Some('M')
            } else {
                Some('m')
            }
        }
        0x33 => {
            if is_shifted {
                Some('<')
            } else {
                Some(',')
            }
        }
        0x34 => {
            if is_shifted {
                Some('>')
            } else {
                Some('.')
            }
        }
        0x35 => {
            if is_shifted {
                Some('?')
            } else {
                Some('/')
            }
        }
        0x36 => {
            KEYBOARD.lock().is_shifted_r = true;
            None
        }
        0x39 => Some(' '),
        0xaa => {
            KEYBOARD.lock().is_shifted_l = false;
            None
        }
        0xb6 => {
            KEYBOARD.lock().is_shifted_r = false;
            None
        }
        _ => None,
    }
}

fn keyboard_intrrupt_handler() {
    use x86_64::instructions::port::Port;
    let mut port = Port::new(0x60);
    let scancode: u8 = unsafe { port.read() };
    if let Some(c) = decode(scancode) {
        KEYBOARD.lock().buffer.write(c);
    }
}

pub fn getchar() -> Option<char> {
    KEYBOARD.lock().getchar()
}

pub(super) fn init() {
    #[cfg(feature = "irq")]
    crate::irq::register_handler(KEYBOARD_IRQ.into(), keyboard_intrrupt_handler);
}
