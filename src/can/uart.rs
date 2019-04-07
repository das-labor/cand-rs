enum CANURcvState {
    Start,
    Len,
    Payload,
    CRC
}

//static mut canu_rcvpkt: RS232CanMsg;

//static mut canu_rcvlen: u8 = 0;
//static mut canu_failcnt: u32 = 0;

pub fn canu_init(serial: &str) {
    //uart_init(serial);
    //canu_reset();
}