# Advanced Subleq Assembler

This Subleq assembler uses a custom assembly-like language that will be directly converted to Subleq


## Features
* Interpreter with in an advanced debugger
* Macros
* Pointer syntax sugar
* In depth assembler feedback
* Optional typing system
* Fully fledged standard lib including functions and high level control flow constructs like If or While
* Scopes

## What is Subleq?
Subleq or SUBtract and jump if Less than or EQual to zero is an assembly language that had only one instruction, namely Subleq. The instruction has three operands: A, B, C
Where the value at memory address A is subtracted from the value at address B. If the resulting number is less than or equal to zero, a jump takes place to address C. Otherwise the next instruction is executed
Since there is only one instruction, the assembly does not contain opcodes.
So: 
SUBLEQ 1 2 3
would just be
1 2 3
A very basic subleq interpreter written in Python would look as follows
```Python
while True:
    a = mem[pc]
    b = mem[pc + 1]
    c = mem[pc + 2]

    result = mem[b] - mem[a]
    mem[b] = result
    if result <= 0:
        pc = c
    else:
        pc += 3
```
Most subleq implementations, this one included, also include the IO operations: INPUT, OUTPUT and HALT.
These can be achieved by respectively having A = -1, B = -1 and C = -1. INPUT and OUTPUT read or write singular ASCII characters

## Usage
```bash
asa my_subleq.sbl
```

## Debugger
