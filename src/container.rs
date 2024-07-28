use crate::errors::ErrorType;
use crate::ipc::create_socketpair;
use crate::utils::random_hex_string;
use crate::childproc::ChildProcess;

use std::ffi::CString;
use std::path::PathBuf;
use std::os::fd::OwnedFd;

pub struct Container {
    pub exec_command: CString,
    pub mount_dir: PathBuf,
    pub addmntpts: Vec<(PathBuf, PathBuf)>,
    pub child_proc: Option<ChildProcess>,
    pub hostname: String,
    pub socket_pair: (OwnedFd, OwnedFd)
}

impl Container {
    pub fn new(exec_command: CString, mount_dir: PathBuf, addmntpts: Vec<(PathBuf, PathBuf)>) -> Result<Container, ErrorType> {
        let hostname = random_hex_string();
        let socket_pair = create_socketpair()?;
        Ok(Container { mount_dir, exec_command, addmntpts, hostname, socket_pair, child_proc: None })
    }
    pub fn create(&mut self) -> Result<(), ErrorType> {
        let child_process = self.create_child_process()?;
        self.child_proc = Some(child_process);
        log::info!("Successfully created container");
        Ok(())
    }
    pub fn destroy(&mut self) {
        
    }
}

pub fn run(exec_command: String, mount_dir: PathBuf, addmntpts: Vec<String>) -> Result<(), ErrorType> {
    let exec_command = CString::new(exec_command).map_err(ErrorType::CStringError)?;
    let addmntpts = addmntpts.into_iter().map(|s| {
        let mut pair = s.split(":");
        let src = PathBuf::from(pair.next().unwrap())
            .canonicalize().expect("PanickedException: Cannot canonicalize the source path").to_path_buf();
        let mnt = PathBuf::from(pair.next().unwrap())
            .strip_prefix("/").expect("PanickedException: Cannot strip prefix ('/') from path: mount path should be absolute").to_path_buf();
        (src, mnt)
    }).collect();
    let mut container = Container::new(exec_command, mount_dir, addmntpts)?;
    container.create()?;
    if let Some(ref mut child_proc) = container.child_proc {
        log::info!("Container child PID: {:?}", child_proc.pid);
        child_proc.wait()?;
        log::info!("Finished, cleaning & exit");
    }
    container.destroy();
    Ok(())
}
