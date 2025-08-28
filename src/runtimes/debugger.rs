use crate::lexer;
use crate::runtimes::RuntimeError;
use crate::symbols::{DEBUG_ADDR, IO_ADDR};
use crate::{
    mem_view,
    tokens::{Info, Token},
};
use colored::Colorize;
use crossterm::{
    ExecutableCommand,
    event::{Event, KeyCode, KeyEventKind, read},
    terminal::{self},
};
use std::fs;
use std::io::{self};
use std::num::Wrapping;
use std::path::PathBuf;

enum DataType {
    Char,
    Int,
    Hex,
}

#[derive(Debug, PartialEq, Eq, Clone)]
enum IOOperation {
    Char(char),
    Debug(u16),
    Halt,
    None,
}

#[derive(Debug, PartialEq, Eq, Clone)]
struct InstructionHistoryItem {
    pub pc: usize,
    pub original_value_at_b: u16,
    pub io_operation: IOOperation,
}

fn revert_historic_instruction(
    mem: &mut [u16],
    inst: &InstructionHistoryItem,
    io_buffer: &mut String,
) -> usize {
    let b = mem[inst.pc + 1] as usize;
    if b < mem.len() {
        mem[b] = inst.original_value_at_b;
    }

    if let IOOperation::Char(..) = inst.io_operation {
        io_buffer.pop();
    }
    inst.pc
}

fn val_to_string(val: u16, data_type: DataType) -> String {
    match data_type {
        DataType::Char => {
            let c = val as u8 as char;
            let length = val.to_string().chars().count(); // Doesn't actually work in some cases
            if length > 1 {
                format!("'{c}'")
            } else {
                " ''".to_owned()
            }
        }
        DataType::Int => format!("{}", val as i16),
        DataType::Hex => format!("{val:X}"),
    }
}
fn address_to_string(addr: u16, mem: &[u16], data_type: DataType) -> String {
    match addr as usize {
        IO_ADDR => "IO".to_string(),
        DEBUG_ADDR => "Debug".to_string(),
        x if x >= mem.len() => "OOB".to_string(),

        _ => val_to_string(mem[addr as usize], data_type),
    }
}

fn get_file_contents(path: &PathBuf) -> String {
    fs::read_to_string(path)
        .unwrap_or_else(|_| crate::error!("Debugger couldn't read file: {}", path.display()))
}

/// Display file preview and instruction that is being ran
fn display(
    info: &Info,
    pc: usize,
    new_pc: usize,
    result: u16,
    mem: &[u16],
    current_error: &Option<RuntimeError>,
) {
    let file = lexer::FILES.with_borrow(|files| {
        if !files.is_empty() {
            // Cant just use .get() because the internal value has to be cloned.
            Some(files[info.file].clone())
        } else {
            None
        }
    });

    let file = match file {
        None => return,
        Some(f) => f,
    };
    println!(
        "[ESC] exit debug mode       [UP/k] up  [DOWN/j] down  [LEFT/h] previous instruction
[RIGHT/l] next instruction  [x] halt   [m] toggle memory mode\n"
    );
    println!("{}:{: <100} ", file.display(), info.line_number);
    let contents = get_file_contents(&file);

    let lines = contents.lines().collect::<Vec<&str>>();

    const UPPER_SIZE: i32 = 15;
    const LOWER_SIZE: i32 = 15;

    // There is probably a cleaner way to do this
    let desired_start_line = info.line_number - 1 - UPPER_SIZE;
    let desired_end_line = info.line_number - 1 + LOWER_SIZE + 1;

    let actual_start_line = desired_start_line.max(0);
    let actual_end_line = desired_end_line.min(lines.len() as i32 - 1);

    let start_line = (actual_start_line - (desired_end_line - actual_end_line)).max(0);
    let end_line =
        (actual_end_line + (actual_start_line - desired_start_line)).min(lines.len() as i32 - 1);

    // File preview
    for i in start_line..end_line {
        if i != info.line_number - 1 {
            println!(
                "{: >4} | {: <100}",
                format!("{}", i + 1).bright_cyan(),
                lines[i as usize]
            );
        } else {
            let line = if current_error.is_none() {
                lines[i as usize].to_string().blue()
            } else {
                lines[i as usize].to_string().red()
            };
            println!(
                "{: >4} {} {: <100}",
                format!("{}", i + 1).bright_cyan(),
                ">".blue(),
                line
            );
        }
    }

    // Instruction preview
    if pc + 2 >= mem.len() {
        println!("Out of bounds");
        return;
    }

    println!("PC: {pc: <100X} ");
    println!("a: {: >4X}  b: {: >4X}", mem[pc], mem[pc + 1]);

    println!(
        " {: >6} - {: >6} = {: >6} ",
        address_to_string(mem[pc + 1], mem, DataType::Int),
        address_to_string(mem[pc], mem, DataType::Int),
        val_to_string(result, DataType::Int)
    );
    println!(
        " {: >6} - {: >6} = {: >6} ",
        address_to_string(mem[pc + 1], mem, DataType::Hex),
        address_to_string(mem[pc], mem, DataType::Hex),
        val_to_string(result, DataType::Hex)
    );
    println!(
        " {: >6} - {: >6} = {: >6} ",
        address_to_string(mem[pc + 1], mem, DataType::Char),
        address_to_string(mem[pc], mem, DataType::Char),
        val_to_string(result, DataType::Char)
    );
    println!("c: {: <100} ", val_to_string(new_pc as u16, DataType::Hex));
}

pub fn get_key() -> KeyCode {
    loop {
        if let Ok(Event::Key(event)) = read() {
            if event.kind == KeyEventKind::Press {
                return event.code;
            }
        }
    }
}

pub fn run_with_debugger(mem: &mut [u16], tokens: &[Token]) {
    debug(mem, tokens, false, get_key);
}

fn debug<T: FnMut() -> KeyCode>(
    mem: &mut [u16],
    tokens: &[Token],
    mut in_debugging_mode: bool,
    mut input: T,
) {
    let mut pc = 0;
    let mut instruction_history: Vec<InstructionHistoryItem> = Vec::new();
    let mut io_buffer: String = String::new();
    let mut current_depth: usize = 0;
    let mut stdout = io::stdout();
    stdout
        .execute(terminal::Clear(terminal::ClearType::All))
        .unwrap();
    let stay_in_file = false;
    let mut mem_mode: bool = false;
    let mut current_error: Option<RuntimeError> = None;
    loop {
        if pc + 2 >= mem.len() {
            current_error = Some(RuntimeError::COutOfRange(pc));
        }

        let a = mem[pc] as usize;
        let b = mem[pc + 1] as usize;
        let c = mem[pc + 2] as usize;

        let mut original_value_at_b = 0;

        let mut result: u16 = 0;
        let mut io: IOOperation = IOOperation::None;

        match (a, b) {
            (a, _) if a >= mem.len() && a != IO_ADDR => {
                current_error = Some(RuntimeError::AOutOfRange(pc));
            }
            (_, b) if b >= mem.len() && b != IO_ADDR && b != DEBUG_ADDR => {
                current_error = Some(RuntimeError::BOutOfRange(pc));
            }
            (_, IO_ADDR) => {
                result = mem[a];
                io = IOOperation::Char(result as u8 as char);
            }
            (_, DEBUG_ADDR) => {
                result = mem[a];
                io = IOOperation::Debug(result);
            }
            (IO_ADDR, _) => {
                println!("Input: ");
                let c = match get_key() {
                    KeyCode::Char(x) => x,
                    _ => '\0',
                };
                result = c as u16;
            }
            (_, _) => {
                original_value_at_b = mem[b];
                result = (Wrapping(mem[b]) - (Wrapping(mem[a]))).0;
            }
        }

        let new_pc = if result as i16 <= 0 {
            match c {
                IO_ADDR => {
                    io = IOOperation::Halt;
                    c
                }
                DEBUG_ADDR => {
                    current_error = Some(RuntimeError::Breakpoint(pc));
                    pc + 3
                }
                _ => c,
            }
        } else {
            pc + 3
        };

        if current_error.is_some() {
            in_debugging_mode = true;
        }

        if in_debugging_mode {
            stdout.execute(crossterm::cursor::MoveTo(0, 0)).unwrap();

            let origin_info = &tokens[pc].origin_info;
            let info = if origin_info.is_empty() {
                &tokens[pc].info
            } else {
                //&origin_info[origin_info.len() - 1].1 };
                let file_name = &origin_info[0].file; // Suboptimal

                let mut deepest_in_file_depth = 999_999;
                if stay_in_file {
                    for (i, x) in origin_info.iter().enumerate() {
                        if x.file == *file_name {
                            deepest_in_file_depth = i;
                        }
                    }
                }

                current_depth = current_depth
                    .min(origin_info.len() - 1)
                    .min(deepest_in_file_depth);

                if current_depth == origin_info.len() - 1 {
                    &tokens[pc].info
                } else {
                    &origin_info[current_depth]
                }
            };

            /*
            if let Some(prev) = &prev_info {
                if info == prev {
                    skip_interaction = true;
                }
            } */

            // asm_instruction!(info, "depth Instruction");
            if !mem_mode {
                display(info, pc, new_pc, result, mem, &current_error);
            //                if let Some(..) = &current_error {}
            } else {
                mem_view::draw_mem(mem, pc);
            }
            //mem_view::draw_mem(mem, pc);
            stdout
                .execute(terminal::Clear(terminal::ClearType::FromCursorDown))
                .unwrap();

            println!("{io_buffer: <100}");

            match input() {
                KeyCode::Char('m') => {
                    stdout
                        .execute(terminal::Clear(terminal::ClearType::All))
                        .unwrap();
                    mem_mode = !mem_mode;
                    continue;
                }
                KeyCode::Char('x') => return,
                KeyCode::Right => {}
                KeyCode::Left | KeyCode::Char('h') => {
                    current_error = None;
                    let instr = instruction_history.pop();
                    match instr {
                        Some(x) => {
                            pc = revert_historic_instruction(mem, &x, &mut io_buffer);
                        }
                        None => continue,
                    }
                    continue;
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    current_depth += 1;
                    continue;
                }
                KeyCode::Up | KeyCode::Char('k') => {
                    current_depth = current_depth.saturating_sub(1);
                    continue;
                }
                KeyCode::Esc => {
                    in_debugging_mode = false;
                }
                _ => {}
            }
        }
        if io == IOOperation::Halt {
            break;
        }

        if current_error.is_none() || matches!(current_error, Some(RuntimeError::Breakpoint(..))) {
            instruction_history.push(InstructionHistoryItem {
                pc,
                original_value_at_b,
                io_operation: io,
            });
            pc = new_pc;
            if b < mem.len() {
                mem[b] = result;
            }
        }
        current_error = None;
    }
    //execute!(io::stdout(), LeaveAlternateScreen);
}

#[cfg(test)]
mod tests {

    use crate::tokens::{TokenVariant, tokens_from_token_variant_vec};

    use super::*;

    #[test]
    fn test_debugger() {
        fn simulate_input() -> impl FnMut() -> KeyCode {
            let keys = vec![
                KeyCode::Right,
                KeyCode::Right,
                KeyCode::Right,
                KeyCode::Left,
                KeyCode::Left,
                KeyCode::Right,
                KeyCode::Left,
                KeyCode::Char('o'),
                KeyCode::Right,
                KeyCode::Right,
                KeyCode::Right,
                KeyCode::Right,
                KeyCode::Char('m'),
                KeyCode::Left,
                KeyCode::Right,
                KeyCode::Left,
                KeyCode::Right,
                KeyCode::Char('m'),
                KeyCode::Left,
                KeyCode::Left,
                KeyCode::Left,
                KeyCode::Right,
                KeyCode::Right,
                KeyCode::Right,
                KeyCode::Right,
                KeyCode::Left,
                KeyCode::Left,
                KeyCode::Left,
                KeyCode::Left,
                KeyCode::Char('x'),
            ];
            let mut iter = keys.into_iter();

            move || iter.next().unwrap()
        }
        let mut mem: Vec<u16> = vec![14, 12, 3, 14, 13, 6, 13, 14, 9, 12, 12, 0, 0, 1, 0];
        let expected: Vec<u16> = vec![14, 12, 3, 14, 13, 6, 13, 14, 9, 12, 12, 0, 0, 1, 0xFFFF];
        let tokens = &tokens_from_token_variant_vec(vec![
            (0, TokenVariant::DecLiteral { value: 14 }),
            (0, TokenVariant::DecLiteral { value: 12 }),
            (0, TokenVariant::DecLiteral { value: 3 }),
            (0, TokenVariant::DecLiteral { value: 14 }),
            (0, TokenVariant::DecLiteral { value: 13 }),
            (0, TokenVariant::DecLiteral { value: 6 }),
            (0, TokenVariant::DecLiteral { value: 13 }),
            (0, TokenVariant::DecLiteral { value: 14 }),
            (0, TokenVariant::DecLiteral { value: 9 }),
            (0, TokenVariant::DecLiteral { value: 12 }),
            (0, TokenVariant::DecLiteral { value: 12 }),
            (0, TokenVariant::DecLiteral { value: 0 }),
            (0, TokenVariant::DecLiteral { value: 0 }),
            (0, TokenVariant::DecLiteral { value: 1 }),
            (0, TokenVariant::DecLiteral { value: 0 }),
        ]);

        debug(&mut mem, tokens, true, simulate_input());
        assert_eq!(mem, expected);
    }
}
