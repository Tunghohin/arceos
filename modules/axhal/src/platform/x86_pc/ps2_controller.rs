use lazy_static::*;
use spinlock::SpinNoIrq;
use x86_64::instructions::port::{Port, PortReadOnly, PortWriteOnly};

const BUFFER_SIZE: usize = 128;

const KEYBOARD_DATA: u16 = 0x60;
const KEYBOARD_STATUS: u16 = 0x64;
const KEYBOARD_COMMAND: u16 = 0x64;

const KEYBOARD_IRQ: u16 = 0x21;

const KBD_US_UPPER: [Option<char>; 58] = [
    None,
    None,
    Some('!'),
    Some('@'),
    Some('#'),
    Some('$'),
    Some('%'),
    Some('^'),
    Some('&'),
    Some('*'),
    Some('('),
    Some(')'),
    Some('_'),
    Some('+'),
    None,       //backspace
    Some('\t'), //tab
    Some('Q'),
    Some('W'),
    Some('E'),
    Some('R'),
    Some('T'),
    Some('Y'),
    Some('U'),
    Some('I'),
    Some('O'),
    Some('P'),
    Some('{'),
    Some('}'),
    Some('\n'), // enter
    None,       // left ctrl
    Some('A'),
    Some('S'),
    Some('D'),
    Some('F'),
    Some('G'),
    Some('H'),
    Some('J'),
    Some('K'),
    Some('L'),
    Some(':'),
    Some('"'),
    Some('~'),
    None, //left shift
    Some('|'),
    Some('Z'),
    Some('X'),
    Some('C'),
    Some('V'),
    Some('B'),
    Some('N'),
    Some('M'),
    Some('<'),
    Some('>'),
    Some('?'),
    None, //right shift
    None,
    None,      //left alt
    Some(' '), //space
];

const KBD_US_LOWER: [Option<char>; 58] = [
    None,
    None,
    Some('1'),
    Some('2'),
    Some('3'),
    Some('4'),
    Some('5'),
    Some('6'),
    Some('7'),
    Some('8'),
    Some('9'),
    Some('0'),
    Some('-'),
    Some('+'),
    None,       //backspace
    Some('\t'), //tab
    Some('q'),
    Some('w'),
    Some('e'),
    Some('r'),
    Some('t'),
    Some('y'),
    Some('u'),
    Some('i'),
    Some('o'),
    Some('p'),
    Some('['),
    Some(']'),
    Some('\n'), // enter
    None,       // left ctrl
    Some('a'),
    Some('s'),
    Some('d'),
    Some('f'),
    Some('g'),
    Some('h'),
    Some('j'),
    Some('k'),
    Some('l'),
    Some(';'),
    Some('\''),
    Some('`'),
    None, //left shift
    Some('\\'),
    Some('z'),
    Some('x'),
    Some('c'),
    Some('v'),
    Some('b'),
    Some('n'),
    Some('m'),
    Some(','),
    Some('.'),
    Some('/'),
    None, //right shift
    None,
    None,      //left alt
    Some(' '), //space
];

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

    fn getchar(&mut self) -> Option<char> {
        self.buffer.read()
    }
}

lazy_static! {
    static ref KEYBOARD: SpinNoIrq<KeyBoard> = SpinNoIrq::new(KeyBoard::new());
}

fn decode(scancode: u8) -> Option<char> {
    let is_shifted = KEYBOARD.lock().is_shifted();
    let is_capslock = KEYBOARD.lock().is_capslock();
    match scancode {
        0..=0x39 => {
            if is_shifted ^ is_capslock {
                KBD_US_UPPER[scancode as usize]
            } else {
                KBD_US_LOWER[scancode as usize]
            }
        }
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

pub fn getchar() -> Option<char> {
    KEYBOARD.lock().getchar()
}

pub(super) fn init() {
    #[cfg(feature = "irq")]
    crate::irq::register_handler(KEYBOARD_IRQ.into(), keyboard_intrrupt_handler);
}
