
use std::io::BufReader;
use std::io::BufRead;
use std::fs::File;
use std::env;
use std::collections::HashMap;

#[derive(Debug, Clone, Copy)]
enum Mnemonic {
    AND,
    OR,
    J,
    SLT,
    ADD,
    SUB,
    ADDI,
    BEQ,
    SW,
    LW,
}

#[derive(Debug, Clone)]
enum Operand {
    Reg(u32),
    Im(i32),
    Label(String),
}

#[derive(Clone)]
struct Instruction {
    label: Option<String>,
    mnemonic: Mnemonic,
    operands: Vec<Operand>,
}

enum InstructionType {
    R,
    I,
    J,
}

fn str2instr(s: &String) -> Result<Instruction, String> {
    let split_colon: Vec<_> = s.trim().split(':').collect();
    let (label, operation_str) = if split_colon.len() >= 2 {
        (Some(split_colon[0].trim().to_string()), split_colon[1])
    } else {
        (None, split_colon[0])
    };

    let operation_str = operation_str.replace(",", " ").replace("(", " ").replace(
        ")",
        "",
    );
    let mut split_space: Vec<_> = operation_str.split(" ").filter(|&s| s != "").collect();

    let operands_str: Vec<_> = split_space.split_off(1);
    let mnemonic = match split_space[0] {
        "and" => Mnemonic::AND,
        "or" => Mnemonic::OR,
        "j" => Mnemonic::J,
        "slt" => Mnemonic::SLT,
        "add" => Mnemonic::ADD,
        "sub" => Mnemonic::SUB,
        "addi" => Mnemonic::ADDI,
        "beq" => Mnemonic::BEQ,
        "sw" => Mnemonic::SW,
        "lw" => Mnemonic::LW,
        _ => {
            return Err(format!("undefined mnemonic:{}", split_space[0]));
        }
    };

    // println!("{:?} {:?}", label, mnemonic);

    let operands: Vec<Operand> = operands_str
        .iter()
        .map(|&s| {
            let cv: Vec<_> = s.chars().collect();
            match cv[0] {
                '$' => Operand::Reg(str2regidx(&s.to_string())),
                '0'...'9' | '-' => Operand::Im(s.parse::<i32>().unwrap()),
                _ => Operand::Label(s.to_string()),
            }
        })
        .collect();
    // println!("{:?}", operands);
    Ok(Instruction {
        label: label,
        mnemonic: mnemonic,
        operands: operands,
    })
}

fn str2regidx(s: &String) -> u32 {
    match s.as_ref() {
        "$0" => 0,
        "$at" => 1,
        "$gp" => 28,
        "$sp" => 29,
        "$fp" => 30,
        "$ra" => 31,
        _ => {
            let cv: Vec<_> = s.chars().collect();
            let prefix = cv[1];
            let n = cv[2].to_digit(10).expect("illegal register");
            match prefix {
                'v' => n + 2,
                'a' => n + 4,
                't' => n + 8,
                's' => if n < 8 { n + 16 } else { n + 24 },
                'k' => n + 26,
                _ => {
                    panic!("illegal register");
                }
            }
        }
    }
}

fn mnemonic2funct(mnemonic: Mnemonic) -> u32 {
    match mnemonic {
        Mnemonic::ADD => 32,
        Mnemonic::SUB => 34,
        Mnemonic::AND => 36,
        Mnemonic::OR => 37,
        Mnemonic::SLT => 42,
        _ => 0,
    }
}

fn mnemonictype(mnemonic: Mnemonic) -> InstructionType {
    match mnemonic {
        Mnemonic::ADD | Mnemonic::SUB | Mnemonic::AND | Mnemonic::OR | Mnemonic::SLT => {
            InstructionType::R
        }
        Mnemonic::ADDI | Mnemonic::BEQ | Mnemonic::LW | Mnemonic::SW => InstructionType::I,
        Mnemonic::J => InstructionType::J,
    }
}

fn mnemonic2op(mnemonic: Mnemonic) -> u32 {
    match mnemonic {
        Mnemonic::ADD | Mnemonic::SUB | Mnemonic::AND | Mnemonic::OR | Mnemonic::SLT => 0,
        Mnemonic::ADDI => 8,
        Mnemonic::LW => 35,
        Mnemonic::SW => 43,
        Mnemonic::BEQ => 4,
        Mnemonic::J => 2,
    }
}

fn instrs2bin(instrs: Vec<Instruction>) -> Vec<u32> {
    let mut labelmap = HashMap::new();
    for (i, instr) in instrs.iter().enumerate() {
        if let Some(ref label) = instr.label {
            labelmap.insert(label.clone(), i);
        }
    }

    let mut instrs_bin: Vec<u32> = Vec::new();
    for (i, instr) in instrs.iter().enumerate() {
        let bin = match mnemonictype(instr.mnemonic) {
            InstructionType::R => {
                let op = mnemonic2op(instr.mnemonic);
                let operands = &instr.operands;
                if operands.len() != 3 {
                    panic!("something wrong!");
                }
                let operands = match (&operands[1], &operands[2], &operands[0]) {
                    (&Operand::Reg(rs), &Operand::Reg(rt), &Operand::Reg(rd)) => {
                        rs << 21 | rt << 16 | rd << 11
                    }
                    _ => {
                        panic!("something wrong!");
                    }
                };
                let funct = mnemonic2funct(instr.mnemonic);
                op << 26 | operands | funct
            }
            InstructionType::I => {
                let op = mnemonic2op(instr.mnemonic);
                let operands = &instr.operands;
                if operands.len() != 3 {
                    panic!("something wrong!");
                }
                let operands = match (&operands[1], &operands[0], &operands[2]) {
                    (&Operand::Reg(rs), &Operand::Reg(rt), &Operand::Im(im)) => {
                        rs << 21 | rt << 16 | (im as u32 & ((1 << 16) - 1))
                    }
                    (&Operand::Im(im), &Operand::Reg(rt), &Operand::Reg(rs)) => {
                        rs << 21 | rt << 16 | (im as u32 & ((1 << 16) - 1))
                    }
                    (&Operand::Reg(rt), &Operand::Reg(rs), &Operand::Label(ref label)) => {
                        if let Some(adr) = labelmap.get(label) {
                            rs << 21 | rt << 16 | ((adr - 1 - i) as u32 & ((1 << 16) - 1))
                        } else {
                            panic!("something wrong!");
                        }
                    }
                    _ => {
                        panic!("something wrong!");
                    }
                };
                op << 26 | operands
            }
            InstructionType::J => {
                let op = mnemonic2op(instr.mnemonic);
                let operands = &instr.operands;
                if operands.len() != 1 {
                    panic!("something wrong!");
                }
                let operands = match &operands[0] {
                    &Operand::Label(ref label) => {
                        if let Some(&pos) = labelmap.get(label) {
                            pos as u32
                        } else {
                            panic!("something wrong!");
                        }
                    }
                    _ => {
                        panic!("something wrong!");
                    }
                };
                op << 26 | operands
            }
        };
        instrs_bin.push(bin);
    }
    instrs_bin
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        println!("{} [file]", args[0]);
        return;
    }
    let filename = args[1].clone();
    let file = match File::open(&filename) {
        Ok(file) => file,
        Err(e) => {
            panic!("{}: {}", filename, e);
        }
    };
    let mut buf_file = BufReader::new(file);

    let mut buffer = String::new();
    let mut program: Vec<Instruction> = Vec::new();
    loop {
        match buf_file.read_line(&mut buffer) {
            Ok(0) => break,
            Ok(_) => {
                program.push(str2instr(&buffer).expect("error:"));
                buffer.clear();
            }
            Err(e) => {
                println!("{}", e);
                break;
            }
        }
    }
    let instrs_bin = instrs2bin(program);
    for instr_bin in &instrs_bin {
        println!("{:08x}", instr_bin);
    }
    for _ in 0..(64 - instrs_bin.len()) {
        println!("{:08x}", 0 as u32);
    }
}
