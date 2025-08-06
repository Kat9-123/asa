use crate::interpreter::RuntimeError;
use crate::{
    interpreter::{self, IOOperation, InstructionHistoryItem},
    mem_view,
    tokens::{Info, Token},
};
use colored::Colorize;
use crossterm::execute;
use crossterm::terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode};
use crossterm::{
    ExecutableCommand,
    event::{Event, KeyCode, KeyEventKind, read},
    terminal::{self, enable_raw_mode},
};
use std::fs;
use std::io::{self, *};

pub fn revert_historic_instruction(
    mem: &mut Vec<u16>,
    inst: &InstructionHistoryItem,
    io_buffer: &mut String,
) -> usize {
    let b = mem[inst.pc + 1] as usize;
    if b < mem.len() {
        mem[b] = inst.original_value_at_b;
    }

    if let IOOperation::Char(c) = inst.io_operation {
        io_buffer.pop();
    }
    return inst.pc;
}

enum DataType {
    Char,
    Int,
    Hex,
}

fn address_to_string(addr: u16, mem: &Vec<u16>, data_type: DataType) -> String {
    match addr {
        0xFFFF => "IO".to_string(),
        0xFFFE => "Debug".to_string(),
        0xFFFD => "Perf".to_string(),
        x if (x as usize) >= mem.len() => "OOB".to_string(),

        _ => match data_type {
            DataType::Char => format!("{}", mem[addr as usize] as u16 as u8 as char),
            DataType::Int => format!("{}", mem[addr as usize] as i16),
            DataType::Hex => format!("{:X}", mem[addr as usize] as u16),
        },
    }
}

fn get_file_contents(path: &String) -> String {
    fs::read_to_string(path).expect("Should have been able to read the file")
}
fn display(info: &Info, pc: usize, mem: &Vec<u16>, current_error: &Option<RuntimeError>) {
    if info.file == "" {
        return;
    }
    let contents = get_file_contents(&info.file);
    let lines = contents.lines().collect::<Vec<&str>>();

    const UPPER_SIZE: i32 = 5;
    const LOWER_SIZE: i32 = 5;

    // There is probably a cleaner way to do this
    let desired_start_line = info.line_number - 1 - UPPER_SIZE;
    let desired_end_line = info.line_number - 1 + LOWER_SIZE + 1; //.min(lines.len() as i32 - 1); // 10

    let actual_start_line = desired_start_line.max(0);
    let actual_end_line = desired_end_line.min(lines.len() as i32 - 1);

    let start_line = (actual_start_line - (desired_end_line - actual_end_line)).max(0);
    let end_line =
        (actual_end_line + (actual_start_line - desired_start_line)).min(lines.len() as i32 - 1);

    for i in start_line..end_line {
        if i != info.line_number - 1 {
            println!(
                "{: >4} | {: <100}",
                format!("{}", i + 1).bright_cyan(),
                lines[i as usize]
            );
        } else {
            let line = if current_error.is_none() {
                format!("{}", lines[i as usize]).blue()
            } else {
                format!("{}", lines[i as usize]).red()
            };
            println!(
                "{: >4} {} {: <100}",
                format!("{}", i + 1).bright_cyan(),
                ">".blue(),
                line
            );
        }
    }
    println!("PC: {: <100X} ", pc);
    if pc < mem.len() {
        println!(
            " a: {: >4X}   mem[a]: {: >7}  {: >4}  {: >4 }  ",
            mem[pc],
            address_to_string(mem[pc], mem, DataType::Int),
            address_to_string(mem[pc], mem, DataType::Hex),
            address_to_string(mem[pc], mem, DataType::Char),
        );
    } else {
        println!("Out of Bounds");
    }

    if pc + 1 < mem.len() {
        println!(
            " b: {: >4X}   mem[b]: {: >7}  {: >4}  {: >4 }  ",
            mem[pc + 1],
            address_to_string(mem[pc + 1], mem, DataType::Int),
            address_to_string(mem[pc + 1], mem, DataType::Hex),
            address_to_string(mem[pc + 1], mem, DataType::Char),
        );
    } else {
        println!("Out of Bounds");
    }

    println!(
        " c: {: <100} ",
        address_to_string(pc as u16, mem, DataType::Hex)
    );
}

fn user_input() -> KeyCode {
    loop {
        match read().unwrap() {
            Event::Key(event) => {
                if event.kind == KeyEventKind::Press {
                    return event.code;
                }
            }
            _ => {}
        }
    }
}

pub fn run_with_debugger(mem: &mut Vec<u16>, tokens: &Vec<Token>, mut in_debugging_mode: bool) {
    debug(mem, tokens, in_debugging_mode, user_input);
}

fn debug<T: FnMut() -> KeyCode>(
    mem: &mut Vec<u16>,
    tokens: &Vec<Token>,
    mut in_debugging_mode: bool,
    mut input: T,
) {
    //execute!(io::stdout(), EnterAlternateScreen);
    let mut pc = 0;
    let mut instruction_history: Vec<InstructionHistoryItem> = Vec::new();
    let mut io_buffer: String = String::new();
    let mut current_depth: usize = 0;
    let mut prev_info: Option<Info> = None;
    let mut stdout = io::stdout();
    //enable_raw_mode();
    stdout.execute(terminal::Clear(terminal::ClearType::All));
    let stay_in_file = false;
    let mut mem_mode: bool = false;
    let mut current_error: Option<RuntimeError> = None;
    loop {
        let mut skip_interaction = false;

        if in_debugging_mode {
            stdout.execute(crossterm::cursor::MoveTo(0, 0));

            let origin_info = &tokens[pc].origin_info;
            let info = if origin_info.len() == 0 {
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
                &origin_info[current_depth]
            };

            /*
            if let Some(prev) = &prev_info {
                if info == prev {
                    skip_interaction = true;
                }
            } */

            prev_info = Some(info.clone());

            if !skip_interaction {
                println!("{}", instruction_history.len());

                // asm_instruction!(info, "depth Instruction");
                if !mem_mode {
                    display(info, pc, &mem, &current_error);
                    if let Some(e) = &current_error {}
                } else {
                    mem_view::draw_mem(mem, pc);
                }
                //mem_view::draw_mem(mem, pc);
                stdout.execute(terminal::Clear(terminal::ClearType::FromCursorDown));

                println!("{: <100}", io_buffer);
                // dbg!(&instruction_history);
                // Only works on windows??
                match input() {
                    KeyCode::Char(c) => match c {
                        'm' => {
                            stdout.execute(terminal::Clear(terminal::ClearType::All));
                            mem_mode = !mem_mode;
                            continue;
                        }
                        'h' => return,
                        ' ' => {}
                        _ => {}
                    },
                    KeyCode::Enter => continue,
                    KeyCode::Right => {}
                    KeyCode::Left => {
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
                    KeyCode::Down => {
                        current_depth += 1;
                        continue;
                    }
                    KeyCode::Up => {
                        if current_depth > 0 {
                            current_depth -= 1;
                        }
                        continue;
                    }
                    KeyCode::Esc => {
                        in_debugging_mode = false;
                    }
                    _ => {}
                }
            }
        }

        if current_error.is_none() {
            let result = interpreter::interpret_single(mem, pc);
            let result = match result {
                Ok(e) => e,
                Err(e) => {
                    current_error = Some(e);
                    in_debugging_mode = true;
                    continue;
                }
            };
            let (new_pc, io_operation, instruction_history_item) = result;
            pc = new_pc;
            let hist_item = Some(instruction_history_item.clone());
            match io_operation {
                IOOperation::Char(c) => {
                    io_buffer.push(c);
                    //   print!("{c}");
                    //      io::stdout().flush();
                }
                IOOperation::Debug(i) => {
                    let i = i as i16;
                    println!("{i}");
                    io::stdout().flush();
                }
                IOOperation::Halt => {
                    break;
                }
                IOOperation::BreakPoint => {
                    in_debugging_mode = true;
                }
                IOOperation::Perf => {
                    todo!()
                }
                IOOperation::None => {}
            }
            instruction_history.push(hist_item.unwrap());
        }
    }
    //execute!(io::stdout(), LeaveAlternateScreen);
}

#[cfg(test)]
mod tests {

    use crate::tokens::{self, TokenVariant, tokens_from_token_variant_vec};

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
                KeyCode::Char('h'),
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
