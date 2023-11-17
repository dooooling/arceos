use axstd::println;

const SYS_HELLO: usize = 1;
const SYS_PUTCHAR: usize = 2;
const SYS_EXIT: usize = 3;

static mut ABI_TABLE: [usize; 16] = [0; 16];

pub fn abi_init() {
    register_abi(SYS_HELLO, abi_hello as usize);
    register_abi(SYS_PUTCHAR, abi_putchar as usize);
    register_abi(SYS_EXIT, abi_exit as usize);
}

fn register_abi(num: usize, handle: usize) {
    unsafe { ABI_TABLE[num] = handle; }
}

pub fn abi_hello() {
    println!("[ABI:Hello] Hello, Apps!");
}

pub fn abi_putchar(c: char) {
    println!("[ABI:Print] {c}");
}

pub fn abi_exit(code: i32) {
    println!("[ABI:Exit] exit code {code}");
    axstd::process::exit(code)
}

pub fn abi_table_ptr() -> usize {
    unsafe { ABI_TABLE.as_ptr() as usize }
}
