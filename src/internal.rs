use crate::errors::ErrorType;
use crate::utils::random_hex_string;
use crate::capabilities::CAPABILITIES_DROP;
use crate::syscalls::{SYSCALLS_REFUSED, SYSCALLS_CONDITIONALLY_REFUSED};

use nix::unistd::{sethostname, pivot_root, chdir};
use nix::mount::{mount, MsFlags, umount2, MntFlags};
use capctl::caps::FullCapState;
use syscallz::{Context, Action, Syscall, Comparator, Cmp};

use std::path::PathBuf;
use std::fs::{create_dir_all, remove_dir};

const EPERM: u16 = 1;

pub fn set_hostname(hostname: &String) -> Result<(), ErrorType> {
    sethostname(hostname).map_err(ErrorType::HostnameError)?;
    Ok(())
}

fn mount_directory(source: Option<&PathBuf>, mount_point: &PathBuf, flags: Vec<MsFlags>) -> Result<(), ErrorType> {
    let mut ms_flags = MsFlags::empty();
    for f in flags.iter() { ms_flags.insert(*f); }
    mount::<PathBuf, PathBuf, PathBuf, PathBuf>(source, mount_point, None, ms_flags, None).map_err(ErrorType::MountError)?;
    Ok(())
}

pub fn set_mountpoint(mount_dir: &PathBuf, addmntpts: &Vec<(PathBuf, PathBuf)>) -> Result<(), ErrorType> {
    mount_directory(None, &PathBuf::from("/"), vec![MsFlags::MS_REC, MsFlags::MS_PRIVATE])?;
    let new_root = PathBuf::from(format!("/tmp/rucker-root-mntpt-{}", random_hex_string()));
    log::debug!("Setting root mount point: {}", new_root.as_path().to_str().unwrap());
    create_dir_all(&new_root).map_err(ErrorType::DirectoryError)?;
    mount_directory(Some(&mount_dir), &new_root, vec![MsFlags::MS_BIND, MsFlags::MS_PRIVATE])?;

    log::debug!("Setting additionnal mount points");
    for (inpath, mntpath) in addmntpts.iter() {
        let outpath = new_root.join(mntpath);
        create_dir_all(&outpath).map_err(ErrorType::DirectoryError)?;
        mount_directory(Some(inpath), &outpath, vec![MsFlags::MS_PRIVATE, MsFlags::MS_BIND])?;
    }

    log::debug!("Pivoting root");
    let old_root_tail = format!("oldroot.{}", random_hex_string());
    let put_old = new_root.join(PathBuf::from(old_root_tail.clone()));
    create_dir_all(&put_old).map_err(ErrorType::DirectoryError)?;
    pivot_root(&new_root, &put_old).map_err(ErrorType::PivotRootError)?;

    log::debug!("Unmounting old root and removing directories");
    let old_root = PathBuf::from(format!("/{}", old_root_tail));
    chdir(&PathBuf::from("/")).map_err(ErrorType::ChDirError)?;
    umount2(&old_root, MntFlags::MNT_DETACH).map_err(ErrorType::UnmountError)?;
    remove_dir(&old_root.as_path()).map_err(ErrorType::DirectoryError)?;

    Ok(())
}

pub fn drop_capabilities() -> Result<(), ErrorType> {
    let mut caps = FullCapState::get_current().map_err(ErrorType::CapabilityError)?;
    caps.bounding.drop_all(CAPABILITIES_DROP.iter().map(|&cap| cap));
    caps.inheritable.drop_all(CAPABILITIES_DROP.iter().map(|&cap| cap));
    log::info!("Successfully dropped unwanted capabilities");
    Ok(())
}

fn refuse_syscall(ctx: &mut Context, syscall: &Syscall) -> Result<(), ErrorType> {
    ctx.set_action_for_syscall(Action::Errno(EPERM), *syscall).map_err(ErrorType::SyscallError)
}

fn refuse_conditionally(ctx: &mut Context, syscall: &Syscall, ind: u32, biteq: u64)-> Result<(), ErrorType> {
    ctx.set_rule_for_syscall(Action::Errno(EPERM), *syscall,
        &[Comparator::new(ind, Cmp::MaskedEq, biteq, Some(biteq))]).map_err(ErrorType::SyscallError)
}

pub fn restrict_syscalls() -> Result<(), ErrorType> {
    let mut ctx = Context::init_with_action(Action::Allow).map_err(ErrorType::SyscallError)?;
    for syscall in SYSCALLS_REFUSED.iter() { refuse_syscall(&mut ctx, syscall)?; }
    for (syscall, ind, biteq) in SYSCALLS_CONDITIONALLY_REFUSED.iter() {
        refuse_conditionally(&mut ctx, syscall, *ind, *biteq)?;
    }
    ctx.load().map_err(ErrorType::SyscallError)?;
    log::info!("Refused and filtered unwanted syscalls");
    Ok(())
}
