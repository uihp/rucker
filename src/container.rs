use crate::errors::ErrorType;
use crate::ipc::create_socketpair;
use crate::utils::random_hex_string;
use crate::childproc::ChildProcess;

use std::path::PathBuf;
use std::ffi::CString;
use std::os::fd::OwnedFd;

pub struct Container {
    pub mount_dir: PathBuf,
    pub exec_command: CString,
    pub addmntpts: Vec<(PathBuf, PathBuf)>,
    pub hostname: String,
    pub socket_pair: (OwnedFd, OwnedFd),
    pub child_proc: Option<ChildProcess>
}

impl Container {
    pub fn new(mount_dir: PathBuf, exec_command: CString, addmntpts: Vec<(PathBuf, PathBuf)>) -> Result<Container, ErrorType> {
        let hostname = random_hex_string();
        let socket_pair = create_socketpair()?;
        Ok(Container { mount_dir, exec_command, addmntpts, hostname, socket_pair, child_proc: None })
    }
    pub fn create(&mut self) -> Result<(), ErrorType> {
        let child_process = self.create_child_process()?;
        self.child_proc = Some(child_process);
        println!("Successfully created container");
        Ok(())
    }
    pub fn destroy(&mut self) {
        
    }
}

pub fn run(mount_dir: String, exec_command: String, addmntpts: Vec<String>) -> Result<(), ErrorType> {
    let mount_dir = PathBuf::from(mount_dir);
    let exec_command = CString::new(exec_command).map_err(ErrorType::CStringError)?;
    let addmntpts = addmntpts.into_iter().map(|s| {
        let mut pair = s.split(":");
        let src = PathBuf::from(pair.next().unwrap())
            .canonicalize().expect("PanickedException: Cannot canonicalize path").to_path_buf();
        let mnt = PathBuf::from(pair.next().unwrap())
            .strip_prefix("/").expect("PanickedException: Cannot strip prefix from path").to_path_buf();
        (src, mnt)
    }).collect();
    let mut container = Container::new(mount_dir, exec_command, addmntpts)?;
    container.create()?;
    if let Some(ref mut child_proc) = container.child_proc {
        println!("Container child PID: {:?}", child_proc.pid);
        child_proc.wait()?;
        println!("Finished, cleaning & exit");
    }
    container.destroy();
    Ok(())
}
