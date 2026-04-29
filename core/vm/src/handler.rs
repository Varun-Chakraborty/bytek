use isa::AddressingMode;

use crate::{MyVM, VMError, instruction::Instruction};
use std::io::{Write, stdin, stdout};

impl MyVM {
    pub fn halt(&mut self) -> Result<(), VMError> {
        self.registers.pc = self.registers.eof;
        Ok(())
    }

    pub fn input(&mut self, instr: &Instruction) -> Result<(), VMError> {
        let operands = instr.get_operands();
        let register = &operands[0];
        let mut input = String::new();
        print!("Enter value for registers {}: ", register.value);
        stdout().flush()?;
        stdin().read_line(&mut input)?;
        let input = input.trim().parse::<i8>()? as u8;
        self.registers.set_general(register.value, input)?;
        Ok(())
    }

    pub fn output(&self, instr: &Instruction) -> Result<(), VMError> {
        let operands = instr.get_operands();
        let register = &operands[0];
        let value = *self.registers.get_general(register.value)? as i8;
        println!("Output from registers {}: {value}", register.value);
        stdout().flush()?;
        Ok(())
    }

    pub fn output_16(&self) -> Result<(), VMError> {
        let high_byte = *self.registers.get_general(1)? as u16;
        let low_byte = *self.registers.get_general(0)? as u16;
        let value = ((high_byte << 8) | low_byte) as i16;
        println!("Combined output from registers 0 and 1: {value}");
        stdout().flush()?;
        Ok(())
    }

    pub fn output_char(&self, instr: &Instruction) -> Result<(), VMError> {
        let operands = instr.get_operands();
        let register = &operands[0];
        let value = *self.registers.get_general(register.value)? as i8;
        print!("{}", value as u8 as char);
        stdout().flush()?;
        Ok(())
    }

    pub fn mover(&mut self, instr: &Instruction) -> Result<(), VMError> {
        let operands = instr.get_operands();
        let register = &operands[0];
        let value = &operands[1];
        let value = match value.mode {
            AddressingMode::Immediate => value.value as u8,
            AddressingMode::DirectData => *self.memory.get(value.value)?,
            AddressingMode::Indirect => *self.memory.get(*self.memory.get(value.value)? as u32)?,
            AddressingMode::IndirectRegister => {
                let address = *self.registers.get_general(value.value)? as u32;
                if address >= self.registers.memory_size {
                    return Err(VMError::RuntimeError {
                        message: format!("Memory address {address} is out of range at instruction {instr}")
                    });
                }
                *self.memory.get(address)?
            }
            _ => {
                panic!("Invalid addressing mode");
            }
        };
        self.registers.set_general(register.value, value)?;
        self.registers.set_flag("zero", value == 0);
        self.registers.set_flag("sign", (value & (1 << 7)) != 0);
        self.registers.set_flag("carry", value & 0b1000_0000 != 0);
        self.registers.set_flag("overflow", false);
        Ok(())
    }

    pub fn movem(&mut self, instr: &Instruction) -> Result<(), VMError> {
        let operands = instr.get_operands();
        let register = &operands[0];
        let memory = &operands[1];
        let memory = match memory.mode {
            AddressingMode::DirectData => memory.value as u8,
            AddressingMode::Indirect => *self.memory.get(memory.value)?,
            AddressingMode::IndirectRegister => {
                let address = *self.registers.get_general(memory.value)?;
                if address as u32 >= self.registers.memory_size {
                    return Err(VMError::RuntimeError {
                        message: format!("Memory address {address} is out of range at instruction {instr}")
                    });
                }
                address
            }
            _ => {
                panic!("Invalid addressing mode");
            }
        } as u32;
        let value = *self.registers.get_general(register.value)?;
        self.memory.set(memory, value)?;
        self.registers.set_flag("zero", value == 0);
        self.registers.set_flag("sign", (value & (1 << 7)) != 0);
        self.registers.set_flag("carry", value & 0b1000_0000 != 0);
        self.registers.set_flag("overflow", false);
        Ok(())
    }

    pub fn add(&mut self, instr: &Instruction) -> Result<(), VMError> {
        let operands = instr.get_operands();
        let dest = &operands[0];
        let operand1 = &operands[1];
        let num1 = *self.registers.get_general(operand1.value)?;

        let operand2 = &operands[2];
        let num2 = match operand2.mode {
            AddressingMode::Immediate => operand2.value as u8,
            AddressingMode::Register => *self.registers.get_general(operand2.value)?,
            _ => {
                panic!("Invalid addressing mode");
            }
        };
        let sum_16 = num1 as u16 + num2 as u16;
        let sum_8 = sum_16 as i8;
        self.registers.set_flag("zero", sum_8 == 0);
        self.registers.set_flag("sign", sum_8 < 0);
        self.registers.set_flag("carry", sum_16 > 255);
        self.registers.set_flag(
            "overflow",
            ((num1 ^ sum_8 as u8) & (num2 ^ sum_8 as u8)) & (1 << 7) != 0,
        );
        self.registers.set_general(dest.value, sum_8 as u8)?;
        Ok(())
    }

    pub fn adc(&mut self, instr: &Instruction) -> Result<(), VMError> {
        let operands = instr.get_operands();
        let dest = &operands[0];
        let operand1 = &operands[1];
        let num1 = *self.registers.get_general(operand1.value)?;
        let operand2 = &operands[2];
        let num2 = match operand2.mode {
            AddressingMode::Immediate => operand2.value as u8,
            AddressingMode::Register => *self.registers.get_general(operand2.value)?,
            _ => {
                panic!("Invalid addressing mode");
            }
        };
        let sum_16 = num1 as u16 + num2 as u16 + self.registers.get_flag("carry") as u16;
        let sum_8 = sum_16 as i8;
        self.registers.set_flag("zero", sum_8 == 0);
        self.registers.set_flag("sign", sum_8 < 0);
        self.registers.set_flag("carry", sum_16 > 255);
        self.registers.set_flag(
            "overflow",
            ((num1 ^ sum_8 as u8) & (num2 ^ sum_8 as u8)) & (1 << 7) != 0,
        );
        self.registers.set_general(dest.value, sum_8 as u8)?;
        Ok(())
    }

    pub fn sub(&mut self, instr: &Instruction) -> Result<(), VMError> {
        let operands = instr.get_operands();
        let dest = &operands[0];
        let operand1 = &operands[1];
        let num1 = *self.registers.get_general(operand1.value)?;
        let operand2 = &operands[2];
        let num2 = match operand2.mode {
            AddressingMode::Immediate => operand2.value as u8,
            AddressingMode::Register => *self.registers.get_general(operand2.value)?,
            _ => {
                panic!("Invalid addressing mode");
            }
        };
        let diff_16 = num1 as u16 + 256 - num2 as u16;
        let diff_8 = diff_16 as i8;
        self.registers.set_flag("zero", diff_8 == 0);
        self.registers.set_flag("sign", diff_8 < 0);
        self.registers.set_flag("carry", num1 < num2);
        self.registers.set_flag(
            "overflow",
            ((num1 ^ num2) & (num1 ^ diff_8 as u8)) & (1 << 7) != 0,
        );
        self.registers.set_general(dest.value, diff_8 as u8)?;
        Ok(())
    }

    pub fn sbc(&mut self, instr: &Instruction) -> Result<(), VMError> {
        let operands = instr.get_operands();
        let dest = &operands[0];
        let operand1 = &operands[1];
        let num1 = *self.registers.get_general(operand1.value)?;
        let operand2 = &operands[2];
        let num2 = match operand2.mode {
            AddressingMode::Immediate => operand2.value as u8,
            AddressingMode::Register => *self.registers.get_general(operand2.value)?,
            _ => {
                panic!("Invalid addressing mode");
            }
        };
        let diff_16 = num1 as u16 + 256 - num2 as u16 - self.registers.get_flag("carry") as u16;
        let diff_8 = diff_16 as i8;
        self.registers.set_flag("zero", diff_8 == 0);
        self.registers.set_flag("sign", diff_8 < 0);
        self.registers.set_flag(
            "carry",
            num1 < (num2 + self.registers.get_flag("carry") as u8),
        );
        self.registers.set_flag(
            "overflow",
            ((num1 ^ num2) & (num1 ^ diff_8 as u8)) & (1 << 7) != 0,
        );
        self.registers.set_general(dest.value, diff_8 as u8)?;
        Ok(())
    }

    pub fn mult(&mut self, instr: &Instruction) -> Result<(), VMError> {
        let operands = instr.get_operands();
        let dest = &operands[0];
        let operand1 = &operands[1];
        let num1 = *self.registers.get_general(operand1.value)? as i8 as i16;
        let operand2 = &operands[2];
        let num2 = match operand2.mode {
            AddressingMode::Immediate => operand2.value as i8 as i16,
            AddressingMode::Register => *self.registers.get_general(operand2.value)? as i8 as i16,
            _ => {
                panic!("Invalid addressing mode");
            }
        };
        let product = num1 * num2;

        let lowbyte = product as u8;
        let highbyte = (product >> 8) as u8;

        self.registers.set_general(dest.value, lowbyte)?;
        self.registers.set_general(dest.value + 1, highbyte)?;

        self.registers.set_flag("zero", product == 0);
        self.registers.set_flag("sign", product < 0);
        self.registers.set_flag("overflow", highbyte != 0);
        self.registers.set_flag("carry", highbyte != 0);
        Ok(())
    }

    pub fn mult_16(&mut self, instr: &Instruction) -> Result<(), VMError> {
        let operands = instr.get_operands();
        let operand1 = &operands[0];
        let num1 = match operand1.mode {
            AddressingMode::Immediate => operand1.value as i8 as i16,
            AddressingMode::Register => *self.registers.get_general(operand1.value)? as i8 as i16,
            _ => {
                panic!("Invalid addressing mode");
            }
        };
        let num2 = (((*self.registers.get_general(1)? as i8 as u16) << 8)
            | *self.registers.get_general(0)? as u16) as i16;
        let product = num1 * num2;
        let highbyte = (product >> 8) as u8;
        let lowbyte = product as u8;

        self.registers.set_general(0, lowbyte)?;
        self.registers.set_general(1, highbyte)?;

        self.registers.set_flag("zero", product == 0);
        self.registers.set_flag("sign", product < 0);
        self.registers.set_flag("overflow", highbyte != 0);
        self.registers.set_flag("carry", highbyte != 0);
        Ok(())
    }

    pub fn push(&mut self, instr: &Instruction) -> Result<(), VMError> {
        let operands = instr.get_operands();
        let reg = &operands[0];
        let value = *self.registers.get_general(reg.value)?;
        self.registers.sp -= 1;
        self.memory.set(self.registers.sp, value)?;
        Ok(())
    }

    pub fn pop(&mut self, instr: &Instruction) -> Result<(), VMError> {
        let operands = instr.get_operands();
        let reg = &operands[0];
        let value = *self.memory.get(self.registers.sp)?;
        self.registers.sp += 1;
        self.registers.set_general(reg.value, value)?;
        Ok(())
    }

    pub fn call(&mut self, instr: &Instruction) -> Result<(), VMError> {
        let operands = instr.get_operands();
        self.registers.sp -= 1;
        self.memory
            .set(self.registers.sp, self.registers.pc as u8)?;
        self.registers.sp -= 1;
        self.memory
            .set(self.registers.sp, (self.registers.pc >> 8) as u8)?;
        // self.registers.sp -= 1;
        // self.memory
        //     .set(self.registers.sp, (self.registers.pc >> 16) as u8)?;
        // self.registers.sp -= 1;
        // self.memory
        //     .set(self.registers.sp, (self.registers.pc >> 24) as u8)?;
        let address = &operands[0];
        self.registers.pc = address.value;
        Ok(())
    }

    pub fn ret(&mut self) -> Result<(), VMError> {
        let mut location: u32 = 0;
        // location |= (*self.memory.get(self.registers.sp)? as u32) << 24;
        // self.registers.sp += 1;
        // location |= (*self.memory.get(self.registers.sp)? as u32) << 16;
        // self.registers.sp += 1;
        location |= (*self.memory.get(self.registers.sp)? as u32) << 8;
        self.registers.sp += 1;
        location |= *self.memory.get(self.registers.sp)? as u32;
        self.registers.sp += 1;
        self.registers.pc = location;
        Ok(())
    }

    pub fn jmp(&mut self, instr: &Instruction) -> Result<(), VMError> {
        let operands = instr.get_operands();
        let address = &operands[0];
        self.registers.pc = address.value;
        Ok(())
    }

    pub fn jz(&mut self, instr: &Instruction) -> Result<(), VMError> {
        let operands = instr.get_operands();
        let address = &operands[0];
        if self.registers.get_flag("zero") {
            self.registers.pc = address.value;
        }
        Ok(())
    }

    pub fn jnz(&mut self, instr: &Instruction) -> Result<(), VMError> {
        let operands = instr.get_operands();
        let address = &operands[0];
        if !self.registers.get_flag("zero") {
            self.registers.pc = address.value;
        }
        Ok(())
    }

    pub fn cmp(&mut self, instr: &Instruction) -> Result<(), VMError> {
    	let operands = instr.get_operands();
        let operand1 = &operands[0];
        let num1 = *self.registers.get_general(operand1.value)? as i8;
        let operand2 = &operands[1];
        let num2 = match operand2.mode {
            AddressingMode::Immediate => operand2.value as i8,
            AddressingMode::Register => *self.registers.get_general(operand2.value)? as i8,
            _ => {
                panic!("Invalid addressing mode");
            }
        };
        let (diff, carry) = (num1).overflowing_sub(num2);
        self.registers.set_flag("sign", diff < 0);
        self.registers.set_flag("zero", diff == 0);
        self.registers.set_flag("carry", carry);
        self.registers
            .set_flag("overflow", ((num1 ^ num2) & (num1 ^ diff)) & (1 << 7) != 0);
        Ok(())
    }

    // pub fn and(&mut self, instr: &Instruction) -> Result<Delta, VMError> {
    //     let operands = instr.get_operands();
    //     let dest = &operands[0];
    //     let num1 = *self.registers.get_general(operands[1])?;
    //     let num2 = *self.registers.get_general(operands[2])?;
    //     let product = num1 & num2;
    //     self.registers.set_flag("zero", product == 0);
    //     self.registers.set_flag("sign", (product & (1 << 7)) != 0);
    //     self.registers.set_flag("overflow", false);
    //     self.registers.set_flag("carry", false);
    //     self.registers.set_general(dest, product)?;
    //     Ok(Delta {
    //         registers: vec![],
    //         flags: vec![],
    //         memory_access: None,
    //     })
    // }

    // pub fn or(&mut self, instr: &Instruction) -> Result<Delta, VMError> {
    //	   let operands = instr.get_operands();
    //     let dest = &operands[0];
    //     let num1 = *self.registers.get_general(operands[1])?;
    //     let num2 = *self.registers.get_general(operands[2])?;
    //     let product = num1 | num2;
    //     self.registers.set_flag("zero", product == 0);
    //     self.registers.set_flag("sign", (product & (1 << 7)) != 0);
    //     self.registers.set_flag("overflow", false);
    //     self.registers.set_flag("carry", false);
    //     self.registers.set_general(dest, product)?;
    //     Ok(Delta {
    //         registers: vec![],
    //         flags: vec![],
    //         memory_access: None,
    //     })
    // }

    // pub fn xor(&mut self, instr: &Instruction) -> Result<Delta, VMError> {
    //	   let operands = instr.get_operands();
    //     let dest = &operands[0];
    //     let num1 = *self.registers.get_general(operands[1])?;
    //     let num2 = *self.registers.get_general(operands[2])?;
    //     let product = num1 ^ num2;
    //     self.registers.set_flag("zero", product == 0);
    //     self.registers.set_flag("sign", (product & (1 << 7)) != 0);
    //     self.registers.set_flag("overflow", false);
    //     self.registers.set_flag("carry", false);
    //     self.registers.set_general(dest, product)?;
    //     Ok(Delta {
    //         registers: vec![],
    //         flags: vec![],
    //         memory_access: None,
    //     })
    // }

    // pub fn not(&mut self, instr: &Instruction) -> Result<Delta, VMError> {
    //	   let operands = instr.get_operands();
    //     let dest = &operands[0];
    //     let num1 = *self.registers.get_general(operands[1])?;
    //     let product = !num1;
    //     self.registers.set_flag("zero", product == 0);
    //     self.registers.set_flag("sign", (product & (1 << 7)) != 0);
    //     self.registers.set_flag("overflow", false);
    //     self.registers.set_flag("carry", false);
    //     self.registers.set_general(dest, product)?;
    //     Ok(Delta {
    //         registers: vec![],
    //         flags: vec![],
    //         memory_access: None,
    //     })
    // }

    // pub fn shl(&mut self, instr: &Instruction) -> Result<Delta, VMError> {
    // 	   let operands = instr.get_operands();
    //     let reg = &operands[0];
    //     let value = *self.registers.get_general(reg)?;
    //     self.registers.set_flag("carry", (value & (1 << 7)) != 0);
    //     let shifted_value = value << 1;
    //     self.registers.set_flag("zero", shifted_value == 0);
    //     self.registers
    //         .set_flag("sign", (shifted_value & (1 << 7)) != 0);
    //     self.registers
    //         .set_flag("overflow", ((shifted_value ^ value) & (1 << 7)) != 0);
    //     self.registers.set_general(operands[0], shifted_value)?;
    //     Ok(Delta {
    //         registers: vec![],
    //         flags: vec![],
    //         memory_access: None,
    //     })
    // }

    // pub fn shr(&mut self, instr: &Instruction) -> Result<Delta, VMError> {
    //	   let operands = instr.get_operands();
    //     let reg = &operands[0];
    //     let value = *self.registers.get_general(reg)? as i8;
    //     self.registers.set_flag("carry", (value & 1) != 0);
    //     let value = value >> 1;
    //     self.registers.set_flag("zero", value == 0);
    //     self.registers.set_flag("sign", (value & (1 << 7)) != 0);
    //     self.registers.set_flag("overflow", false);
    //     self.registers.set_general(operands[0], value as u8)?;
    //     Ok(Delta {
    //         registers: vec![],
    //         flags: vec![],
    //         memory_access: None,
    //     })
    // }
}
