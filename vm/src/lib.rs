mod handler;
mod instruction;
mod memory;
mod registers;

use crate::instruction::{Instruction, InstructionError};
use crate::memory::{Memory, MemoryError};
use crate::registers::{Registers, RegisterError};
use isa::OptSpec;
use logger::{LogTo, Logger, LoggerError};
use std::{num::ParseIntError, io};

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
    pub address: u32,
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
pub struct VMState {
    pub registers: Registers<u8>,
    pub memory: Memory<u8>,
}

impl MyVM {
    pub fn new() -> Result<Self, VMError> {
        Ok(Self {
            opt_spec: OptSpec::clone(),
            memory: Memory::new(256),
            registers: Registers::new(4, 256),
            logger: Logger::new(
                String::from("vm.txt"),
                String::from("/logs/"),
                LogTo::Console,
            )?,
            debug: false,
        })
    }

    fn execute(
        &mut self,
        instruction: Instruction,
    ) -> Result<ExecutionStep, VMError> {
        let operands = instruction.get_operands();

        let changes = match instruction.get_operation_name().to_lowercase().as_str() {
            "halt" => Ok(self.halt(operands)?),
            "in" => Ok(self.input(operands)?),
            "out" => Ok(self.output(operands)?),
            "out_16" => Ok(self.output_16(operands)?),
            "out_char" => Ok(self.output_char(operands)?),
            
            "mover" => Ok(self.mover(operands, false)?),
            "movem" => Ok(self.movem(operands)?),
            "movei" => Ok(self.mover(operands, true)?),
            
            "add" => Ok(self.add(operands, false)?),
            "sub" => Ok(self.sub(operands, false)?),
            "mult" => Ok(self.mult(operands, false)?),
            
            "addi" => Ok(self.add(operands, true)?),
            "subi" => Ok(self.sub(operands, true)?),
            "multi" => Ok(self.mult(operands, true)?),
            
            "adc" => Ok(self.adc(operands, false)?),
            "sbc" => Ok(self.sbc(operands, false)?),
            
            "adci" => Ok(self.adc(operands, true)?),
            "sbci" => Ok(self.sbc(operands, true)?),
            
            "mult_16" => Ok(self.mult_16(operands, false)?),
            "multi_16" => Ok(self.mult_16(operands, true)?),
            
            "jmp" => Ok(self.jmp(operands)?),
            "push" => Ok(self.push(operands)?),
            "pop" => Ok(self.pop(operands)?),
            "call" => Ok(self.call(operands)?),
            "ret" => Ok(self.ret(operands)?),
            _ => Err(VMError::NoImplementation(
                instruction.get_operation_name().to_string(),
            )),
        }?;

        Ok(ExecutionStep {
            instruction_str: format!("{:?}", instruction),
            address: self.registers.pc,
            changed_regs: changes.registers,
            memory_access: changes.memory_access,
            is_halted: self.registers.eof == self.registers.pc,
        })
    }

    pub fn step(&mut self) -> Result<ExecutionStep, VMError> {
        let instruction = Instruction::new(
            &self.memory,
            &mut self.registers.pc,
            &self.opt_spec,
        )?;

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

    pub fn get_state(&self) -> VMState {
        VMState {
            registers: self.registers.clone(),
            memory: self.memory.clone(),
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
