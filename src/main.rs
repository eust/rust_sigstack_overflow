use libc::{c_int, c_void, siginfo_t, SIGFPE};
use libc::{mmap, MAP_ANON, MAP_PRIVATE, PROT_READ, PROT_WRITE};
use libc::{sigaction, sigaltstack, sighandler_t, SA_ONSTACK, SA_SIGINFO, SIGSTKSZ};
use std::ptr::null_mut;
use std::thread;

extern "C" fn signal_handler(sn: c_int, _: *mut siginfo_t, _: *mut c_void) {
    eprintln!("Signal caught: {}", sn);
    heavy_fun();
    eprintln!("Handler finished");
}

fn heavy_fun() {
    eprintln!("Heavy fun");
    const S: usize = SIGSTKSZ;
    let arr = [7u8; S];
    eprintln!(
        "array start: {:?}, end: {:?}, size: {}",
        &arr as *const _,
        &arr[S - 1] as *const _,
        std::mem::size_of_val(&arr)
    );
}

unsafe fn setup_signal_handler() {
    let mut action: sigaction = std::mem::zeroed();
    action.sa_flags = SA_SIGINFO | SA_ONSTACK;
    action.sa_sigaction = signal_handler as sighandler_t;
    sigaction(SIGFPE, &action, std::ptr::null_mut());
}

unsafe fn print_sigstack_info(caller: &str) {
    let mut prev_stack = std::mem::MaybeUninit::<libc::stack_t>::uninit();
    let _res = sigaltstack(std::ptr::null(), prev_stack.as_mut_ptr());
    let ps = prev_stack.as_mut_ptr();
    eprintln!(
        "{} sigstack: {:?} size: {}",
        caller,
        (*ps).ss_sp,
        (*ps).ss_size
    );
}

static MSIZE: usize = 16 * 1024;

fn main() {
    unsafe {
        //main sigstack is already allocated here
        print_sigstack_info("main");
        setup_signal_handler();

        //this gets overwritten on MacOS
        let mem1 = mmap(
            null_mut(),
            MSIZE,
            PROT_READ | PROT_WRITE,
            MAP_PRIVATE | MAP_ANON,
            -1,
            0,
        );
        libc::memset(mem1, 1, MSIZE);
        eprintln!("mem1: {:?}-{:?}", mem1, (mem1 as usize + MSIZE) as *mut u8);
        eprintln!("last val1: {}", *((mem1 as usize + MSIZE - 1) as *mut u8));

        let handler = thread::spawn(|| {
            eprintln!("Thread spawned");
            print_sigstack_info("thread");
            //this gets overwritten on Linux
            let mem2 = mmap(
                null_mut(),
                MSIZE,
                PROT_READ | PROT_WRITE,
                MAP_PRIVATE | MAP_ANON,
                -1,
                0,
            );
            libc::memset(mem2, 2, MSIZE);
            eprintln!("mem2: {:?}-{:?}", mem2, (mem2 as usize + MSIZE) as *mut u8);
            eprintln!("last val2: {}", *((mem2 as usize + MSIZE - 1) as *mut u8));

            libc::raise(SIGFPE);

            eprintln!("last val2: {}", *((mem2 as usize + MSIZE - 1) as *mut u8));
        });
        handler.join().unwrap();
        eprintln!("Thread joined");
        eprintln!("last val1: {}", *((mem1 as usize + MSIZE - 1) as *mut u8));
    };

    eprintln!("main finished");
}
