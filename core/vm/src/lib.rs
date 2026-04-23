mod handler;
mod instruction;
mod memory;
mod registers;

use crate::instruction::{Instruction, InstructionError};
use crate::memory::{Memory, MemoryError};
use crate::registers::{RegisterError, Registers};
use isa::{AddressingMode, MEM_BYTES, OptSpec, REG_COUNT};
use logger::{LogTo, Logger, LoggerError};
use std::{io, num::ParseIntError};

#[derive(Debug, thiserror::Error)]
pub enum VMError {
    #[error("{0}")]
    Memory(#[from] MemoryError),
    #[error("{0}")]
    Register(#[from] RegisterError),
    #[error("{0}")]
    IO(#[from] io::Error),
    #[error("{0}")]
    ParseInt(#[from] ParseIntError),
    #[error("Operation {0} not implemented yet")]
    NoImplementation(String),
    #[error("{0}")]
    Instruction(#[from] InstructionError),
    #[error("Logger error: {0}")]
    Logger(#[from] LoggerError),
    #[error("Invalid binary")]
    InvalidBinary,
    #[error("Error converting Vec to slice")]
    VecToSlice,
    #[error("Invalid operand mode: {mode} with value: {value}")]
    InvalidOperandMode { mode: AddressingMode, value: u32 },
}

pub struct MyVM {
    pub registers: Registers<u8>,
    pub memory: Memory<u8>,
    pub debug: bool,
    pub opt_spec: OptSpec,
    pub logger: Logger,
}

#[derive(Debug, PartialEq, Clone)]
pub enum MemoryAccessType {
    Read,
    Write,
}

#[derive(Debug, Clone)]
pub struct MemoryAccess {
    pub addresses: Vec<u32>,
    pub value: u8,
    pub type_: MemoryAccessType,
}

#[derive(Debug, Clone)]
pub struct ExecutionStep {
    pub instruction_str: String,
    pub address: u32,
    pub changed_regs: Vec<String>,
    pub memory_access: Option<MemoryAccess>,
    pub is_halted: bool,
}

#[derive(Clone)]
pub struct VMState<'a> {
    pub registers: &'a Registers<u8>,
    pub memory: &'a Memory<u8>,
}

impl MyVM {
    pub fn new() -> Result<Self, VMError> {
        Ok(Self {
            opt_spec: OptSpec::clone(),
            memory: Memory::new(MEM_BYTES),
            registers: Registers::new(REG_COUNT, MEM_BYTES),
            logger: Logger::new(
                String::from("vm.txt"),
                String::from("/logs/"),
                LogTo::Console,
            )?,
            debug: false,
        })
    }

    fn execute(&mut self, instr: Instruction) -> Result<ExecutionStep, VMError> {
        let changes = match instr.get_operation_name().to_lowercase().as_str() {
            "halt" => Ok(self.halt()?),
            "in" => Ok(self.input(&instr)?),
            "out" => Ok(self.output(&instr)?),
            "out_16" => Ok(self.output_16()?),
            "out_char" => Ok(self.output_char(&instr)?),

            "mover" => Ok(self.mover(&instr)?),
            "movem" => Ok(self.movem(&instr)?),

            "add" => Ok(self.add(&instr)?),
            "sub" => Ok(self.sub(&instr)?),
            "mult" => Ok(self.mult(&instr)?),

            "adc" => Ok(self.adc(&instr)?),
            "sbc" => Ok(self.sbc(&instr)?),

            "mult_16" => Ok(self.mult_16(&instr)?),

            "jmp" => Ok(self.jmp(&instr)?),
            "jz" => Ok(self.jz(&instr)?),
            "jnz" => Ok(self.jnz(&instr)?),
            "push" => Ok(self.push(&instr)?),
            "pop" => Ok(self.pop(&instr)?),
            "call" => Ok(self.call(&instr)?),
            "ret" => Ok(self.ret()?),
            _ => Err(VMError::NoImplementation(
                instr.get_operation_name().to_string(),
            )),
        }?;

        Ok(ExecutionStep {
            instruction_str: format!("{:?}", instr),
            address: self.registers.pc,
            changed_regs: changes.registers,
            memory_access: changes.memory_access,
            is_halted: self.registers.eof == self.registers.pc,
        })
    }

    pub fn step(&mut self) -> Result<ExecutionStep, VMError> {
        let instruction = Instruction::new(&self.memory, &mut self.registers.pc, &self.opt_spec)?;

        if self.debug {
            self.logger.log(format!(
                "Executing instruction at PC {}: {}",
                self.registers.pc, instruction
            ))?;
        }
        self.execute(instruction)
    }

    pub fn run(&mut self) -> Result<(), VMError> {
        println!("Starting execution...");
        while self.registers.pc < self.memory.size() * 8 && self.registers.pc < self.registers.eof {
            self.step()?;
        }
        println!("End of Execution.");
        if self.debug {
            self.registers.print();
        }
        Ok(())
    }

    pub fn reset(&mut self) {
        self.registers.pc = 0;
        self.registers.reset();
        self.memory.reset();
    }

    pub fn get_state(&self) -> VMState<'_> {
        VMState {
            registers: &self.registers,
            memory: &self.memory,
        }
    }

    pub fn load_kernel(&mut self) -> Result<(), VMError> {
        self.reset();

        let mut kernel_binary = match std::fs::read("kernel.bin") {
            Ok(bytes) => bytes,
            Err(e) => {
                println!("Failed to read kernel.bin: {}", e);
                return Ok(());
            }
        };

        let eof = kernel_binary.split_off(kernel_binary.len() - 4);
        self.registers.eof = u32::from_be_bytes(eof.try_into().map_err(|_| VMError::VecToSlice)?);

        for byte in kernel_binary {
            self.memory.set(self.registers.pc, byte)?;
            self.registers.increment_pc();
        }

        self.registers.pc = 0;
        self.run()
    }

    pub fn start(&mut self) -> Result<(), VMError> {
        self.load_kernel()
    }
}
