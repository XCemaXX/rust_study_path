use std::{
    collections::HashSet,
    env,
    fs::File,
    io::{BufRead, BufReader, Error, ErrorKind, Read, Write},
    process::{Command, Stdio},
};

fn get_command_output() -> Result<(), Box<dyn std::error::Error>> {
    let output = Command::new("rustc").arg("--version").output()?;
    if !output.status.success() {
        return Err("Cmd failed".to_string().into());
    }
    String::from_utf8(output.stdout)?
        .lines()
        .for_each(|l| println!("{l}"));
    Ok(())
}

fn run_piped_python() -> Result<(), Box<dyn std::error::Error>> {
    let mut child = Command::new("python3")
        .stdin(Stdio::piped())
        .stderr(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()?;
    child
        .stdin
        .as_mut()
        .ok_or("Child stdin has not been captured")?
        .write_all(b"import this; copyright(); credits(); exit()")?;

    let output = child.wait_with_output()?;

    if !output.status.success() {
        let err = String::from_utf8(output.stderr)?;
        return Err(format!("Cmd failed:\n {err}").to_string().into());
    }
    let output_str = String::from_utf8(output.stdout.clone())?;
    let words = output_str
        .split_whitespace()
        .map(|w| w.to_lowercase())
        .collect::<HashSet<_>>()
        .into_iter()
        .take(5)
        .collect::<Vec<_>>();
    println!("Unique first 5: {words:?}");
    Ok(())
}

fn run_pipes() -> Result<(), Box<dyn std::error::Error>> {
    let mut du = Command::new("du")
        .args(&["-ah", "."])
        .stdout(Stdio::piped())
        .spawn()?;
    //du.wait()?;
    let mut sort = Command::new("sort")
        .arg("-hr")
        .stdin(du.stdout.take().unwrap())
        .stdout(Stdio::piped())
        .spawn()?;
    //sort.wait()?;
    let mut head = Command::new("head")
        .args(&["-n", "5"])
        .stdin(sort.stdout.take().unwrap())
        .stdout(Stdio::piped())
        .spawn()?;
    let mut output = String::new();
    head.stdout.as_mut().unwrap().read_to_string(&mut output)?;
    println!("Top 10 biggest files and directories:\n{output}");
    Ok(())
}

fn pipes_to_file() -> Result<(), Box<dyn std::error::Error>> {
    let output = File::create("/tmp/pipes.txt")?;
    let errors = output.try_clone()?;

    Command::new("ls")
        .args(&[".", "oops"])
        .stdout(Stdio::from(output))
        .stderr(Stdio::from(errors))
        .spawn()?
        .wait_with_output()?;
    Ok(())
}

fn continious_reading() -> Result<(), Box<dyn std::error::Error>> {
    let stdout = Command::new("journalctl")
        .stdout(Stdio::piped())
        .spawn()?
        .stdout
        .ok_or_else(|| Error::new(ErrorKind::Other, "Could not capture output"))?;
    let reader = BufReader::new(stdout);

    reader
        .lines()
        .filter_map(|l| l.ok())
        .filter(|l| l.find("PCI").is_some())
        .take(5)
        .for_each(|l| println!("{l}"));
    Ok(())
}

fn read_env() -> Result<(), Box<dyn std::error::Error>> {
    let path = env::var("PATH").unwrap_or("/tmp".to_string());
    println!("Path: {path}");
    Ok(())
}

fn main() {
    get_command_output().unwrap();
    run_piped_python().unwrap();
    run_pipes().unwrap();
    pipes_to_file().unwrap();
    continious_reading().unwrap();
    read_env().unwrap();
}
