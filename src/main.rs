use container::run;
use errors::ErrorType;

use std::process;
use users::get_current_uid;
use std::path::PathBuf;
use structopt::StructOpt;

mod utils;
mod errors;
mod container;
mod childproc;
mod internal;
mod ipc;
mod namespace;

#[derive(Debug, StructOpt)]
#[structopt(name = "rucker", about = "Linux container written in Rust")]
pub struct CLI {
    // Prints debug information
    #[structopt(long, short, global = true)]
    debug: bool,
    #[structopt(subcommand)]
    command: Command
}

#[derive(Debug, StructOpt)]
pub enum Command {
    #[structopt(name = "run", about = "Run a container from a mount directory")]
    Run(RunOptions)
}

#[derive(Debug, StructOpt)]
pub struct RunOptions {
    // Command to execute inside the container
    #[structopt(short="c", long)]
    exec_command: String,
    // Root directory inside the container to mount
    #[structopt(short, long, parse(from_os_str))]
    pub mount_dir: PathBuf,
    // Mount more directories inside the container
    #[structopt(short, long="additional_mount_dirs")]
    pub addmntpts: Vec<String>,
    // User ID to create inside the container
    #[structopt(short, long)]
    pub uid: Option<u32>
}

fn main() {
    let args = CLI::from_args();
    env_logger::Builder::from_default_env()
        .format_timestamp_secs()
        .filter(None, if args.debug { log::LevelFilter::Debug } else { log::LevelFilter::Info })
        .init();
    if get_current_uid() != 0 {
        log::error!("You need root privileges to run this program");
        process::exit(0);
    }
    match args.command {
        Command::Run(opt) => {
            match run(opt.exec_command, opt.mount_dir, opt.addmntpts) {
                Ok(()) => log::info!("All done"),
                Err(err_type) => {
                    if let ErrorType::CStringError(err) = err_type {
                        log::error!("Error converting String to CString, check the <exec_command>: {:?}", err);
                    } else if let ErrorType::SocketPairError(err) = err_type {
                        log::error!("Failed to create the socketpair: {:?}", err);
                    } else if let ErrorType::ChildProcessError(err) = err_type {
                        log::error!("Failed to setup child process: {:?}", err);
                    } else if let ErrorType::WaitingError(err) = err_type {
                        log::error!("Error while waiting for child process to finish: {:?}", err);
                    }
                }
            };
        }
    }
}
