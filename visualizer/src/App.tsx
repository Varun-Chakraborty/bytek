import { useState, useEffect, useCallback } from 'react';
import {
    Play, Pause, StepForward, RotateCcw, Cpu, Save,
    Zap, Database, FileCode, Terminal, Activity,
    Microchip, Layers, ArrowRight
} from 'lucide-react';

import { VM } from './lib/vm';
import type { ExecutionStep, VMState } from './types/vm';

const toHex = (num: number | undefined): string =>
    (num || 0).toString(16).toUpperCase().padStart(2, '0');

interface PanelProps {
    title: string;
    icon: React.ElementType;
    children: React.ReactNode;
    className?: string;
}

const Panel: React.FC<PanelProps> = ({ title, icon: Icon, children, className = "" }) => (
    <div className={`bg-gray-950/60 backdrop-blur-md border border-white/5 rounded-xl flex flex-col overflow-hidden shadow-2xl ${className}`}>
        <div className="bg-white/5 px-4 py-3 flex items-center justify-between border-b border-white/5">
            <div className="flex items-center gap-2 text-gray-400">
                <Icon size={14} className="text-cyan-400" />
                <span className="text-xs font-bold tracking-wider uppercase">{title}</span>
            </div>
            <div className="flex gap-1.5">
                <div className="w-1.5 h-1.5 rounded-full bg-red-500/20"></div>
                <div className="w-1.5 h-1.5 rounded-full bg-yellow-500/20"></div>
                <div className="w-1.5 h-1.5 rounded-full bg-green-500/20"></div>
            </div>
        </div>
        <div className="flex-1 relative">
            {children}
        </div>
    </div>
);

interface MemoryCellProps {
    val: number;
    type: 'program' | 'data';
    idx: number;
    activeType: 'pc' | 'sp' | 'read' | 'write' | null;
}

const MemoryCell: React.FC<MemoryCellProps> = ({ val, type, idx, activeType }) => {
    let bgClass = 'bg-gray-900/40 text-gray-700';

    if (activeType === 'pc') bgClass = 'bg-emerald-500 text-black font-bold shadow-[0_0_15px_rgba(16,185,129,0.5)] z-10 scale-110';
    else if (activeType === 'sp') bgClass = 'bg-pink-500 text-white font-bold shadow-[0_0_15px_rgba(236,72,153,0.5)] z-10 scale-110';
    else if (activeType === 'write') bgClass = 'bg-orange-500 text-white animate-pulse z-10';
    else if (activeType === 'read') bgClass = 'bg-yellow-400 text-black animate-pulse z-10';
    else if (val !== 0) bgClass = type === 'program' ? 'bg-emerald-900/20 text-emerald-400 border border-emerald-500/20' : 'bg-cyan-900/20 text-cyan-400 border border-cyan-500/20';

    return (
        <div
            className={`aspect-square flex items-center justify-center text-[9px] font-mono rounded-sm transition-all duration-300 ${bgClass}`}
            title={`0x${toHex(idx)}: 0x${toHex(val)}`}
        >
            {toHex(val)}
        </div>
    );
};

interface RegisterBoxProps {
    label: string;
    value?: number;
    hexValue: string;
    isChanged: boolean;
    colorClass?: string;
    borderColor?: string;
    children?: React.ReactNode;
}

const RegisterBox: React.FC<RegisterBoxProps> = ({
    label, value, hexValue, isChanged,
    colorClass = "text-white", borderColor = "border-white/5", children
}) => (
    <div className={`relative group bg-black/40 rounded-lg p-3 border transition-all duration-300 ${isChanged ? 'border-cyan-500 shadow-[0_0_15px_rgba(6,182,212,0.15)]' : borderColor}`}>
        <div className="flex justify-between items-start mb-1">
            <span className={`text-[10px] font-bold ${isChanged ? 'text-cyan-400' : 'text-gray-500'}`}>{label}</span>
            {value !== undefined && <span className="text-[10px] text-gray-600">DEC: {value}</span>}
        </div>
        <div className={`text-2xl ${colorClass} font-light tracking-wider flex items-center gap-2`}>
            {hexValue}
            {isChanged && <Activity size={14} className="text-cyan-500 animate-pulse" />}
        </div>
        {children}
        <div className="absolute bottom-0 left-0 h-0.5 bg-cyan-500 transition-all duration-500" style={{ width: isChanged ? '100%' : '0%' }}></div>
    </div>
);

export default function VMVisualizer() {
    // Initialize Controller
    // Ensure MyVMController is available in the global scope or imported
    const [controller, setController] = useState<VM>();

    useEffect(() => {
        VM.initialize().then(setController)
    }, []);

    // State
    const [VMState, setVMState] = useState<VMState>();

    const [lastStep, setLastStep] = useState<ExecutionStep | null>(null);
    const [isRunning, setIsRunning] = useState<boolean>(false);
    const [speed, setSpeed] = useState<number>(200);

    const [sourceCode, setSourceCode] = useState<string>(
        `; CYBERPUNK ISA DEMO
; R0-R3: General Registers

START:
  MOVEI R0, 5    ; Counter
  MOVEI R1, 1    ; Increment
  MOVEI R2, 0    ; Acc

  CALL FUNCADD  
  
  MOVEI R3, 10
  MOVEM R3, 250  ; Save

LOOP:
  ADD R2, R2, R1 ; Acc++
  CMP R2, R0     
  JZ DONE        
  JMP LOOP       

FUNCADD:
  PUSH R0        
  MOVEI R0, 99   
  POP R0         
  RET

DONE:
  HALT
`
    );

    const loadAndReset = useCallback(async () => {
        try {
            if (!controller) throw new Error("Controller not initialized");
            console.log(sourceCode, controller);
            controller.loadProgram(sourceCode);
            setVMState(controller.getState());
            setLastStep(null);
            setIsRunning(false);
        } catch (e) {
            console.error("WASM Load Error:", e);
        }
    }, [sourceCode, controller]);

    const executeStep = useCallback(async () => {
        try {
            const stepInfo = controller?.step() ?? null;
            setLastStep(stepInfo);
            setVMState(controller?.getState());

            if (stepInfo?.is_halted) {
                setIsRunning(false);
            }
        } catch (e) {
            console.error("Execution Error:", e);
            setIsRunning(false);
        }
    }, [controller]);

    useEffect(() => {
        let interval: ReturnType<typeof setInterval>;
        if (isRunning) {
            interval = setInterval(executeStep, speed);
        }
        return () => clearInterval(interval);
    }, [isRunning, executeStep, speed]);

    // Cleanup on unmount
    useEffect(() => {
        if (!controller) return;
        setVMState(controller.getState())
        setVMState({
            registers: {
                general_registers: { count: 4, regs: [0, 0, 0, 0] },
                flags: { zero: false, carry: false, sign: false, overflow: false },
                pc: 0, eof: 0, sp: 0
            },
            program_memory: { mem: new Array(256).fill(0) },
            data_memory: { mem: new Array(256).fill(0) }
        })
        return () => {
            controller.free();
        }
    }, [controller]);

    // Access registers for rendering
    const regs = VMState?.registers;
    const genRegs = regs?.general_registers.regs;

    return (
        <div className="min-h-screen bg-[#0a0a0f] text-gray-300 font-mono p-4 flex flex-col gap-4 overflow-hidden selection:bg-cyan-500/30">

            {/* --- TOP BAR --- */}
            <div className="flex flex-col md:flex-row justify-between items-center gap-4 bg-gray-900/30 border border-white/5 p-3 rounded-xl backdrop-blur-sm">
                <div className="flex items-center gap-3">
                    <div className="relative">
                        <div className="w-10 h-10 bg-gradient-to-br from-cyan-600 to-blue-700 rounded-lg flex items-center justify-center shadow-lg shadow-cyan-500/20">
                            <Cpu className="w-6 h-6 text-white" />
                        </div>
                        <div className={`absolute -bottom-1 -right-1 w-3 h-3 rounded-full border-2 border-[#0a0a0f] ${isRunning ? 'bg-green-500 animate-pulse' : 'bg-red-500'}`}></div>
                    </div>
                    <div>
                        <h1 className="text-lg font-bold text-white tracking-tight">VIRTUAL<span className="text-cyan-400">CORE</span></h1>
                        <p className="text-[10px] text-gray-500 uppercase tracking-widest">WASM-8 Architecture</p>
                    </div>
                </div>

                {/* Control Deck */}
                <div className="flex items-center gap-2 bg-black/40 p-1.5 rounded-lg border border-white/5">
                    <button onClick={loadAndReset} className="p-2 hover:bg-white/10 text-gray-400 hover:text-white rounded-md transition-colors" title="Reset">
                        <RotateCcw size={18} />
                    </button>
                    <div className="w-px h-6 bg-white/10 mx-1"></div>
                    <button
                        onClick={() => setIsRunning(!isRunning)}
                        className={`flex items-center gap-2 px-4 py-1.5 rounded-md font-bold transition-all ${isRunning
                            ? 'bg-red-500/20 text-red-400 hover:bg-red-500/30 border border-red-500/50 shadow-[0_0_15px_rgba(239,68,68,0.2)]'
                            : 'bg-emerald-500/20 text-emerald-400 hover:bg-emerald-500/30 border border-emerald-500/50 shadow-[0_0_15px_rgba(16,185,129,0.2)]'
                            }`}
                    >
                        {isRunning ? <Pause size={16} /> : <Play size={16} />}
                        <span className="text-xs">{isRunning ? "HALT" : "EXECUTE"}</span>
                    </button>
                    <button onClick={executeStep} disabled={isRunning} className="p-2 hover:bg-white/10 text-cyan-400 disabled:opacity-30 rounded-md transition-colors" title="Step">
                        <StepForward size={18} />
                    </button>
                </div>

                {/* Speed Control */}
                <div className="flex items-center gap-3 px-4">
                    <Zap size={14} className="text-yellow-500" />
                    <div className="flex flex-col w-32">
                        <input
                            type="range" min="10" max="1000" step="10" dir="rtl"
                            value={speed} onChange={(e) => setSpeed(Number(e.target.value))}
                            className="h-1 bg-gray-800 rounded-lg appearance-none cursor-pointer accent-yellow-500"
                        />
                    </div>
                    <span className="text-[10px] text-gray-500 font-bold">{speed}ms</span>
                </div>
            </div>

            <main className="flex-1 grid grid-cols-1 lg:grid-cols-12 gap-4 h-full min-h-0">

                {/* --- LEFT: SOURCE TERMINAL --- */}
                <Panel title="Assembly Terminal" icon={Terminal} className="lg:col-span-4 min-h-[400px]">
                    <div className="absolute inset-0 flex flex-col">
                        <textarea
                            value={sourceCode}
                            onChange={(e) => setSourceCode(e.target.value)}
                            className="flex-1 bg-transparent text-sm p-4 text-emerald-400 focus:outline-none font-mono resize-none leading-relaxed placeholder-white/20 custom-scrollbar"
                            spellCheck="false"
                        />
                        <div className="h-8 bg-black/40 border-t border-white/5 flex items-center px-4 text-[10px] text-gray-500 justify-between">
                            <span>STATUS: {isRunning ? 'RUNNING' : 'IDLE'}</span>
                            <span className="text-cyan-400">{lastStep ? lastStep.instruction : "READY"}</span>
                        </div>
                    </div>
                </Panel>

                {/* --- CENTER & RIGHT: MACHINE STATE --- */}
                <div className="lg:col-span-8 flex flex-col gap-4">

                    {/* Unified Registers Panel (General + Special) */}
                    <Panel title="VM Registers" icon={Microchip} className="h-64">
                        <div className="absolute inset-0 p-4 overflow-y-auto custom-scrollbar">

                            <div className="p-4 h-full flex flex-col gap-4">

                                {/* General Purpose Section */}
                                <div className="flex items-center gap-2 mb-1">
                                    <div className="h-px bg-white/10 flex-1"></div>
                                    <span className="text-[10px] text-gray-500 font-bold uppercase tracking-widest">General Purpose</span>
                                    <div className="h-px bg-white/10 flex-1"></div>
                                </div>

                                <div className="grid grid-cols-4 gap-4">
                                    {genRegs?.map((val, idx) => (
                                        <RegisterBox
                                            key={`r${idx}`}
                                            label={`R${idx}`}
                                            value={val}
                                            hexValue={`0x${toHex(val)}`}
                                            isChanged={lastStep?.changed_registers.includes(`R${idx}`) ?? false}
                                        />
                                    ))}
                                </div>

                                {/* Special Purpose Section */}
                                <div className="flex items-center gap-2 mb-1 mt-2">
                                    <div className="h-px bg-white/10 flex-1"></div>
                                    <span className="text-[10px] text-gray-500 font-bold uppercase tracking-widest">Special Purpose</span>
                                    <div className="h-px bg-white/10 flex-1"></div>
                                </div>

                                <div className="grid grid-cols-4 gap-4">
                                    {/* PC */}
                                    <div className="col-span-1">
                                        <RegisterBox
                                            label="PC"
                                            value={regs?.pc}
                                            hexValue={`0x${toHex(regs?.pc)}`}
                                            isChanged={false}
                                            colorClass="text-emerald-400"
                                            borderColor="border-emerald-500/20"
                                        />
                                    </div>

                                    {/* SP */}
                                    <div className="col-span-1">
                                        <RegisterBox
                                            label="SP"
                                            value={regs?.sp}
                                            hexValue={`0x${toHex(regs?.sp)}`}
                                            isChanged={false}
                                            colorClass="text-pink-400"
                                            borderColor="border-pink-500/20"
                                        />
                                    </div>

                                    {/* EOF */}
                                    <div className="col-span-1">
                                        <RegisterBox
                                            label="EOF"
                                            hexValue={`0x${toHex(regs?.eof)}`}
                                            isChanged={false}
                                            colorClass={regs?.eof ? "text-red-400" : "text-gray-600"}
                                            borderColor={regs?.eof ? "border-red-500/20" : "border-white/5"}
                                        />
                                    </div>

                                    {/* Flags Grouped */}
                                    <div className="col-span-1 flex gap-1 items-end justify-end pb-2">
                                        {[
                                            { label: 'Z', val: regs?.flags.zero },
                                            { label: 'S', val: regs?.flags.sign },
                                            { label: 'O', val: regs?.flags.overflow },
                                            { label: 'C', val: regs?.flags.carry },
                                        ].map((f, i) => (
                                            <div key={i} className={`w-6 h-6 rounded flex items-center justify-center text-[10px] font-bold border transition-all ${f.val ? 'bg-yellow-500 text-black border-yellow-400 shadow-[0_0_10px_rgba(234,179,8,0.5)]' : 'bg-black/20 text-gray-700 border-white/5'}`}>
                                                {f.label}
                                            </div>
                                        ))}
                                    </div>
                                </div>
                            </div>
                        </div>
                    </Panel>

                    {/* Memory Banks */}
                    <div className="flex-1 grid grid-cols-1 md:grid-cols-2 gap-4 min-h-[300px]">

                        {/* Program Memory */}
                        <Panel title="Program Memory" icon={FileCode}>
                            <div className="absolute inset-0 p-4 overflow-y-auto custom-scrollbar">
                                <div className="grid grid-cols-16 gap-0.5">
                                    {VMState?.program_memory.mem.map((val, i) => (
                                        <MemoryCell
                                            key={`pm-${i}`} idx={i} val={val} type="program"
                                            activeType={i === regs?.pc ? 'pc' : null}
                                        />
                                    ))}
                                </div>
                            </div>
                        </Panel>

                        {/* Data Memory */}
                        <Panel title="Data Stack" icon={Database}>
                            <div className="absolute inset-0 p-4 overflow-y-auto custom-scrollbar">
                                <div className="grid grid-cols-16 gap-0.5">
                                    {VMState?.data_memory.mem.map((val, i) => {
                                        let accessType: 'sp' | 'read' | 'write' | null = null;
                                        if (i === regs?.sp) accessType = 'sp';
                                        else if (lastStep?.memory_access?.address === i && lastStep?.memory_access?.type_ === 0) accessType = 'read';
                                        else if (lastStep?.memory_access?.address === i && lastStep?.memory_access?.type_ === 1) accessType = 'write';

                                        return <MemoryCell key={`dm-${i}`} idx={i} val={val} type="data" activeType={accessType} />;
                                    })}
                                </div>
                            </div>
                        </Panel>

                    </div>
                </div>
            </main>

            <style>{`
        .custom-scrollbar::-webkit-scrollbar { width: 4px; }
        .custom-scrollbar::-webkit-scrollbar-track { background: rgba(0,0,0,0.3); }
        .custom-scrollbar::-webkit-scrollbar-thumb { background: rgba(255,255,255,0.1); border-radius: 2px; }
        .custom-scrollbar::-webkit-scrollbar-thumb:hover { background: rgba(255,255,255,0.2); }
      `}</style>
        </div>
    );
}