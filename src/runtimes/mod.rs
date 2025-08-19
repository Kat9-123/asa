pub mod debugger;
pub mod interpreter;

#[derive(Debug)]
pub enum RuntimeError {
    COutOfRange(usize),
    AOutOfRange(usize),
    BOutOfRange(usize),
    Breakpoint(usize),
}
