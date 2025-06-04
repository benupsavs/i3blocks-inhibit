use std::process::{Command, Child, Stdio};
use std::io::{self, BufRead, Write};

const STR_DISABLED: &str = "Z";
const STR_ENABLED:  &str = "<span foreground=\"red\">I</span>";

fn main() {
    let stdin = io::stdin();
    let mut enabled = false;
    let mut child_process: Option<(Child, std::process::ChildStdin)> = None;
    print_disabled();
    for line in stdin.lock().lines() {
        if line.is_err() {
            break;
        }
        let line = line.unwrap();
        if line.trim() != "1" {
            continue;
        }
        enabled = !enabled;
        if enabled {
            let (child, child_stdin) = run_dummy_command();
            child_process = Some((child, child_stdin));
        } else {
            if let Some((mut child, mut child_stdin)) = child_process.take() {
                // Write to child's stdin and close it to make 'cat' exit.
                // This also sends the disabled string back to i3blocks.
                let _ = writeln!(child_stdin, "{}", STR_DISABLED);
                let _ = child_stdin.flush();
                drop(child_stdin);
                let _ = child.wait();
            }
        }
        if enabled {
            print_enabled();
        }
    }
}

fn print_disabled() {
    println!("{}", STR_DISABLED);
}

fn print_enabled() {
    println!("{}", STR_ENABLED);
}

fn run_dummy_command() -> (Child, std::process::ChildStdin) {
    let mut child = Command::new("systemd-inhibit")
        .arg("--who=i3blocks-inhibit")
        .arg("--why=User disabled idle")
        .arg("cat")
        .stdin(Stdio::piped())
        .spawn()
        .expect("systemd-inhibit failed to start");
    let child_stdin = child.stdin.take().expect("Failed to open stdin");
    (child, child_stdin)
}
