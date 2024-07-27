use crate::errors::ErrorType;

use std::os::{fd::OwnedFd, unix::io::RawFd};
use nix::sys::socket::{socketpair, AddressFamily, SockType, SockFlag, MsgFlags, send, recv};

pub fn create_socketpair() -> Result<(OwnedFd, OwnedFd), ErrorType> {
    socketpair(
        AddressFamily::Unix,
        SockType::SeqPacket,
        None,
        SockFlag::SOCK_CLOEXEC
    ).map_err(ErrorType::SocketPairError)
}

pub fn send_boolean(fd: RawFd, boolean: bool) -> Result<(), ErrorType> {
    let data: [u8; 1] = [boolean.into()];
    send(fd, &data, MsgFlags::empty()).map_err(ErrorType::SocketSendError)?;
    Ok(())
}

pub fn recv_boolean(fd: RawFd) -> Result<bool, ErrorType> {
    let mut data: [u8; 1] = [0];
    recv(fd, &mut data, MsgFlags::empty()).map_err(ErrorType::SocketRecvError)?;
    Ok(data[0] == 1)
}
