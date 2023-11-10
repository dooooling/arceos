use pc_keyboard::{
    DecodedKey, HandleControl, Keyboard, ScancodeSet1,
};
use pc_keyboard::layouts::Us104Key;
use x86_64::instructions::port::Port;

use spinlock::SpinNoIrq;

#[cfg(feature = "irq")]
use crate::irq;

static KEYBUFFER: SpinNoIrq<KeyboardBuffer> = SpinNoIrq::new(KeyboardBuffer::new());
static KEYBOARD: SpinNoIrq<Keyboard<Us104Key, ScancodeSet1>> = SpinNoIrq::new(
    Keyboard::new(ScancodeSet1::new(), Us104Key, HandleControl::Ignore),
);
/// 键盘中断号
const KEYBOARD_IRQ: usize = 0x21;
/// 键盘输出缓存端口
const PORT_KB_DATA: u16 = 0x60;
/// buffer大小
const BUF_SIZE: usize = 100;


/// 键盘输入buffer 环形队列？
struct KeyboardBuffer {
    buf: [u8; BUF_SIZE],
    head_index: usize,
    tail_index: usize,
    count: usize,
}

impl KeyboardBuffer {
    const fn new() -> Self {
        Self {
            buf: [0; BUF_SIZE],
            head_index: 0,
            tail_index: 0,
            count: 0,
        }
    }

    fn pop(&mut self) -> Option<u8> {
        if self.tail_index >= self.buf.len() {
            self.tail_index = 0;
        }
        let res = if self.count > 0 {
            let res = Some(self.buf[self.tail_index]);
            self.tail_index += 1;
            self.count -= 1;
            res
        } else {
            None
        };
        return res;
    }
    fn push(&mut self, data: u8) {
        if self.head_index >= self.buf.len() {
            self.head_index = 0;
        }
        self.buf[self.head_index] = data;
        self.head_index += 1;
        self.count += 1;
    }
}

fn handler() {
    let mut keybuffer = KEYBUFFER.lock();
    let scancode: u8 = unsafe { Port::new(PORT_KB_DATA).read() };

    let mut keyboard = KEYBOARD.lock();
    if let Ok(Some(key_event)) = keyboard.add_byte(scancode) {
        if let Some(key) = keyboard.process_keyevent(key_event) {
            match key {
                DecodedKey::Unicode(character) => keybuffer.push(character as u8),
                _ => {}
            }
        }
    }
}

pub(super) fn init() {
    #[cfg(feature = "irq")]
    {
        irq::register_handler(KEYBOARD_IRQ, handler);
    }
}

pub fn getchar() -> Option<u8> {
    KEYBUFFER.lock().pop()
}

