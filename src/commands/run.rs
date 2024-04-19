use std::fmt::Debug;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::thread;
use std::thread::JoinHandle;
use std::time::Instant;
use clap::Args;
use crate::utils::config;
use crate::utils::config::{Config, ConfigStructure, get_ordered_tasks, SortableTask, SortableTasks, Task, TaskExit};
use crate::utils::file::ConfigFile;

#[derive(Args, Debug)]
pub struct Arguments {
    #[arg(help = "Which task to run")]
    task_name: String,
    #[arg(long, help = "The entry directory or rask.yaml file")]
    entry: Option<String>,
    #[arg(long, help = "enable strict command matching, defaults to checking if a command starts with a key")]
    parallel: bool,
    #[arg(long, help = "enable strict command matching, defaults to checking if a command starts with a key")]
    strict: bool,
}

pub fn execute (arguments: &Arguments) -> Result<(), String> {
    let Arguments { entry, task_name, parallel, strict } = arguments;

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
    let sortable_tasks: SortableTasks = config::resolve_sortable_task(config_structure, task_name, strict)?;

    // Run the commands, one by one
    // > In the future this is configurable on the rask level and maybe on the config file level
    // > Initially it fails the whole command if one task fails, but will also be configurable in the future
    let task_exit: TaskExit = run_sortable_tasks(&sortable_tasks, parallel)?;

    let task_amount = sortable_tasks.len();
    let execution_time = start_time.elapsed().as_secs_f32();
    let formatted_execution_time = (execution_time * 100.0).round() / 100.0;

    match task_exit {
        TaskExit::SUCCESS => println!("{}", format!("Successfully executed {} tasks within {} seconds", task_amount, formatted_execution_time)),
        // TaskExit::FAILURE => println!("{}", format!("Failed after executing {} tasks within {} seconds", task_amount, formatted_execution_time)),
    }

    Ok(())
}

fn run_sortable_tasks(sortable_tasks: &SortableTasks, parallel: &bool) -> Result<TaskExit, String> {
    let highest_order = find_highest_order(&sortable_tasks)?;

    for order in (0..=highest_order).rev() {
        let ordered_tasks = get_ordered_tasks(sortable_tasks, order)?;

        let result = match parallel {
            true => run_parallel_ordered_tasks(&ordered_tasks),
            false => run_ordered_tasks(&ordered_tasks),
        };

        match result {
            Ok(_) => {}, // Silence is victory!~
            Err(err) => return Err(err)
        }
    }

    Ok(TaskExit::SUCCESS)
}

fn run_ordered_tasks (ordered_tasks: &SortableTasks) -> Result<(), String> {
    for sortable_task in ordered_tasks {
        let SortableTask{ task, .. } = sortable_task;
        match execute_task(task.clone()) {
            Ok(_) => {}
            Err(err) => return Err(format!("Command did not execute {:?}", err))
        }
    }

    Ok(())
}

fn run_parallel_ordered_tasks (ordered_tasks: &SortableTasks) -> Result<(), String> {
    let mut task_threads: Vec<JoinHandle<Result<(), String>>> = vec![];

    for sortable_task in ordered_tasks {
        let SortableTask{ task, .. } = sortable_task;
        let handle = execute_task_parallel(task.clone());
        task_threads.push(handle);
    }

    for task_thread in task_threads {
        match task_thread.join() {
            Ok(_) => {}
            Err(err) => return Err(format!("Command did not execute {:?}", err))
        }
    }

    Ok(())
}

// Function to execute a command string and wait for it to finish
fn execute_task(task: Task) -> Result<(), String> {
    let Task { command, directory } = task.clone();

    println!("[COMMAND] {} @ {:?}", command, directory);
    let mut binding = Command::new("sh");
    let command = binding
        .arg("-c")
        .arg(command)
        .current_dir(directory)
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit());

    let status = command.status().expect("Failed to execute command");
    if !status.success() {
        return Err("Command execution failed".to_string());
    }

    Ok(())
}

// Function to execute a command string without blocking
fn execute_task_parallel(task: Task) -> JoinHandle<Result<(), String> > {
    let handle = thread::spawn(move || {
        execute_task(task)
    });
    handle
}

fn find_highest_order(ordered_tasks: &SortableTasks) -> Result<u64, String> {
    let mut highest_order: u64 = 0u64;

    for ordered_task in ordered_tasks {
        if ordered_task.order > highest_order {
            highest_order = ordered_task.order;
        }
    }

    Ok(highest_order)
}
