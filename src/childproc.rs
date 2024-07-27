use crate::errors::ErrorType;
use crate::container::Container;
use crate::internal::{set_hostname, set_mountpoint};

use std::ffi::CString;
use std::os::fd::AsRawFd;
use nix::unistd::{Pid, close, execve};
use nix::sys::{signal::Signal, wait::waitpid};
use nix::sched::clone;
use nix::sched::CloneFlags;

const STACK_SIZE: usize = 1024 * 1024;

pub struct ChildProcess {
    pub pid: Pid
}

impl ChildProcess {
    pub fn of(pid: Pid) -> ChildProcess {
        ChildProcess { pid }
    }
    pub fn wait(&mut self) -> Result<(), ErrorType> {
        println!("Waiting for child_proc (pid {}) to finish", self.pid);
        waitpid(self.pid, None).map_err(ErrorType::WaitingError)?;
        Ok(())
    }
}

fn handle_internal(result: Result<isize, ErrorType>) -> isize {
    match result {
        Ok(pid) => pid,
        Err(err_type) => {
            if let ErrorType::HostnameError(err) = err_type {
                println!("Failed to set container hostname: {:?}", err);
            } else if let ErrorType::DirectoryError(err) = err_type {
                println!("DirectoryError: {:?}", err);
            } else if let ErrorType::ChDirError(err) = err_type {
                println!("ChDirError: {:?}", err);
            } else if let ErrorType::MountError(err) = err_type {
                println!("MountError: {:?}", err);
            } else if let ErrorType::UnmountError(err) = err_type {
                println!("UnmountError: {:?}", err);
            } else if let ErrorType::PivotRootError(err) = err_type {
                println!("PivotRootError: {:?}", err);
            } else if let ErrorType::SocketCloseError(err) = err_type {
                println!("Failed to close socket fd: {:?}", err);
            } else if let ErrorType::ExecveError(err) = err_type {
                println!("Failed to perform execve: {:?}", err);
            }
            -1
        }
    }
}

impl Container {
    fn child_process(&mut self) -> Result<isize, ErrorType> {
        set_hostname(&self.hostname)?;
        set_mountpoint(&self.mount_dir, &self.addmntpts)?;
        close(self.socket_pair.1.as_raw_fd()).map_err(ErrorType::SocketCloseError)?;
        println!("Starting container with <exec_command:{}>", self.exec_command.to_str().unwrap());
        execve::<CString, CString>(&self.exec_command, &[], &[]).map_err(ErrorType::ExecveError)?;
        Ok(0)
    }

    pub fn create_child_process(&mut self) -> Result<ChildProcess, ErrorType> {
        let mut tmp_stack: [u8; STACK_SIZE] = [0; STACK_SIZE];
        let mut flags = CloneFlags::empty();
        flags.insert(CloneFlags::CLONE_NEWNS);
        flags.insert(CloneFlags::CLONE_NEWCGROUP);
        flags.insert(CloneFlags::CLONE_NEWPID);
        flags.insert(CloneFlags::CLONE_NEWIPC);
        flags.insert(CloneFlags::CLONE_NEWNET);
        flags.insert(CloneFlags::CLONE_NEWUTS);
        match unsafe { clone(
            Box::new(|| handle_internal(self.child_process())),
            &mut tmp_stack,
            flags,
            Some(Signal::SIGCHLD as i32)
        ) } {
            Ok(pid) => Ok(ChildProcess::of(pid)),
            Err(err) => Err(ErrorType::ChildProcessError(err))
        }
    }
}
