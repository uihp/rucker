use crate::errors::ErrorType;
use crate::utils::random_hex_string;

use std::path::PathBuf;
use std::fs::{create_dir_all, remove_dir};
use nix::unistd::{sethostname, pivot_root, chdir};
use nix::mount::{mount, MsFlags, umount2, MntFlags};

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
    println!("Start setting mount points ...");
    mount_directory(None, &PathBuf::from("/"), vec![MsFlags::MS_REC, MsFlags::MS_PRIVATE])?;

    let new_root = PathBuf::from(format!("/tmp/rucker-root-mntpt-{}", random_hex_string()));
    println!("Setting root mount point: {}", new_root.as_path().to_str().unwrap());
    create_dir_all(&new_root).map_err(ErrorType::DirectoryError)?;
    mount_directory(Some(&mount_dir), &new_root, vec![MsFlags::MS_BIND, MsFlags::MS_PRIVATE])?;

    println!("Setting additionnal mount points");
    for (inpath, mntpath) in addmntpts.iter() {
        let outpath = new_root.join(mntpath);
        create_dir_all(&outpath).map_err(ErrorType::DirectoryError)?;
        mount_directory(Some(inpath), &outpath, vec![MsFlags::MS_PRIVATE, MsFlags::MS_BIND])?;
    }

    println!("Pivoting root");
    let old_root_tail = format!("oldroot.{}", random_hex_string());
    let put_old = new_root.join(PathBuf::from(old_root_tail.clone()));
    create_dir_all(&put_old).map_err(ErrorType::DirectoryError)?;
    pivot_root(&new_root, &put_old).map_err(ErrorType::PivotRootError)?;

    println!("Unmounting old root and removing directories");
    let old_root = PathBuf::from(format!("/{}", old_root_tail));
    chdir(&PathBuf::from("/")).map_err(ErrorType::ChDirError)?;
    umount2(&old_root, MntFlags::MNT_DETACH).map_err(ErrorType::UnmountError)?;
    remove_dir(&old_root.as_path()).map_err(ErrorType::DirectoryError)?;

    Ok(())
}
