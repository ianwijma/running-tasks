use std::process::Command;

pub fn run_task(task: &String) -> Result<(), String> {
    let mut command = Command::new(task);
    match command.spawn().unwrap().wait() {
        Ok(_) => Ok(()),
        Err(_) => Err(format!("Task failed: {}", task))
    }
}