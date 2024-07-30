use libc::TIOCSTI;
use nix::sys::stat::Mode;
use nix::sched::CloneFlags;
use syscallz::Syscall;

pub const S_ISUID: u64 = Mode::S_ISUID.bits() as u64;
pub const S_ISGID: u64 = Mode::S_ISGID.bits() as u64;
pub const CLONE_NEWUSER: u64 = CloneFlags::CLONE_NEWUSER.bits() as u64;

pub const SYSCALLS_CONDITIONALLY_REFUSED: [(Syscall, u32, u64); 9] = [
    (Syscall::chmod, 1, S_ISUID), (Syscall::chmod, 1, S_ISGID),
    (Syscall::fchmod, 1, S_ISUID), (Syscall::fchmod, 1, S_ISGID),
    (Syscall::fchmodat, 2, S_ISUID), (Syscall::fchmodat, 2, S_ISGID),
    (Syscall::unshare, 0, CLONE_NEWUSER),
    (Syscall::clone, 0, CLONE_NEWUSER),
    (Syscall::ioctl, 1, TIOCSTI),
];

pub const SYSCALLS_REFUSED: [Syscall; 9] = [
    Syscall::keyctl,
    Syscall::add_key,
    Syscall::request_key,
    Syscall::mbind,
    Syscall::migrate_pages,
    Syscall::move_pages,
    Syscall::set_mempolicy,
    Syscall::userfaultfd,
    Syscall::perf_event_open
];
