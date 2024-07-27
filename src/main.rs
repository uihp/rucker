use container::run;
use errors::ErrorType;

use std::{env, process};
use users::get_current_uid;

mod utils;
mod errors;
mod container;
mod childproc;
mod internal;
mod ipc;
mod namespace;

fn print_usage() {
	println!("Usage: rucker <command> [options]");
	println!("\trucker run <mount_dir> <exec_command> [...--additional_mount_dirs|-a source:mount]");
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let cmd: String = match args.get(1) {
        Some(cmd) => cmd.to_string(),
        None => {
            print_usage();
            process::exit(0);
        }
    };
    if get_current_uid() != 0 {
        println!("You need root privileges to run this program");
        process::exit(0);
    }
    match cmd.as_str() {
        "run" => {
            let mount_dir: String = match args.get(2) {
                Some(option) => option.to_string(),
                None => {
                    println!("rucker run <mount_dir> <exec_command>");
                    println!("<mount_dir> is required but not found");
                    process::exit(0);
                }
            };
            let exec_command: String = match args.get(3) {
                Some(option) => option.to_string(),
                None => {
                    println!("rucker run <mount_dir> <exec_command>");
                    println!("<exec_command> is required but not found");
                    process::exit(0);
                }
            };
            if let Some(option) = args.get(4) {
                if option != "-a" && option != "--additional_mount_dirs" {
                    println!("Unknown option: {}", option);
                    process::exit(0);
                }
            }
            match run(mount_dir, exec_command, args[5..].to_vec()) {
                Ok(()) => println!("All done"),
                Err(err_type) => {
                    if let ErrorType::CStringError(err) = err_type {
                        println!("Error converting String to CString, check the <exec_command>: {:?}", err);
                    } else if let ErrorType::SocketPairError(err) = err_type {
                        println!("Failed to create the socketpair: {:?}", err);
                    } else if let ErrorType::ChildProcessError(err) = err_type {
                        println!("Failed to setup child process: {:?}", err);
                    } else if let ErrorType::WaitingError(err) = err_type {
                        println!("Error while waiting for child process to finish: {:?}", err);
                    }
                }
            };
        }
        _ => {
            if !cmd.starts_with('-') { println!("Unknown command: {}", cmd); }
            print_usage();
            process::exit(0);
        }
    }
}
