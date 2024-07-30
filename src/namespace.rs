use crate::errors::ErrorType;
use crate::container::Container;
use crate::ipc::{send_boolean, recv_boolean};

use nix::sched::{unshare, CloneFlags};
use nix::unistd::{Gid, Uid};
use nix::unistd::{setgroups, setresuid, setresgid};
use nix::unistd::close;

use std::os::fd::AsRawFd;
use std::fs::File;
use std::io::Write;

const USERNS_OFFSET: u64 = 10000;
const USERNS_COUNT: u64 = 2000;

impl Container {
    pub fn setup_user_namespace(&mut self) -> Result<(), ErrorType> {
        if let Err(err) = unshare(CloneFlags::CLONE_NEWUSER) {
            log::warn!("Failed to setup user namespace, maybe not supported: {:?}", err);
            send_boolean(&self.socket_pair.1, false)?;
        } else {
            log::info!("Successfully set up user namespace");
            send_boolean(&self.socket_pair.1, true)?;
        }
        let is_mapped = recv_boolean(&self.socket_pair.1)?;
        log::debug!("Parent process acknowledged, is_mapped: {}", is_mapped);
        close(self.socket_pair.1.as_raw_fd()).map_err(ErrorType::SocketCloseError)?;
        let (uid, gid) = (Uid::from_raw(self.uid), Gid::from_raw(self.uid));
        setgroups(&[gid]).map_err(ErrorType::UserSysError)?;
        setresgid(gid, gid, gid).map_err(ErrorType::UserSysError)?;
        setresuid(uid, uid, uid).map_err(ErrorType::UserSysError)?;
        log::info!("Successfully switched to {}:{}", uid, gid);
        Ok(())
    }

    pub fn map_child_uid(&mut self) -> Result<(), ErrorType> {
        let mut uid_map = File::create(format!("/proc/{}/{}", self.child_proc.as_ref().unwrap().pid.as_raw(), "uid_map")).map_err(ErrorType::FileError)?;
        uid_map.write_all(format!("0 {} {}", USERNS_OFFSET, USERNS_COUNT).as_bytes()).map_err(ErrorType::FileError)?;
        let mut gid_map = File::create(format!("/proc/{}/{}", self.child_proc.as_ref().unwrap().pid.as_raw(), "gid_map")).map_err(ErrorType::FileError)?;
        gid_map.write_all(format!("0 {} {}", USERNS_OFFSET, USERNS_COUNT).as_bytes()).map_err(ErrorType::FileError)?;
        Ok(())
    }
}
