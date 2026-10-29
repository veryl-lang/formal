use std::{collections::BTreeMap, error::Error, io, path::Path, process::Command};

pub mod checks;
pub mod locks;
pub mod nightly;
pub mod report;

pub type Result<T> = std::result::Result<T, Box<dyn Error>>;
pub type Environment = BTreeMap<String, String>;

pub fn error(message: impl Into<String>) -> Box<dyn Error> {
    io::Error::other(message.into()).into()
}

pub fn required<'a>(env: &'a Environment, key: &str) -> Result<&'a str> {
    env.get(key)
        .map(String::as_str)
        .ok_or_else(|| error(format!("missing environment variable {key}")))
}

pub trait Commands {
    fn run(&mut self, cwd: &Path, program: &str, args: &[&str]) -> Result<String>;
}

pub struct Process;

impl Commands for Process {
    fn run(&mut self, cwd: &Path, program: &str, args: &[&str]) -> Result<String> {
        let output = Command::new(program).args(args).current_dir(cwd).output()?;
        if !output.status.success() {
            return Err(error(format!(
                "{program} {args:?} failed ({:?}): {}",
                output.status.code(),
                String::from_utf8_lossy(&output.stderr)
            )));
        }
        Ok(String::from_utf8(output.stdout)?.trim().into())
    }
}
