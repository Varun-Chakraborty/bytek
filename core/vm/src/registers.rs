use std::fmt::Debug;

use num_traits::PrimInt;

#[derive(Debug, thiserror::Error)]
pub enum RegisterError {
    #[error("Invalid register {0}")]
    InvalidRegister(u32),
}

#[derive(Debug, Clone)]
pub struct GeneralRegisters<T> {
    pub count: u32,
    pub regs: Vec<T>,
}

impl<T: Copy + Default + PrimInt + Debug + std::fmt::Display> GeneralRegisters<T> {
    pub fn new(count: u32) -> Self {
        return Self {
            count,
            regs: vec![T::default(); count as usize],
        };
    }

    pub fn reset(&mut self) {
        self.regs = vec![T::default(); self.count as usize];
    }
}

#[derive(Debug, Copy, Clone)]
pub struct Flags {
    pub zero: bool,
    pub sign: bool,
    pub overflow: bool,
    pub carry: bool,
}

#[derive(Debug, Clone)]
pub struct Registers<T> {
    pub general_regs: GeneralRegisters<T>,
    pub flags: Flags,
    pub sp: u32,
    pub pc: u32,
    pub eof: u32,
    pub memory_size: u32,
}

impl<T: Copy + Default + PrimInt + Debug + std::fmt::Display> Registers<T> {
    pub fn new(count: u32, memory_size: u32) -> Self {
        return Self {
            general_regs: GeneralRegisters::new(count),
            flags: Flags {
                zero: false,
                sign: false,
                overflow: false,
                carry: false,
            },
            sp: memory_size - 1,
            pc: 0,
            eof: 0,
            memory_size,
        };
    }

    pub fn set_general(&mut self, register: u32, value: T) -> Result<(), RegisterError> {
        if register > self.general_regs.count - 1 {
            return Err(RegisterError::InvalidRegister(register));
        }
        // set value
        self.general_regs.regs[register as usize] = value;
        Ok(())
    }

    pub fn get_general(&self, register: u32) -> Result<&T, RegisterError> {
        if register > self.general_regs.count - 1 {
            return Err(RegisterError::InvalidRegister(register));
        }
        Ok(self
            .general_regs
            .regs
            .get(register as usize)
            .ok_or(RegisterError::InvalidRegister(register))?)
    }

    pub fn set_flag(&mut self, flag: &str, value: bool) {
        match flag {
            "zero" => self.flags.zero = value,
            "sign" => self.flags.sign = value,
            "overflow" => self.flags.overflow = value,
            "carry" => self.flags.carry = value,
            _ => panic!("Invalid flag: {}", flag),
        }
    }

    pub fn get_flag(&self, flag: &str) -> bool {
        match flag {
            "zero" => self.flags.zero,
            "sign" => self.flags.sign,
            "overflow" => self.flags.overflow,
            "carry" => self.flags.carry,
            _ => panic!("Invalid flag: {}", flag),
        }
    }

    pub fn increment_pc(&mut self) {
        self.pc += 1;
    }

    pub fn reset(&mut self) {
        self.general_regs.reset();
        self.flags = Flags {
            zero: false,
            sign: false,
            overflow: false,
            carry: false,
        };
        self.sp = self.memory_size - 1;
        self.pc = 0;
        self.eof = 0;
    }

    pub fn print(&self) {
        println!("Registers:");
        println!("General registers:");
        for (i, reg) in self.general_regs.regs.iter().enumerate() {
            println!("\tRegister {}: {}", i, reg);
        }
        println!("Flags:");
        println!("\tZero: {}", self.flags.zero);
        println!("\tSign: {}", self.flags.sign);
        println!("\tOverflow: {}", self.flags.overflow);
        println!("\tCarry: {}", self.flags.carry);
        println!("Stack pointer: {}", self.sp);
        println!("PC: {}", self.pc);
        println!("EOF: {}", self.eof);
    }
}
