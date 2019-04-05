const RS232CAN_MAXLENGTH: usize = 20;

use can::can;

enum Rs232CanCmd {
    Reset = 0x00,
    SetFilter = 0x10,
    Pkt = 0x11,
    SetMode = 0x12,
    Error = 0x13,
    NotifyReset = 0x14,
    PingGateway = 0x15,
    Resync = 0x16,
    Version = 0x17,
    IDString = 0x18,
    Packetcounters = 0x19,
    Errorcounters = 0x1A,
    Powerdraw = 0x1B,
    ReadCtrlReg = 0x1C,
    WriteCtrlReg = 0x1D,
    GetResetCause = 0x1E,
    NotifyTXOvf = 0x1F
}

struct RS232CanMsg {
    cmd: uchar,
    len: uchar,
    data: [uchar; RS232CAN_MAXLENGTH]
}

fn can_message_raw_from_rs232can_msg(cmsg: &mut CanMessageRaw, rmsg: &RS232CanMsg) {
    *cmsg = rmsg.data;
}

fn rs232can_msg_from_can_message_raw(rmsg: &mut RS232CanMsg, cmsg: &CanMessageRaw) {
    rmsg.cmd = Pkt;
    rmsg.len = sizeof(CanMessageRaw) + cmsg.dlc - 8;
}

fn can_message_from_can_message_raw(cmsg: &mut CanMessage, rmsg: &CanMessageRaw) {
    cmsg.addr_src = (rmsg.id >> 8) & 0xFF;
    cmsg.addr_dst = rmsg.id & 0xFF;
    cmsg.port_src = (uint8_t) ((rmsg.id >> 23) & 0x3f);
    cmsg.port_dst = (uint8_t) (((rmsg.id >> 16) & 0x0f) | ((rmsg.id >> 17) & 0x30));
    cmsg.dlc = rmsg.dlc;
    memcpy(cmsg.data, rmsg.data, rmsg.dlc);
}

fn can_message_raw_from_can_message(can_message_raw *raw_msg, can_message *cmsg) {
    memset(raw_msg, 0, sizeof(can_message_raw));

    raw_msg->id = ((cmsg->port_src & 0x3f) << 23) |
    ((cmsg->port_dst & 0x30) << 17) |
    ((cmsg->port_dst & 0x0f) << 16) |
    (cmsg->addr_src << 8) | (cmsg->addr_dst);
    raw_msg->dlc = cmsg->dlc;
    memcpy(raw_msg->data, cmsg->data, cmsg->dlc);
}