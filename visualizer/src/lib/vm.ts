import type { VMState, ExecutionStep } from '../types/vm';

import init, { MyVMController } from './wasm-vm/wasm_wrapper';

export class VM {
  private vm: MyVMController | null = null;
  private static wasmInitialized = false;
  private is_halted = false;

  public static async initialize(): Promise<VM> {
    if (!VM.wasmInitialized) {
      await init(); // This loads the .wasm file
      VM.wasmInitialized = true;
    }

    return new VM();
  }


  private constructor() {
    this.vm = new MyVMController(); // This calls your Rust constructor
  }

  loadProgram(assembly: string): boolean {
    if (!this.vm) return false;

    try {
      return this.vm.loadProgram(assembly);
    } catch (error) {
      console.error('Failed to load program:', error);
      return false;
    }
  }

  step(): ExecutionStep | null {
    if (!this.vm) return null;

    try {
      const result = this.vm.step();
      console.log(result);
      this.is_halted = result.is_halted;
      return {
        instruction: result.instruction,
        address: result.address,
        changed_registers: result.changed_registers || [],
        is_halted: result.is_halted,
        memory_access: result.memory_access
      };
    } catch (error) {
      console.error('Step execution failed:', error);
      return null;
    }
  }

  reset(): void {
    if (this.vm) this.vm.reset();
  }

  getState(): VMState {
    if (!this.vm) throw new Error("VM not initialized");

    return this.vm.getState() as VMState;
  }

  isHalted(): boolean {
    if (!this.vm) return true;

    return this.is_halted;
  }

  free(): void {
    if (this.vm) this.vm.free();
  }
}
