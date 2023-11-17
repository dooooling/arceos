/// abi 调用函数
unsafe fn abi_call(addr: usize, abi_num: usize, arg0: usize) {
    core::arch::asm!("
        mv      a7, {abi_addr}
        mv      t0, {abi_num}
        slli    t0, t0, 3
        add     t1, a7, t0
        ld      t1, (t1)
        jalr    t1",
    abi_addr = in(reg) addr,
    abi_num = in(reg) abi_num,
    in("a0") arg0,
    clobber_abi("C")
    )
}

static mut ABI_ADDR: usize = 0;
const SYS_HELLO: usize = 1;
const SYS_PUTCHAR: usize = 2;
const SYS_EXIT: usize = 3;

pub fn init_abi(addr: usize) {
    unsafe {
        ABI_ADDR = addr;
    }
}

pub fn putchar(c: char) {
    unsafe { abi_call(ABI_ADDR, SYS_PUTCHAR, c as usize) }
}

pub fn hello() {
    unsafe { abi_call(ABI_ADDR, SYS_HELLO, 0) }
}

pub fn exit() {
    unsafe { abi_call(ABI_ADDR, SYS_EXIT, 0) }
}