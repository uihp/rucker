use crate::errors::ErrorType;

use std::os::fd::{AsRawFd, OwnedFd};
use nix::sys::socket::{socketpair, AddressFamily, SockType, SockFlag, MsgFlags, send, recv};

pub fn create_socketpair() -> Result<(OwnedFd, OwnedFd), ErrorType> {
    socketpair(
        AddressFamily::Unix,
        SockType::SeqPacket,
        None,
        SockFlag::SOCK_CLOEXEC
    ).map_err(ErrorType::SocketPairError)
}

pub fn send_boolean(fd: &OwnedFd, boolean: bool) -> Result<(), ErrorType> {
    let data: [u8; 1] = [boolean.into()];
    send(fd.as_raw_fd(), &data, MsgFlags::empty()).map_err(ErrorType::SocketSendError)?;
    Ok(())
}

pub fn recv_boolean(fd: &OwnedFd) -> Result<bool, ErrorType> {
    let mut data: [u8; 1] = [0];
    recv(fd.as_raw_fd(), &mut data, MsgFlags::empty()).map_err(ErrorType::SocketRecvError)?;
    Ok(data[0] == 1)
}
