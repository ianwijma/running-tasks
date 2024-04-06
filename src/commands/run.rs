use std::fmt::Debug;
use std::path::PathBuf;
use std::process::Command;
use std::thread;
use std::thread::JoinHandle;
use std::time::Instant;
use clap::Args;
use crate::utils::config;
use crate::utils::config::{Config, ConfigStructure, OrderedTask, OrderedTasks, Task, TaskExit};
use crate::utils::file::ConfigFile;

#[derive(Args, Debug)]
pub struct Arguments {
    #[arg(help = "Which task to run")]
    task_name: String,
    #[arg(long, help = "The entry directory or rask.yaml file")]
    entry: Option<String>,
}

pub fn execute (arguments: &Arguments) -> Result<(), String> {
    let Arguments { entry, task_name } = arguments;

    // Start the timer
    let start_time = Instant::now();

    // Resolve the entry path
    let entry_config_path: PathBuf = config::resolve_config_path(&entry.clone().unwrap_or(".".to_string()))?;

    // Discover all config paths
    let config_file_paths: Vec<PathBuf> = config::discover_config_paths(&entry_config_path)?;

    // Parse config file content
    let config_files: Vec<ConfigFile> = config::read_config_files(config_file_paths)?;

    // Parse config files
    let configs: Vec<Config> = config::parse_config_files(config_files)?;

    // Resolve dependencies based on the directory structure
    // (In the future this will be configurable based on a dependency config field)
    let config_structure: ConfigStructure = config::resolve_config_structure(&entry_config_path, configs)?;

    // Gather the tasks from the config
    let ordered_tasks: OrderedTasks = config::resolve_task_order(config_structure, task_name)?;

    // Run the commands, one by one
    // > In the future this is configurable on the rask level and maybe on the config file level
    // > Initially it fails the whole command if one task fails, but will also be configurable in the future
    let task_exit: TaskExit = run_task_structure(&ordered_tasks)?;

    let task_amount = ordered_tasks.len();
    let execution_time = start_time.elapsed().as_secs_f32();
    let formatted_execution_time = (execution_time * 100.0).round() / 100.0;

    match task_exit {
        TaskExit::SUCCESS => println!("{}", format!("Successfully executed {} tasks within {} seconds", task_amount, formatted_execution_time)),
        TaskExit::FAILURE => println!("{}", format!("Failed after executing {} tasks within {} seconds", task_amount, formatted_execution_time)),
    }

    Ok(())
}

fn run_task_structure(ordered_tasks: &OrderedTasks) -> Result<TaskExit, String> {
    let highest_order = find_highest_order(&ordered_tasks)?;

    for order in (0..=highest_order).rev() {
        match run_task_order(&ordered_tasks, order) {
            Ok(_) => {}
            Err(err) => {
                println!("{}", err);
                return Ok(TaskExit::FAILURE);
            }
        }
    }

    Ok(TaskExit::SUCCESS)
}

fn run_task_order(ordered_tasks: &OrderedTasks, order: u64) -> Result<(), String> {
    let mut tasks: Vec<&Task> = vec![];
    for ordered_task in ordered_tasks {
        let OrderedTask { task, order: task_order } = ordered_task;
        if *task_order == order {
            tasks.push(task);
        }
    }

    let mut task_threads: Vec<JoinHandle<bool>> = vec![];
    for task in tasks {
        let task_thread = execute_task(task)?;
        task_threads.push(task_thread);
    }

    for task_thread in task_threads {
        if let Ok(success) = task_thread.join() {
            if !success {
                return Err("Command execution failed.".to_string());
            }
        } else {
            return Err("Thread panicked.".to_string());
        }
    }

    Ok(())
}

fn execute_task(task: &Task) -> Result<JoinHandle<bool>, String> {
    let task_thread = thread::spawn({
        let Task { command, directory } = task.clone();
        println!("[COMMAND] {} @ {:?}", command, directory);
        move || {
            let status = Command::new("sh")
                .arg("-c")
                .arg(command)
                .current_dir(directory)
                .status()
                .expect("Failed to execute command");
            status.success()
        }
    });

    Ok(task_thread)
}

fn find_highest_order(ordered_tasks: &OrderedTasks) -> Result<u64, String> {
    let mut highest_order: u64 = 0u64;

    for ordered_task in ordered_tasks {
        if ordered_task.order > highest_order {
            highest_order = ordered_task.order;
        }
    }

    Ok(highest_order)
}
