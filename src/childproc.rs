use crate::errors::ErrorType;
use crate::container::Container;
use crate::internal::{set_hostname, set_mountpoint};

use std::ffi::CString;
use nix::unistd::{Pid, execve};
use nix::sched::{clone, CloneFlags};
use nix::sys::{signal::Signal, wait::waitpid};

const STACK_SIZE: usize = 1024 * 1024;

pub struct ChildProcess {
    pub pid: Pid
}

impl ChildProcess {
    pub fn of(pid: Pid) -> ChildProcess {
        ChildProcess { pid }
    }
    pub fn wait(&mut self) -> Result<(), ErrorType> {
        log::debug!("Waiting for child_proc (pid {}) to finish", self.pid);
        waitpid(self.pid, None).map_err(ErrorType::WaitingError)?;
        Ok(())
    }
}

fn handle_internal(result: Result<isize, ErrorType>) -> isize {
    match result {
        Ok(pid) => pid,
        Err(err_type) => {
            if let ErrorType::HostnameError(err) = err_type {
                log::error!("Failed to set container hostname: {:?}", err);
            } else if let ErrorType::DirectoryError(err) = err_type {
                log::error!("DirectoryError: {:?}", err);
            } else if let ErrorType::ChDirError(err) = err_type {
                log::error!("ChDirError: {:?}", err);
            } else if let ErrorType::MountError(err) = err_type {
                log::error!("MountError: {:?}", err);
            } else if let ErrorType::UnmountError(err) = err_type {
                log::error!("UnmountError: {:?}", err);
            } else if let ErrorType::PivotRootError(err) = err_type {
                log::error!("Failed to pivot root: {:?}", err);
            } else if let ErrorType::SocketCloseError(err) = err_type {
                log::error!("Failed to close socket fd: {:?}", err);
            } else if let ErrorType::SocketSendError(err) = err_type {
                log::error!("Failed to send via socket: {:?}", err);
            } else if let ErrorType::SocketRecvError(err) = err_type {
                log::error!("Failed to recv via socket: {:?}", err);
            } else if let ErrorType::ExecveError(err) = err_type {
                log::error!("Failed to perform execve: {:?}", err);
            } else if let ErrorType::UserSysError(err) = err_type {
                log::error!("Failed to switch uid for child process: {:?}", err);
            }
            -1
        }
    }
}

impl Container {
    fn child_process(&mut self) -> Result<isize, ErrorType> {
        set_hostname(&self.id)?;
        set_mountpoint(&self.mount_dir, &self.addmntpts)?;
        self.setup_user_namespace()?;
        log::info!("Starting container with <exec_command:{}>", self.exec_command.to_str().unwrap());
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
