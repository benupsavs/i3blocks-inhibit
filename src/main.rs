use std::process::{Command, Child, Stdio};
use std::io::{self, BufRead, Write};

const STR_DISABLED:   &str = "Z";
const STR_SLEEP_IDLE: &str = "<span foreground=\"red\">I</span>";
const STR_SLEEP:      &str = "<span foreground=\"yellow\">I</span>";

#[derive(PartialEq)]
enum InhibitState {
    Disabled,
    SleepIdle,
    Sleep,
}

fn main() {
    let stdin = io::stdin();
    let mut state = InhibitState::Disabled;
    let mut child_process: Option<(Child, std::process::ChildStdin)> = None;
    print_disabled();
    for line in stdin.lock().lines() {
        if line.is_err() {
            break;
        }
        let line = line.unwrap();
        if (line.trim() == "1" || line.trim() == "3") && state != InhibitState::Disabled {
            if (line.trim() == "1" && state == InhibitState::Sleep) ||
               (line.trim() == "3" && state == InhibitState::SleepIdle) {
                state = InhibitState::Disabled;
            } else {
                // Don't allow implicit canceling of one inhibit with another.
                continue;
            }
        } else if line.trim() == "1" {
            state = InhibitState::Sleep;
        } else if line.trim() == "3" {
            state = InhibitState::SleepIdle;
        } else {
            // Ignore any other input.
            continue;
        }
        if state != InhibitState::Disabled {
            let (child, child_stdin) = enable_inhibit(state == InhibitState::SleepIdle);
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
        if state != InhibitState::Disabled {
            if state == InhibitState::SleepIdle {
                print_enabled_sleep_idle();
            } else {
                print_enabled_sleep();
            }
        }
    }
}

fn print_disabled() {
    println!("{}", STR_DISABLED);
}

fn print_enabled_sleep() {
    println!("{}", STR_SLEEP);
}

fn print_enabled_sleep_idle() {
    println!("{}", STR_SLEEP_IDLE);
}

const WHAT_SLEEP: &str = "--what=sleep";
const WHAT_IDLE: &str = "--what=idle:sleep";

fn enable_inhibit(include_idle: bool) -> (Child, std::process::ChildStdin) {
    let mut child = Command::new("systemd-inhibit")
        .arg("--who=i3blocks-inhibit")
        .arg("--why=User disabled sleep")
        .arg(if include_idle { WHAT_IDLE } else { WHAT_SLEEP })
        .arg("cat")
        .stdin(Stdio::piped())
        .spawn()
        .expect("systemd-inhibit failed to start");
    let child_stdin = child.stdin.take().expect("Failed to open stdin");
    (child, child_stdin)
}
