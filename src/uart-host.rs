extern crate nix;

use nix::fcntl::{open, close, OFlag};
use nix::sys::stat::Mode;

fn uart_init(port: &str) {
    //let uart_fd = open(port, OFlag::RDWR | OFlag::NOCTTY |  OFlag::NDELAY | OFlag::NONBLOCK);

}

fn uart_close() {
    //close(ua)
}

fn uart_putc(c: char) {

}

fn uart_putstr(str: &str) {

}

fn uart_getc() -> char {

}

fn uart_getc_nb(c: &char) {

}