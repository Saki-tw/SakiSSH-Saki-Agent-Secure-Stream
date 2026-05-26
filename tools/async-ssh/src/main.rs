use tokio::process::Command;
use std::process::Stdio;
use tokio::io::{AsyncBufReadExt, BufReader};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 3 { eprintln!("Usage: async-ssh <host> <command>"); return Ok(()); }
    
    let host = &args[1];
    let cmd = &args[2..].join(" ");

    let mut child = Command::new("ssh")
        .arg(host)
        .arg(cmd)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;

    let stdout = child.stdout.take().unwrap();
    let stderr = child.stderr.take().unwrap();

    let mut stdout_reader = BufReader::new(stdout).lines();
    let mut stderr_reader = BufReader::new(stderr).lines();

    loop {
        tokio::select! {
            line = stdout_reader.next_line() => {
                match line? {
                    Some(l) => println!("[STDOUT] {}", l),
                    None => break,
                }
            }
            line = stderr_reader.next_line() => {
                match line? {
                    Some(l) => eprintln!("[STDERR] {}", l),
                    None => {},
                }
            }
        }
    }

    child.wait().await?;
    Ok(())
}
