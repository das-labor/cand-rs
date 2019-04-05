type CanAddr = uchar;
type CanPort = uchar;
type CanChannel = u16;
type CanSubchannel = u8;

struct CanMessageRaw {
    id: u32,
    dlc: u8,
    data: [u8; 8]
}

struct CanMessage {
    addr_src: can_addr,
    addr_dst: can_addr,
    port_src: can_port,
    port_dst: can_port,
    dlc: uchar,
    data: [u8; 8]
}

struct CanMessageV2 {
    channel: CanChannel,
    subchannel: CanSubchannel,
    addr_src: CanAddr,
    addr_dst: CanAddr,
    dlc: u8,
    data: [u8; 8]
}

enum CanMode {
    Normal,
    Sleep,
    Loopback,
    ListenOnly,
    Config
}

// all AVR below here, probably

// Management

fn can_init() {

}

fn can_setfilter() {

}

fn can_setmode(mode: CanMode) {

}

fn can_setled(led: uchar, state: uchar) {

}

// Sending

fn can_buffer_get() -> CanMessage {

}

fn can_transmit(msg: CanMessage) {

}

fn can_transmit_raw_gateway_message(rmsg: RS232CanMsg) {

}


/*****************************************************************************
 * Receiving
 */

fn can_get() -> CanMessage {

}

fn can_get_nb() -> CanMessage {

}

fn can_free(msg: CanMessage) {

}


/*****************************************************************************
 * Sending
 */

fn can_buffer_get_raw() -> CanMessageRaw {

}

fn can_transmit_raw(msg: CanMessageRaw) {

}

/*****************************************************************************
 * Receiving
 */

fn can_get_raw() -> CanMessageRaw {

}

fn can_get_raw_nb() -> CanMessageRaw {

}

fn can_free_raw(msg: CanMessageRaw) {

}

/*****************************************************************************
 * Sending
 */

fn can_transmit_v2(msg: CanMessageV2) {

}

/*****************************************************************************
 * Receiving
 */


fn can_get_v2_nb() -> CanMessageV2 {

}

fn can_free_v2(msg: CanMessageV2) {

}