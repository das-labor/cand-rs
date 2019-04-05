use nix::libc::getaddrinfo;
use nix::libc::freeaddrinfo;
use nix::libc::gai_strerror;
use nix::libc::addrinfo;
use nix::libc::AF_UNSPEC;
use nix::libc::SOCK_STREAM;
use nix::libc::{AI_PASSIVE, AI_V4MAPPED, AI_NUMERICSERV, AI_ALL};

use std::mem;

use std::ptr::{null_mut, null};
use std::ffi::CString;
use nix::sys::socket::{socket, AddressFamily, SockType, setsockopt, SockFlag, bind, SockAddr, SockProtocol};
use nix::sys::socket::sockopt::ReuseAddr;
use nix::unistd::close;

listen_socket;

pub fn cann_listen(port: CString) {
    let mut hints = addrinfo {
        ai_family: AF_UNSPEC,
        ai_socktype: SOCK_STREAM,
        ai_flags: AI_PASSIVE | AI_V4MAPPED | AI_NUMERICSERV | AI_ALL,
        ai_addr: null_mut(),
        ai_addrlen: 0,
        ai_canonname: null_mut(),
        ai_next: null_mut(),
        ai_protocol: 0
    };

    let mut result: *mut addrinfo = unsafe { mem::uninitialized() };
    let mut rp: *mut addrinfo;

    let s = unsafe { getaddrinfo(null(), port.as_ptr(), &hints, &mut result) };

    if s != 0 {
        let error_str = unsafe { gai_strerror(s) };
        panic!("No listening addresses available(getaddrinfo: %{error_str})\n");
    }

    rp = result;
    while rp != null_mut() {
        let sfd = socket(AddressFamily::from_i32((unsafe { *rp }).ai_family).unwrap(),
                    SockType::Stream,
                  SockFlag::empty(),
                SockProtocol::Tcp
        );
        if sfd.is_err() { continue; }

        let ret = setsockopt(sfd.unwrap(), ReuseAddr, &true);

        if ret.is_err() { println!("Could not set socket options: "); }

        let ret = bind(sfd.unwrap(), & unsafe { SockAddr::from_libc_sockaddr((*rp).ai_addr).unwrap()});

        if ret.is_ok() {
            break;
        }

        panic!("Could not bind to address");

        close(sfd.unwrap());

        rp = (unsafe { *rp }).ai_next;
    }

    if rp == null_mut() {
        panic!("All addrs in use");
    } else {
        println!("Bind succeeded!");
    }

    unsafe { freeaddrinfo(result); }



}