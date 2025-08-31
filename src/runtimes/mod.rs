use crossterm::{
    event::{Event, KeyCode, KeyEventKind, read},
    terminal::{disable_raw_mode, enable_raw_mode},
};

pub mod debugger;
pub mod interpreter;

/// These are all the issues that can occur when running a subleq program.
/// Note that Breakpoints are non-canonical and specific to this assembler.
/// They occur during a jump to -2
#[derive(Debug)]
pub enum RuntimeError {
    AOutOfRange(usize),
    BOutOfRange(usize),
    COutOfRange(usize),
    Breakpoint(usize),
}

pub fn get_key() -> KeyCode {
    enable_raw_mode().unwrap();
    loop {
        if let Ok(Event::Key(event)) = read() {
            if event.kind == KeyEventKind::Press {
                disable_raw_mode().unwrap();

                return event.code;
            }
        }
    }
}
