use crate::{MemoryAccess, MemoryAccessType, MyVM, VMError};
use std::io::{Write, stdin, stdout};

pub struct Delta {
    pub registers: Vec<String>,
    pub flags: Vec<String>,
    pub memory_access: Option<MemoryAccess>,
}

impl MyVM {
    pub fn halt(&mut self, _: &[u32]) -> Result<Delta, VMError> {
        self.registers.pc = self.registers.eof;
        Ok(Delta {
            registers: vec!["PC".to_string(), "EOF".to_string()],
            flags: vec![],
            memory_access: None,
        })
    }

    pub fn input(&mut self, operands: &[u32]) -> Result<Delta, VMError> {
        let registers = operands[0];
        let mut input = String::new();
        print!("Enter value for registers {registers}: ");
        stdout().flush()?;
        stdin().read_line(&mut input)?;
        let input = input.trim().parse::<i8>()? as u8;
        self.registers.set_general(registers, input)?;
        Ok(Delta {
            registers: vec![format!("R{registers}")],
            flags: vec![],
            memory_access: None,
        })
    }

    pub fn output(&self, operands: &[u32]) -> Result<Delta, VMError> {
        let registers = operands[0];
        let value = *self.registers.get_general(registers)? as i8;
        println!("Output from registers {registers}: {value}");
        stdout().flush()?;
        Ok(Delta {
            registers: vec![format!("R{registers}")],
            flags: vec![],
            memory_access: None,
        })
    }

    pub fn output_16(&self, _: &[u32]) -> Result<Delta, VMError> {
        let high_byte = *self.registers.get_general(1)? as u16;
        let low_byte = *self.registers.get_general(0)? as u16;
        let value = ((high_byte << 8) | low_byte) as i16;
        println!("Combined output from registers 0 and 1: {value}");
        stdout().flush()?;
        Ok(Delta {
            registers: vec![format!("R0"), format!("R1")],
            flags: vec![],
            memory_access: None,
        })
    }

    pub fn output_char(&self, operands: &[u32]) -> Result<Delta, VMError> {
        let registers = operands[0];
        let value = *self.registers.get_general(registers)? as i8;
        print!("{}", value as u8 as char);
        stdout().flush()?;
        Ok(Delta {
            registers: vec![format!("R{registers}")],
            flags: vec![],
            memory_access: None,
        })
    }

    pub fn mover(&mut self, operands: &[u32], immediate: bool) -> Result<Delta, VMError> {
        let registers = operands[0];
        let value = if immediate {
            operands[1] as u8
        } else {
            *self.memory.get(operands[1])?
        };
        self.registers.set_general(registers, value)?;
        Ok(Delta {
            registers: vec![format!("R{registers}")],
            flags: vec![],
            memory_access: if immediate {
                None
            } else {
                Some(MemoryAccess {
                    address: operands[1],
                    value: value,
                    type_: MemoryAccessType::Read,
                })
            },
        })
    }

    pub fn movem(&mut self, operands: &[u32]) -> Result<Delta, VMError> {
        let registers = operands[0];
        let memory = operands[1];
        let value = *self.registers.get_general(registers)?;
        self.memory.set(memory, value)?;
        Ok(Delta {
            registers: vec![],
            flags: vec![],
            memory_access: Some(MemoryAccess {
                address: memory,
                value: value,
                type_: MemoryAccessType::Write,
            }),
        })
    }

    pub fn add(&mut self, operands: &[u32], immediate: bool) -> Result<Delta, VMError> {
        let dest = operands[0];
        let reg1 = operands[1];
        let reg2 = if immediate { None } else { Some(operands[2]) };

        let num1 = *self.registers.get_general(reg1)?;
        let num2 = if let Some(reg2) = reg2 {
            *self.registers.get_general(reg2)?
        } else {
            operands[2] as u8
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
        self.registers.set_general(dest, sum_8 as u8)?;
        Ok(Delta {
            registers: vec![format!("R{dest}")],
            flags: vec![],
            memory_access: None,
        })
    }

    pub fn adc(&mut self, operands: &[u32], immediate: bool) -> Result<Delta, VMError> {
        let dest = operands[0];
        let num1 = *self.registers.get_general(operands[1])?;
        let num2 = if immediate {
            operands[2] as u8
        } else {
            *self.registers.get_general(operands[2])?
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
        self.registers.set_general(dest, sum_8 as u8)?;
        Ok(Delta {
            registers: vec![],
            flags: vec![],
            memory_access: None,
        })
    }

    pub fn sub(&mut self, operands: &[u32], immediate: bool) -> Result<Delta, VMError> {
        let dest = operands[0];
        let num1 = *self.registers.get_general(operands[1])?;
        let num2 = if immediate {
            operands[2] as u8
        } else {
            *self.registers.get_general(operands[2])?
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
        self.registers.set_general(dest, diff_8 as u8)?;
        Ok(Delta {
            registers: vec![],
            flags: vec![],
            memory_access: None,
        })
    }

    pub fn sbc(&mut self, operands: &[u32], immediate: bool) -> Result<Delta, VMError> {
        let dest = operands[0];
        let num1 = *self.registers.get_general(operands[1])?;
        let num2 = if immediate {
            operands[2] as u8
        } else {
            *self.registers.get_general(operands[2])?
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
        self.registers.set_general(dest, diff_8 as u8)?;
        Ok(Delta {
            registers: vec![],
            flags: vec![],
            memory_access: None,
        })
    }

    pub fn mult(&mut self, operands: &[u32], immediate: bool) -> Result<Delta, VMError> {
        let dest = operands[0];
        let num1 = *self.registers.get_general(operands[1])? as i8 as i16;
        let num2 = if immediate {
            operands[2] as i8 as i16
        } else {
            *self.registers.get_general(operands[2])? as i8 as i16
        };
        let product = num1 * num2;

        let lowbyte = product as u8;
        let highbyte = (product >> 8) as u8;

        self.registers.set_general(dest, lowbyte)?;
        self.registers.set_general(dest + 1, highbyte)?;

        self.registers.set_flag("zero", product == 0);
        self.registers.set_flag("sign", product < 0);
        self.registers.set_flag("overflow", highbyte != 0);
        self.registers.set_flag("carry", highbyte != 0);
        Ok(Delta {
            registers: vec![],
            flags: vec![],
            memory_access: None,
        })
    }

    pub fn mult_16(&mut self, operands: &[u32], immediate: bool) -> Result<Delta, VMError> {
        let num1 = if immediate {
            operands[0] as i8 as i16
        } else {
            *self.registers.get_general(operands[0])? as i8 as i16
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
        Ok(Delta {
            registers: vec![],
            flags: vec![],
            memory_access: None,
        })
    }

    pub fn cmp(&mut self, operands: &[u32], immediate: bool) -> Result<Delta, VMError> {
        let num1 = *self.registers.get_general(operands[0])? as i8;
        let num2 = if immediate {
            operands[1] as i8
        } else {
            *self.registers.get_general(operands[1])? as i8
        };
        let (diff, carry) = num1.overflowing_sub(num2);
        self.registers.set_flag("sign", diff < 0);
        self.registers.set_flag("zero", diff == 0);
        self.registers.set_flag("carry", carry);
        self.registers
            .set_flag("overflow", ((num1 ^ num2) & (num1 ^ diff)) & (1 << 7) != 0);
        Ok(Delta {
            registers: vec![],
            flags: vec![],
            memory_access: None,
        })
    }

    pub fn and(&mut self, operands: &[u32]) -> Result<Delta, VMError> {
        let dest = operands[0];
        let num1 = *self.registers.get_general(operands[1])?;
        let num2 = *self.registers.get_general(operands[2])?;
        let product = num1 & num2;
        self.registers.set_flag("zero", product == 0);
        self.registers.set_flag("sign", (product & (1 << 7)) != 0);
        self.registers.set_flag("overflow", false);
        self.registers.set_flag("carry", false);
        self.registers.set_general(dest, product)?;
        Ok(Delta {
            registers: vec![],
            flags: vec![],
            memory_access: None,
        })
    }

    pub fn or(&mut self, operands: &[u32]) -> Result<Delta, VMError> {
        let dest = operands[0];
        let num1 = *self.registers.get_general(operands[1])?;
        let num2 = *self.registers.get_general(operands[2])?;
        let product = num1 | num2;
        self.registers.set_flag("zero", product == 0);
        self.registers.set_flag("sign", (product & (1 << 7)) != 0);
        self.registers.set_flag("overflow", false);
        self.registers.set_flag("carry", false);
        self.registers.set_general(dest, product)?;
        Ok(Delta {
            registers: vec![],
            flags: vec![],
            memory_access: None,
        })
    }

    pub fn xor(&mut self, operands: &[u32]) -> Result<Delta, VMError> {
        let dest = operands[0];
        let num1 = *self.registers.get_general(operands[1])?;
        let num2 = *self.registers.get_general(operands[2])?;
        let product = num1 ^ num2;
        self.registers.set_flag("zero", product == 0);
        self.registers.set_flag("sign", (product & (1 << 7)) != 0);
        self.registers.set_flag("overflow", false);
        self.registers.set_flag("carry", false);
        self.registers.set_general(dest, product)?;
        Ok(Delta {
            registers: vec![],
            flags: vec![],
            memory_access: None,
        })
    }

    pub fn not(&mut self, operands: &[u32]) -> Result<Delta, VMError> {
        let dest = operands[0];
        let num1 = *self.registers.get_general(operands[1])?;
        let product = !num1;
        self.registers.set_flag("zero", product == 0);
        self.registers.set_flag("sign", (product & (1 << 7)) != 0);
        self.registers.set_flag("overflow", false);
        self.registers.set_flag("carry", false);
        self.registers.set_general(dest, product)?;
        Ok(Delta {
            registers: vec![],
            flags: vec![],
            memory_access: None,
        })
    }

    pub fn shl(&mut self, operands: &[u32]) -> Result<Delta, VMError> {
        let reg = operands[0];
        let value = *self.registers.get_general(reg)?;
        self.registers.set_flag("carry", (value & (1 << 7)) != 0);
        let shifted_value = value << 1;
        self.registers.set_flag("zero", shifted_value == 0);
        self.registers
            .set_flag("sign", (shifted_value & (1 << 7)) != 0);
        self.registers
            .set_flag("overflow", ((shifted_value ^ value) & (1 << 7)) != 0);
        self.registers.set_general(operands[0], shifted_value)?;
        Ok(Delta {
            registers: vec![],
            flags: vec![],
            memory_access: None,
        })
    }

    pub fn shr(&mut self, operands: &[u32]) -> Result<Delta, VMError> {
        let reg = operands[0];
        let value = *self.registers.get_general(reg)? as i8;
        self.registers.set_flag("carry", (value & 1) != 0);
        let value = value >> 1;
        self.registers.set_flag("zero", value == 0);
        self.registers.set_flag("sign", (value & (1 << 7)) != 0);
        self.registers.set_flag("overflow", false);
        self.registers.set_general(operands[0], value as u8)?;
        Ok(Delta {
            registers: vec![],
            flags: vec![],
            memory_access: None,
        })
    }

    pub fn push(&mut self, operands: &[u32]) -> Result<Delta, VMError> {
        let reg = operands[0];
        let value = *self.registers.get_general(reg)?;
        self.registers.sp -= 1;
        self.memory.set(self.registers.sp, value)?;
        Ok(Delta {
            registers: vec![],
            flags: vec![],
            memory_access: None,
        })
    }

    pub fn pop(&mut self, operands: &[u32]) -> Result<Delta, VMError> {
        let reg = operands[0];
        let value = *self.memory.get(self.registers.sp)?;
        self.registers.sp += 1;
        self.registers.set_general(reg, value)?;
        Ok(Delta {
            registers: vec![],
            flags: vec![],
            memory_access: None,
        })
    }

    pub fn call(&mut self, operands: &[u32]) -> Result<Delta, VMError> {
        self.registers.sp -= 1;
        self.memory
            .set(self.registers.sp, self.registers.pc as u8)?;
        // self.registers.sp -= 1;
        // self.memory
        //     .set(self.registers.sp, (self.registers.pc >> 8) as u8)?;
        // self.registers.sp -= 1;
        // self.memory
        //     .set(self.registers.sp, (self.registers.pc >> 16) as u8)?;
        // self.registers.sp -= 1;
        // self.memory
        //     .set(self.registers.sp, (self.registers.pc >> 24) as u8)?;
        self.registers.pc = operands[0];
        Ok(Delta {
            registers: vec![],
            flags: vec![],
            memory_access: None,
        })
    }

    pub fn ret(&mut self, _: &[u32]) -> Result<Delta, VMError> {
        let mut location: u32 = 0;
        location |= *self.memory.get(self.registers.sp)? as u32;
        self.registers.sp += 1;
        // location |= (*self.memory.get(self.registers.sp)? as u32) << 8;
        // self.registers.sp += 1;
        // location |= (*self.memory.get(self.registers.sp)? as u32) << 16;
        // self.registers.sp += 1;
        // location |= (*self.memory.get(self.registers.sp)? as u32) << 24;
        // self.registers.sp += 1;
        self.registers.pc = location;
        Ok(Delta {
            registers: vec![],
            flags: vec![],
            memory_access: None,
        })
    }

    pub fn jmp(&mut self, operands: &[u32]) -> Result<Delta, VMError> {
        let address = operands[0];
        self.registers.pc = address;
        Ok(Delta {
            registers: vec![],
            flags: vec![],
            memory_access: None,
        })
    }
}
