//! avatar_vm bytecode interpreter — a port of `stackchan-idf`'s
//! `components/avatar_vm` (`decoder.cpp` + `vm.cpp`, bytecode format `AVDS` v1).
//!
//! The face is described by a small stack-machine program compiled from the avatar DSL
//! (`.avdsl` → `.avbc`, see `tools/avatar_dsl` in stackchan-idf). The VM is canvas-size
//! agnostic: `Tx`/`Ty` map the 320x240 design space to the live canvas around its
//! center with a uniform scale, so the same bytecode drives different panels.
//!
//! Colours travel across the operand stack as `f32`-encoded RGB565 values, exactly like
//! the C++ implementation.

use alloc::vec::Vec;

use micromath::F32Ext;

/// `"AVDS"` little-endian.
pub const MAGIC: u32 = 0x5344_5641;
pub const VERSION: u16 = 1;

const HEADER_SIZE: usize = 16;
const STACK_SIZE: usize = 64;
const MAX_LOCALS: usize = 256;
const MAX_CALL_DEPTH: usize = 16;

// Opcodes (see opcodes.hpp).
mod op {
    pub const NOP: u8 = 0x00;
    pub const PUSH_F32: u8 = 0x01;
    pub const PUSH_I8: u8 = 0x02;
    pub const PUSH_I16: u8 = 0x03;
    pub const PUSH_CONST: u8 = 0x04;
    pub const PUSH_VAR: u8 = 0x05;
    pub const PUSH_LOCAL: u8 = 0x06;
    pub const STORE_LOCAL: u8 = 0x07;
    pub const POP: u8 = 0x08;
    pub const DUP: u8 = 0x09;
    pub const ADD: u8 = 0x10;
    pub const SUB: u8 = 0x11;
    pub const MUL: u8 = 0x12;
    pub const DIV: u8 = 0x13;
    pub const NEG: u8 = 0x14;
    pub const MIN: u8 = 0x15;
    pub const MAX: u8 = 0x16;
    pub const ABS: u8 = 0x17;
    pub const FLOOR: u8 = 0x18;
    pub const ROUND: u8 = 0x19;
    pub const MOD: u8 = 0x1A;
    pub const SQRT: u8 = 0x1B;
    pub const CLAMP: u8 = 0x1C;
    pub const SCALE: u8 = 0x1D;
    pub const TX: u8 = 0x1E;
    pub const TY: u8 = 0x1F;
    pub const EQ: u8 = 0x20;
    pub const NE: u8 = 0x21;
    pub const LT: u8 = 0x22;
    pub const LE: u8 = 0x23;
    pub const GT: u8 = 0x24;
    pub const GE: u8 = 0x25;
    pub const NOT: u8 = 0x26;
    pub const AND: u8 = 0x27;
    pub const OR: u8 = 0x28;
    pub const XOR: u8 = 0x29;
    pub const JMP: u8 = 0x30;
    pub const JZ: u8 = 0x31;
    pub const JNZ: u8 = 0x32;
    pub const CALL: u8 = 0x33;
    pub const RET: u8 = 0x34;
    pub const FILL_RECT: u8 = 0x40;
    pub const FILL_CIRCLE: u8 = 0x41;
    pub const FILL_TRIANGLE: u8 = 0x42;
    pub const BEGIN_GROUP: u8 = 0x45;
    pub const END_GROUP: u8 = 0x46;
}

// Context variable IDs (see opcodes.hpp `Var`).
const VAR_COUNT: u8 = 0x1F;

// Const table tags.
const CONST_TAG_F32: u8 = 0x01;
const CONST_TAG_I32: u8 = 0x02;
const CONST_TAG_COLOR: u8 = 0x03;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum VmError {
    BadMagic,
    BadVersion,
    Truncated,
    BadConstTag,
    UnknownOpcode,
    BadVarId,
    BadConstId,
    BadFnId,
    BadLocalSlot,
    StackUnderflow,
    StackOverflow,
    CallDepthExceeded,
    JumpOutOfBounds,
    DivideByZero,
    EntryFnInvalid,
}

/// One entry in the function table. Code offset is into the code section.
#[derive(Clone, Copy, Debug)]
pub struct FnEntry {
    pub code_offset: u16,
    pub param_count: u8,
    pub local_count: u8,
}

/// Decoded bytecode. Unlike the C++ version the code section is copied (owned), so the
/// source buffer (e.g. a transient HTTP body) may be freed immediately.
pub struct Bytecode {
    consts: Vec<f32>,
    fns: Vec<FnEntry>,
    code: Vec<u8>,
    entry_fn_id: u16,
}

fn rd_u16(p: &[u8]) -> u16 {
    u16::from_le_bytes([p[0], p[1]])
}
fn rd_i16(p: &[u8]) -> i16 {
    rd_u16(p) as i16
}
fn rd_u32(p: &[u8]) -> u32 {
    u32::from_le_bytes([p[0], p[1], p[2], p[3]])
}
fn rd_f32(p: &[u8]) -> f32 {
    f32::from_bits(rd_u32(p))
}

/// Parse an `AVDS` v1 bytecode buffer.
pub fn decode(data: &[u8]) -> Result<Bytecode, VmError> {
    if data.len() < HEADER_SIZE {
        return Err(VmError::Truncated);
    }
    if rd_u32(data) != MAGIC {
        return Err(VmError::BadMagic);
    }
    if rd_u16(&data[4..]) != VERSION {
        return Err(VmError::BadVersion);
    }
    let const_count = rd_u16(&data[8..]) as usize;
    let fn_count = rd_u16(&data[10..]) as usize;
    let code_size = rd_u16(&data[12..]) as usize;
    let entry_fn = rd_u16(&data[14..]);

    let mut off = HEADER_SIZE;
    let mut consts = Vec::with_capacity(const_count);
    for _ in 0..const_count {
        if off + 1 > data.len() {
            return Err(VmError::Truncated);
        }
        let tag = data[off];
        off += 1;
        match tag {
            CONST_TAG_F32 => {
                if off + 4 > data.len() {
                    return Err(VmError::Truncated);
                }
                consts.push(rd_f32(&data[off..]));
                off += 4;
            }
            CONST_TAG_I32 => {
                if off + 4 > data.len() {
                    return Err(VmError::Truncated);
                }
                consts.push(rd_u32(&data[off..]) as i32 as f32);
                off += 4;
            }
            CONST_TAG_COLOR => {
                if off + 2 > data.len() {
                    return Err(VmError::Truncated);
                }
                consts.push(rd_u16(&data[off..]) as f32);
                off += 2;
            }
            _ => return Err(VmError::BadConstTag),
        }
    }

    let mut fns = Vec::with_capacity(fn_count);
    for _ in 0..fn_count {
        if off + 6 > data.len() {
            return Err(VmError::Truncated);
        }
        let fe = FnEntry {
            code_offset: rd_u16(&data[off..]),
            param_count: data[off + 2],
            local_count: data[off + 3],
        };
        if fe.local_count < fe.param_count {
            return Err(VmError::BadLocalSlot);
        }
        fns.push(fe);
        off += 6;
    }

    if off + code_size > data.len() {
        return Err(VmError::Truncated);
    }
    let code = data[off..off + code_size].to_vec();

    if entry_fn as usize >= fns.len() {
        return Err(VmError::EntryFnInvalid);
    }
    let entry = fns[entry_fn as usize];
    if entry.code_offset as usize >= code_size || entry.param_count != 0 {
        return Err(VmError::EntryFnInvalid);
    }
    Ok(Bytecode {
        consts,
        fns,
        code,
        entry_fn_id: entry_fn,
    })
}

/// Drawing + context backend the VM renders through. Coordinates are canvas pixels;
/// colours are RGB565. Mirrors `stackchan::avatar::Canvas` (the drawing subset the VM
/// uses) plus the context-variable reads.
pub trait VmCanvas {
    fn width(&self) -> i32;
    fn height(&self) -> i32;
    fn is_circular(&self) -> bool {
        false
    }
    fn fill_rect(&mut self, x: i32, y: i32, w: i32, h: i32, color: u16);
    fn fill_circle(&mut self, cx: i32, cy: i32, r: i32, color: u16);
    #[allow(clippy::too_many_arguments)]
    fn fill_triangle(&mut self, x0: i32, y0: i32, x1: i32, y1: i32, x2: i32, y2: i32, color: u16);
    fn begin_group(&mut self, x: i32, y: i32, w: i32, h: i32);
    fn end_group(&mut self);
    /// Context variable read (`Var` id from opcodes.hpp). `scale` is the design→canvas
    /// uniform scale, provided back for `Var::CanvasScale`.
    fn read_var(&self, id: u8, scale: f32) -> f32;
}

#[derive(Clone, Copy, Default)]
struct Frame {
    return_pc: u16,
    locals_base: u16,
    locals_size: u16,
}

/// The interpreter. Fixed-size stacks, no allocation during `run`.
pub struct Vm {
    stack: [f32; STACK_SIZE],
    locals: [f32; MAX_LOCALS],
    frames: [Frame; MAX_CALL_DEPTH],
}

impl Vm {
    pub const fn new() -> Self {
        Self {
            stack: [0.0; STACK_SIZE],
            locals: [0.0; MAX_LOCALS],
            frames: [Frame {
                return_pc: 0,
                locals_base: 0,
                locals_size: 0,
            }; MAX_CALL_DEPTH],
        }
    }

    /// Compute the design→canvas uniform scale (from vm.cpp `canvas_scale`).
    pub fn canvas_scale(canvas: &dyn VmCanvas) -> f32 {
        const BASE_W: f32 = 320.0;
        const BASE_H: f32 = 240.0;
        if canvas.is_circular() {
            const CORNER_MARGIN: f32 = 0.97;
            const BASE_DIAG: f32 = 400.0; // sqrt(320² + 240²)
            let diameter = canvas.width().min(canvas.height()) as f32;
            return diameter / BASE_DIAG * CORNER_MARGIN;
        }
        (canvas.width() as f32 / BASE_W).min(canvas.height() as f32 / BASE_H)
    }

    pub fn run<C: VmCanvas>(&mut self, bc: &Bytecode, canvas: &mut C) -> Result<(), VmError> {
        let mut sp: usize = 0;
        let mut fp: usize = 0;
        let scale = Self::canvas_scale(canvas);
        let cx_screen = canvas.width() as f32 / 2.0;
        let cy_screen = canvas.height() as f32 / 2.0;

        macro_rules! push {
            ($v:expr) => {{
                if sp >= STACK_SIZE {
                    return Err(VmError::StackOverflow);
                }
                self.stack[sp] = $v;
                sp += 1;
            }};
        }
        macro_rules! pop {
            () => {{
                if sp == 0 {
                    return Err(VmError::StackUnderflow);
                }
                sp -= 1;
                self.stack[sp]
            }};
        }

        // Entry frame (entry fn has 0 params, verified by the decoder).
        let entry = bc.fns[bc.entry_fn_id as usize];
        self.frames[fp] = Frame {
            return_pc: 0,
            locals_base: 0,
            locals_size: entry.local_count as u16,
        };
        fp += 1;
        for slot in self.locals[..entry.local_count as usize].iter_mut() {
            *slot = 0.0;
        }

        let code = bc.code.as_slice();
        let code_size = code.len();
        let mut pc = entry.code_offset as usize;

        macro_rules! jump {
            ($off:expr) => {{
                let target = pc as isize + $off as isize;
                if target < 0 || target > code_size as isize {
                    return Err(VmError::JumpOutOfBounds);
                }
                pc = target as usize;
            }};
        }
        macro_rules! need {
            ($n:expr) => {
                if pc + $n > code_size {
                    return Err(VmError::Truncated);
                }
            };
        }

        loop {
            if pc >= code_size {
                return Err(VmError::Truncated);
            }
            let opcode = code[pc];
            pc += 1;
            match opcode {
                op::NOP => {}

                op::PUSH_F32 => {
                    need!(4);
                    push!(rd_f32(&code[pc..]));
                    pc += 4;
                }
                op::PUSH_I8 => {
                    need!(1);
                    push!(code[pc] as i8 as f32);
                    pc += 1;
                }
                op::PUSH_I16 => {
                    need!(2);
                    push!(rd_i16(&code[pc..]) as f32);
                    pc += 2;
                }
                op::PUSH_CONST => {
                    need!(1);
                    let id = code[pc] as usize;
                    pc += 1;
                    if id >= bc.consts.len() {
                        return Err(VmError::BadConstId);
                    }
                    push!(bc.consts[id]);
                }
                op::PUSH_VAR => {
                    need!(1);
                    let id = code[pc];
                    pc += 1;
                    if id >= VAR_COUNT {
                        return Err(VmError::BadVarId);
                    }
                    push!(canvas.read_var(id, scale));
                }
                op::PUSH_LOCAL => {
                    need!(1);
                    let slot = code[pc] as u16;
                    pc += 1;
                    let f = self.frames[fp - 1];
                    if slot >= f.locals_size {
                        return Err(VmError::BadLocalSlot);
                    }
                    push!(self.locals[(f.locals_base + slot) as usize]);
                }
                op::STORE_LOCAL => {
                    need!(1);
                    let slot = code[pc] as u16;
                    pc += 1;
                    let f = self.frames[fp - 1];
                    if slot >= f.locals_size {
                        return Err(VmError::BadLocalSlot);
                    }
                    let v = pop!();
                    self.locals[(f.locals_base + slot) as usize] = v;
                }
                op::POP => {
                    let _ = pop!();
                }
                op::DUP => {
                    if sp == 0 {
                        return Err(VmError::StackUnderflow);
                    }
                    if sp >= STACK_SIZE {
                        return Err(VmError::StackOverflow);
                    }
                    self.stack[sp] = self.stack[sp - 1];
                    sp += 1;
                }

                op::ADD | op::SUB | op::MUL | op::DIV | op::MIN | op::MAX | op::MOD | op::EQ
                | op::NE | op::LT | op::LE | op::GT | op::GE | op::AND | op::OR | op::XOR => {
                    let b = pop!();
                    let a = pop!();
                    let v = match opcode {
                        op::ADD => a + b,
                        op::SUB => a - b,
                        op::MUL => a * b,
                        op::DIV => {
                            if b == 0.0 {
                                return Err(VmError::DivideByZero);
                            }
                            a / b
                        }
                        op::MIN => a.min(b),
                        op::MAX => a.max(b),
                        op::MOD => {
                            if b == 0.0 {
                                return Err(VmError::DivideByZero);
                            }
                            a % b
                        }
                        op::EQ => (a == b) as u8 as f32,
                        op::NE => (a != b) as u8 as f32,
                        op::LT => (a < b) as u8 as f32,
                        op::LE => (a <= b) as u8 as f32,
                        op::GT => (a > b) as u8 as f32,
                        op::GE => (a >= b) as u8 as f32,
                        op::AND => (a != 0.0 && b != 0.0) as u8 as f32,
                        op::OR => (a != 0.0 || b != 0.0) as u8 as f32,
                        _ => ((a != 0.0) != (b != 0.0)) as u8 as f32, // XOR
                    };
                    push!(v);
                }

                op::NEG | op::ABS | op::FLOOR | op::ROUND | op::SQRT | op::NOT | op::SCALE
                | op::TX | op::TY => {
                    let a = pop!();
                    let v = match opcode {
                        op::NEG => -a,
                        op::ABS => a.abs(),
                        op::FLOOR => a.floor(),
                        op::ROUND => a.round(),
                        op::SQRT => a.sqrt(),
                        op::NOT => (a == 0.0) as u8 as f32,
                        op::SCALE => {
                            let s = a * scale;
                            if s < 1.0 { 1.0 } else { s }
                        }
                        op::TX => cx_screen + (a - 160.0) * scale,
                        _ => cy_screen + (a - 120.0) * scale, // TY
                    };
                    push!(v);
                }

                op::CLAMP => {
                    let hi = pop!();
                    let lo = pop!();
                    let x = pop!();
                    push!(if x < lo {
                        lo
                    } else if x > hi {
                        hi
                    } else {
                        x
                    });
                }

                op::JMP => {
                    need!(2);
                    let off = rd_i16(&code[pc..]);
                    pc += 2;
                    jump!(off);
                }
                op::JZ | op::JNZ => {
                    need!(2);
                    let off = rd_i16(&code[pc..]);
                    pc += 2;
                    let a = pop!();
                    let take = if opcode == op::JZ { a == 0.0 } else { a != 0.0 };
                    if take {
                        jump!(off);
                    }
                }

                op::CALL => {
                    need!(1);
                    let fn_id = code[pc] as usize;
                    pc += 1;
                    if fn_id >= bc.fns.len() {
                        return Err(VmError::BadFnId);
                    }
                    let fe = bc.fns[fn_id];
                    if fp >= MAX_CALL_DEPTH {
                        return Err(VmError::CallDepthExceeded);
                    }
                    let parent = self.frames[fp - 1];
                    let new_base = parent.locals_base + parent.locals_size;
                    if new_base as usize + fe.local_count as usize > MAX_LOCALS {
                        return Err(VmError::BadLocalSlot);
                    }
                    if sp < fe.param_count as usize {
                        return Err(VmError::StackUnderflow);
                    }
                    // Pop arguments into the callee's first `param_count` local slots.
                    for i in (0..fe.param_count as usize).rev() {
                        sp -= 1;
                        self.locals[new_base as usize + i] = self.stack[sp];
                    }
                    for i in fe.param_count as usize..fe.local_count as usize {
                        self.locals[new_base as usize + i] = 0.0;
                    }
                    self.frames[fp] = Frame {
                        return_pc: pc as u16,
                        locals_base: new_base,
                        locals_size: fe.local_count as u16,
                    };
                    fp += 1;
                    pc = fe.code_offset as usize;
                }
                op::RET => {
                    if fp == 0 {
                        return Err(VmError::StackUnderflow);
                    }
                    fp -= 1;
                    let f = self.frames[fp];
                    if fp == 0 {
                        // Root frame returned — program complete.
                        return Ok(());
                    }
                    pc = f.return_pc as usize;
                }

                op::FILL_RECT => {
                    let color = pop!() as u16;
                    let h = pop!() as i32;
                    let w = pop!() as i32;
                    let y = pop!() as i32;
                    let x = pop!() as i32;
                    canvas.fill_rect(x, y, w, h, color);
                }
                op::FILL_CIRCLE => {
                    let color = pop!() as u16;
                    let r = pop!() as i32;
                    let cy = pop!() as i32;
                    let cx = pop!() as i32;
                    canvas.fill_circle(cx, cy, r, color);
                }
                op::FILL_TRIANGLE => {
                    let color = pop!() as u16;
                    let y2 = pop!() as i32;
                    let x2 = pop!() as i32;
                    let y1 = pop!() as i32;
                    let x1 = pop!() as i32;
                    let y0 = pop!() as i32;
                    let x0 = pop!() as i32;
                    canvas.fill_triangle(x0, y0, x1, y1, x2, y2, color);
                }
                op::BEGIN_GROUP => {
                    let h = pop!() as i32;
                    let w = pop!() as i32;
                    let y = pop!() as i32;
                    let x = pop!() as i32;
                    canvas.begin_group(x, y, w, h);
                }
                op::END_GROUP => {
                    canvas.end_group();
                }

                _ => return Err(VmError::UnknownOpcode),
            }
        }
    }
}

impl Default for Vm {
    fn default() -> Self {
        Self::new()
    }
}
