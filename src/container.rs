use crate::errors::ErrorType;
use crate::utils::random_hex_string;
use crate::ipc::{create_socketpair, send_boolean, recv_boolean};
use crate::childproc::ChildProcess;

use std::ffi::CString;
use std::os::fd::OwnedFd;
use std::path::PathBuf;

pub struct Container {
    pub id: String,
    pub exec_command: CString,
    pub mount_dir: PathBuf,
    pub addmntpts: Vec<(PathBuf, PathBuf)>,
    pub child_proc: Option<ChildProcess>,
    pub socket_pair: (OwnedFd, OwnedFd),
    pub uid: u32
}

impl Container {
    pub fn new(exec_command: CString, mount_dir: PathBuf, addmntpts: Vec<(PathBuf, PathBuf)>, uid: u32) -> Result<Container, ErrorType> {
        let id = random_hex_string();
        log::info!("Successfully newed a container, ID: {}", id);
        Ok(Container { id, exec_command, mount_dir, addmntpts, socket_pair: create_socketpair()?, child_proc: None, uid })
    }
    pub fn create(&mut self) -> Result<(), ErrorType> {
        let child_process = self.create_child_process()?;
        log::info!("Successfully created child process: {:?}", child_process.pid);
        self.child_proc = Some(child_process);
        if recv_boolean(&self.socket_pair.0)? {
            self.map_child_uid()?;
            send_boolean(&self.socket_pair.0, true)?;
            log::info!("Successfully mapped UID/GID for child process");
        } else {
            log::debug!("Namespace issue, skipped id mapping");
            send_boolean(&self.socket_pair.0, false)?;
        }
        Ok(())
    }
    pub fn destroy(&mut self) {
        log::info!("Finished, cleaning & exit");
    }
}

pub fn run(exec_command: String, mount_dir: PathBuf, addmntpts: Vec<String>, uid: u32) -> Result<(), ErrorType> {
    let exec_command = CString::new(exec_command).map_err(ErrorType::CStringError)?;
    let addmntpts = addmntpts.into_iter().map(|s| {
        let mut pair = s.split(":");
        let src = PathBuf::from(pair.next().unwrap())
            .canonicalize().expect("PanickedException: Cannot canonicalize the source path").to_path_buf();
        let mnt = PathBuf::from(pair.next().unwrap())
            .strip_prefix("/").expect("PanickedException: Cannot strip prefix ('/') from path: mount path should be absolute").to_path_buf();
        (src, mnt)
    }).collect();
    let mut container = Container::new(exec_command, mount_dir, addmntpts, uid)?;
    container.create().or_else(|err| { container.destroy(); Err(err) })?;
    container.child_proc.as_mut().unwrap().wait().or_else(|err| { container.destroy(); Err(err) })?;
    container.destroy();
    Ok(())
}
