use std::env;

use cand::can::can_uart::canu_init;
use cand::can::can_tcp::*;
use std::ffi::CString;

static GREGS: [&str; 19] = [
    "GS",
    "FS",
    "ES",
    "DS",
    "EDI",
    "ESI",
    "EBP",
    "ESP",
    "EBX",
    "EDX",
    "ECX",
    "EAX",
    "TRAPNO",
    "ERR",
    "EIP",
    "CS",
    "EFL",
    "UESP",
    "SS"
];

fn event_loop() {

}

fn main() {
    let args: Vec<String> = env::args().collect();

    let port = 2342;

    let tmp = 0;
    let raw_vid: [u8; 2] = [ 0xc0, 0x16 ];
    let raw_pid: [u8; 2] = [ 0xdc, 0x05 ];

    canu_init("/dev/tty.usbmodem46041");

    cann_listen(CString::new("2342").unwrap());

    event_loop();
}
