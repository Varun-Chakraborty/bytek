export interface VMState {
  registers: {
    general_registers: {
      count: number;
      regs: number[];
    }
    flags: {
      zero: boolean;
      carry: boolean;
      sign: boolean;
      overflow: boolean;
    }
    pc: number;
    eof: number;
    sp: number;
  };
  program_memory: {
    mem: number[];
  };
  data_memory: {
    mem: number[];
  };
}

export interface ExecutionStep {
  instruction: string;
  address: number;
  changed_registers: string[];
  memory_access?: {
    address: number;
    type_: number;
    value: number;
  };
  is_halted: boolean;
}
