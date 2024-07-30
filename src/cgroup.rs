use crate::errors::ErrorType;
use crate::container::Container;

use cgroups_rs::cgroup_builder::CgroupBuilder;
use cgroups_rs::{MaxValue, CgroupPid};
use rlimit::{setrlimit, Resource};

use std::fs::{canonicalize, remove_dir};
use std::convert::TryInto;

const KMEM_LIMIT: i64 = 1024 * 1024 * 1024;
const MEM_LIMIT: i64 = KMEM_LIMIT;
const MAX_PID: MaxValue = MaxValue::Value(64);
const NOFILE_RLIMIT: u64 = 64;

impl Container {
    pub fn restrict_resources(&mut self) -> Result<(), ErrorType> {
        let cgroup = CgroupBuilder::new(self.id.as_str())
            .cpu().shares(256).done()
            .memory().kernel_memory_limit(KMEM_LIMIT).memory_hard_limit(MEM_LIMIT).done()
            .pid().maximum_number_of_processes(MAX_PID).done()
            .blkio().weight(50).done()
            .build(cgroups_rs::hierarchies::auto()).map_err(ErrorType::CgroupError)?;
        log::debug!("Cgroup built successfully");
        let pid : u64 = self.child_proc.as_ref().unwrap().pid.as_raw().try_into().unwrap();
        cgroup.add_task(CgroupPid::from(pid)).map_err(ErrorType::CgroupError)?;
        setrlimit(Resource::NOFILE, NOFILE_RLIMIT, NOFILE_RLIMIT).map_err(ErrorType::RlimitError)?;
        Ok(())
    }
    pub fn clean_cgroup(&mut self) -> Result<(), ErrorType> {
        let path = canonicalize(format!("/sys/fs/cgroup/{}/", self.id)).map_err(ErrorType::DirectoryError)?;
        remove_dir(path).map_err(ErrorType::DirectoryError)?;
        log::debug!("Cgroup cleaned");
        Ok(())
    }
}
