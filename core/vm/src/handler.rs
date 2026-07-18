use isa::AddressingMode;

use crate::{MyVM, VMError, device::Device, instruction::Instruction};

impl<D: Device> MyVM<D> {
    pub fn halt(&mut self) -> Result<(), VMError> {
        self.registers.pc = self.registers.eof;
        Ok(())
    }

    pub fn input(&mut self, instr: &Instruction) -> Result<(), VMError> {
        let operands = instr.get_operands();
        let register = &operands[0];
        let byte = self.device.read_byte()?;
        self.registers.set_general(register.value, byte)?;
        Ok(())
    }

    pub fn output(&mut self, instr: &Instruction) -> Result<(), VMError> {
        let operands = instr.get_operands();
        let register = &operands[0];
        let value = *self.registers.get_general(register.value)?;
        self.device.write_byte(value)?;
        Ok(())
    }

    pub fn mover(&mut self, instr: &Instruction) -> Result<(), VMError> {
        let operands = instr.get_operands();
        let register = &operands[0];
        let value = &operands[1];
        let value = match value.mode {
            AddressingMode::Immediate => value.value as u8,
            AddressingMode::Register => *self.registers.get_general(value.value)?,
            AddressingMode::DirectData => *self.memory.get(value.value)?,
            AddressingMode::Indirect => *self.memory.get(*self.memory.get(value.value)? as u32)?,
            AddressingMode::IndirectRegister => {
                let address = *self.registers.get_general(value.value)? as u32;
                if address >= self.registers.memory_size {
                    return Err(VMError::RuntimeError {
                        message: format!(
                            "Memory address {address} is out of range at instruction {instr}"
                        ),
                    });
                }
                *self.memory.get(address)?
            }
            _ => unreachable!("Invalid addressing mode"),
        };
        self.registers.set_general(register.value, value)?;
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
                        message: format!(
                            "Memory address {address} is out of range at instruction {instr}"
                        ),
                    });
                }
                address
            }
            _ => unreachable!("Invalid addressing mode"),
        } as u32;
        let value = *self.registers.get_general(register.value)?;
        self.memory.set(memory, value)?;
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
            _ => unreachable!("Invalid addressing mode"),
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
            _ => unreachable!("Invalid addressing mode"),
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
            _ => unreachable!("Invalid addressing mode"),
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
            _ => unreachable!("Invalid addressing mode"),
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

    pub fn push(&mut self, instr: &Instruction) -> Result<(), VMError> {
        let operands = instr.get_operands();
        let reg = &operands[0];
        let value = *self.registers.get_general(reg.value)?;
        self.memory.set(self.registers.sp, value)?;
        self.registers.sp -= 1;
        Ok(())
    }

    pub fn pop(&mut self, instr: &Instruction) -> Result<(), VMError> {
        let operands = instr.get_operands();
        let reg = &operands[0];
        self.registers.sp += 1;
        let value = *self.memory.get(self.registers.sp)?;
        self.registers.set_general(reg.value, value)?;
        Ok(())
    }

    pub fn call(&mut self, instr: &Instruction) -> Result<(), VMError> {
        let operands = instr.get_operands();
        self.memory
            .set(self.registers.sp, self.registers.pc as u8)?;
        self.registers.sp -= 1;
        self.memory
            .set(self.registers.sp, (self.registers.pc >> 8) as u8)?;
        self.registers.sp -= 1;
        self.memory
            .set(self.registers.sp, (self.registers.pc >> 16) as u8)?;
        self.registers.sp -= 1;
        // self.memory
        //     .set(self.registers.sp, (self.registers.pc >> 24) as u8)?;
        // self.registers.sp -= 1;
        let address = &operands[0];
        self.registers.pc = address.value;
        Ok(())
    }

    pub fn ret(&mut self) -> Result<(), VMError> {
        let mut location: u32 = 0;
        // self.registers.sp += 1;
        // location |= (*self.memory.get(self.registers.sp)? as u32) << 24;
        self.registers.sp += 1;
        location |= (*self.memory.get(self.registers.sp)? as u32) << 16;
        self.registers.sp += 1;
        location |= (*self.memory.get(self.registers.sp)? as u32) << 8;
        self.registers.sp += 1;
        location |= *self.memory.get(self.registers.sp)? as u32;
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
        let num1 = *self.registers.get_general(operand1.value)?;
        let operand2 = &operands[1];
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
        Ok(())
    }

    pub fn shl(&mut self, instr: &Instruction) -> Result<(), VMError> {
        let operands = instr.get_operands();
        let operand1 = &operands[0];
        let value = *self.registers.get_general(operand1.value)?;
        self.registers.set_flag("carry", (value & (1 << 7)) != 0);
        let value = value << 1;
        self.registers.set_flag("zero", value == 0);
        self.registers.set_flag("sign", (value & (1 << 7)) != 0);
        self.registers.set_flag("overflow", false);
        self.registers.set_general(operand1.value, value)?;
        Ok(())
    }

    pub fn shr(&mut self, instr: &Instruction) -> Result<(), VMError> {
        let operands = instr.get_operands();
        let operand1 = &operands[0];
        let value = *self.registers.get_general(operand1.value)?;
        self.registers.set_flag("carry", (value & 1) != 0);
        let value = value >> 1;
        self.registers.set_flag("zero", value == 0);
        self.registers.set_flag("sign", (value & (1 << 7)) != 0);
        self.registers.set_flag("overflow", false);
        self.registers.set_general(operand1.value, value)?;
        Ok(())
    }

    pub fn and(&mut self, instr: &Instruction) -> Result<(), VMError> {
        let operands = instr.get_operands();
        let dest = &operands[0];
        let operand1 = &operands[1];
        let num1 = *self.registers.get_general(operand1.value)?;
        let operand2 = &operands[2];
        let num2 = match operand2.mode {
            AddressingMode::Immediate => operand2.value as u8,
            AddressingMode::Register => *self.registers.get_general(operand2.value)?,
            _ => unreachable!("Invalid addressing mode"),
        };
        let product = num1 & num2;
        self.registers.set_flag("zero", product == 0);
        self.registers.set_flag("sign", (product & (1 << 7)) != 0);
        self.registers.set_flag("overflow", false);
        self.registers.set_flag("carry", false);
        self.registers.set_general(dest.value, product)?;
        Ok(())
    }

    pub fn or(&mut self, instr: &Instruction) -> Result<(), VMError> {
        let operands = instr.get_operands();
        let dest = &operands[0];
        let operand1 = &operands[1];
        let num1 = *self.registers.get_general(operand1.value)?;
        let operand2 = &operands[2];
        let num2 = match operand2.mode {
            AddressingMode::Immediate => operand2.value as u8,
            AddressingMode::Register => *self.registers.get_general(operand2.value)?,
            _ => unreachable!("Invalid addressing mode"),
        };
        let product = num1 | num2;
        self.registers.set_flag("zero", product == 0);
        self.registers.set_flag("sign", (product & (1 << 7)) != 0);
        self.registers.set_flag("overflow", false);
        self.registers.set_flag("carry", false);
        self.registers.set_general(dest.value, product)?;
        Ok(())
    }

    pub fn xor(&mut self, instr: &Instruction) -> Result<(), VMError> {
        let operands = instr.get_operands();
        let dest = &operands[0];
        let operand1 = &operands[1];
        let num1 = *self.registers.get_general(operand1.value)?;
        let operand2 = &operands[2];
        let num2 = match operand2.mode {
            AddressingMode::Immediate => operand2.value as u8,
            AddressingMode::Register => *self.registers.get_general(operand2.value)?,
            _ => unreachable!("Invalid addressing mode"),
        };
        let product = num1 ^ num2;
        self.registers.set_flag("zero", product == 0);
        self.registers.set_flag("sign", (product & (1 << 7)) != 0);
        self.registers.set_flag("overflow", false);
        self.registers.set_flag("carry", false);
        self.registers.set_general(dest.value, product)?;
        Ok(())
    }

    // pub fn mult(&mut self, instr: &Instruction) -> Result<(), VMError> {
    //     let operands = instr.get_operands();
    //     let dest = &operands[0];
    //     let operand1 = &operands[1];
    //     let num1 = *self.registers.get_general(operand1.value)? as i8 as i16;
    //     let operand2 = &operands[2];
    //     let num2 = match operand2.mode {
    //         AddressingMode::Immediate => operand2.value as i8 as i16,
    //         AddressingMode::Register => *self.registers.get_general(operand2.value)? as i8 as i16,
    //         _ => unreachable!("Invalid addressing mode")
    //     };
    //     let product = num1 * num2;

    //     let lowbyte = product as u8;
    //     let highbyte = (product >> 8) as u8;

    //     self.registers.set_general(dest.value, lowbyte)?;
    //     self.registers.set_general(dest.value + 1, highbyte)?;

    //     self.registers.set_flag("zero", product == 0);
    //     self.registers.set_flag("sign", product < 0);
    //     self.registers.set_flag("overflow", highbyte != 0);
    //     self.registers.set_flag("carry", highbyte != 0);
    //     Ok(())
    // }

    // pub fn div(&mut self, instr: &Instruction) -> Result<(), VMError> {
    //     let operands = instr.get_operands();
    //     let dest = &operands[0];
    //     let operand1 = &operands[1];
    //     let num1 = *self.registers.get_general(operand1.value)? as i16;
    //     let operand2 = &operands[2];
    //     let num2 = match operand2.mode {
    //         AddressingMode::Immediate => operand2.value as i8 as i16,
    //         AddressingMode::Register => *self.registers.get_general(operand2.value)? as i8 as i16,
    //         _ => unreachable!("Invalid addressing mode")
    //     };
    //     if num2 == 0 {
    //         return Err(VMError::RuntimeError {
    //             message: format!("Division by zero at instruction {instr}"),
    //         });
    //     }
    //     let quotient = num1 / num2;
    //     self.registers.set_general(dest.value, quotient as u8)?;

    //     self.registers.set_flag("zero", quotient == 0);
    //     self.registers.set_flag("sign", quotient < 0);
    //     self.registers.set_flag("overflow", quotient < 0);
    //     self.registers.set_flag("carry", quotient < 0);
    //     Ok(())
    // }

    // pub fn modulus(&mut self, instr: &Instruction) -> Result<(), VMError> {
    //     let operands = instr.get_operands();
    //     let dest = &operands[0];
    //     let operand1 = &operands[1];
    //     let num1 = *self.registers.get_general(operand1.value)? as i16;
    //     let operand2 = &operands[2];
    //     let num2 = match operand2.mode {
    //         AddressingMode::Immediate => operand2.value as i8 as i16,
    //         AddressingMode::Register => *self.registers.get_general(operand2.value)? as i8 as i16,
    //         _ => unreachable!("Invalid addressing mode")
    //     };
    //     if num2 == 0 {
    //         return Err(VMError::RuntimeError {
    //             message: format!("Modulo by zero at instruction {instr}"),
    //         });
    //     }
    //     let remainder = num1 % num2;
    //     self.registers.set_general(dest.value, remainder as u8)?;

    //     self.registers.set_flag("zero", remainder == 0);
    //     self.registers.set_flag("sign", remainder < 0);
    //     self.registers.set_flag("overflow", remainder < 0);
    //     self.registers.set_flag("carry", remainder < 0);
    //     Ok(())
    // }

    // pub fn not(&mut self, instr: &Instruction) -> Result<(), VMError> {
    //	   let operands = instr.get_operands();
    //     let dest = &operands[0];
    //     let num1 = *self.registers.get_general(operands[1])?;
    //     let product = !num1;
    //     self.registers.set_flag("zero", product == 0);
    //     self.registers.set_flag("sign", (product & (1 << 7)) != 0);
    //     self.registers.set_flag("overflow", false);
    //     self.registers.set_flag("carry", false);
    //     self.registers.set_general(dest, product)?;
    //     Ok(())
    // }
}
