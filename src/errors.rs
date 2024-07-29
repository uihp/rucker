#[derive(Debug)]
pub enum ErrorType {
    CStringError(std::ffi::NulError),
    SocketPairError(nix::errno::Errno),
    SocketSendError(nix::errno::Errno),
    SocketRecvError(nix::errno::Errno),
    SocketCloseError(nix::errno::Errno),
    ChildProcessError(nix::errno::Errno),
    ExecveError(nix::errno::Errno),
    WaitingError(nix::errno::Errno),
    HostnameError(nix::errno::Errno),
    MountError(nix::errno::Errno),
    UnmountError(nix::errno::Errno),
    PivotRootError(nix::errno::Errno),
    ChDirError(nix::errno::Errno),
    DirectoryError(std::io::Error),
    FileError(std::io::Error),
    UserSysError(nix::errno::Errno)
}
