use std::collections::VecDeque;

enum Instruction {
    ClearScreen,
    LoadNormalRegisterImmediate { register: u8, value: u8 },
    LoadIndexRegisterImmediate { value: u16 },
    DrawSpriteToScreen { x: u8, y: u8, n: u8},
    Jump { address: u16},
    AddImmediateToNormalRegister { register: u8, value: u8 },
    JumpToSubroutine { address: u16},
    ReturnFromSubroutine,
}

struct CPU {
    pc: u16,
    index: u16,
    stack: VecDeque<u16>,
    v: [u8; 16]
}
impl CPU {
    fn new() -> Self {
        CPU {
            pc: 0,
            index: 0,
            stack: VecDeque::new(),
            v: [0; 16]
        }
    }
}

pub struct System {
    cpu: CPU,
    memory: [u8; 4096]
}

pub struct DrawAction {
    pub x: u8,
    pub y: u8,
    pub pixels: Vec<[bool; 8]>
}

impl DrawAction {
    fn new() -> Self {
        DrawAction {
            x: 0,
            y: 0,
            pixels: vec![[false; 8]]
        }
    }
}

impl System {
    pub fn new() -> Self {
        System {
            cpu: CPU::new(),
            memory: [0; 4096]
        }
    }

    pub fn load_rom(&mut self, rom: Vec<u8>) {
        self.memory[0x200..0x200 + rom.len()].copy_from_slice(&rom);
        self.cpu.pc = 0x200;
    }

    fn decode(&mut self) -> Result<Instruction, String> {
        let pc = self.cpu.pc as usize;
        self.cpu.pc += 2;
        let bytes: [u8; 2] = match self.memory[pc..pc + 2].try_into() {
            Ok(b) => b,
            Err(_) => return Err(String::from("Memory to byte failed"))
        };
        let opcode = u16::from_be_bytes(bytes);
        println!("Decoding {:#06X}", opcode);
        if opcode == 0x00e0 {
            return Ok(Instruction::ClearScreen);
        }
        let mut rv: Result<Instruction, String> = Err(format!("Unknown opcode {:#x}", opcode));
        match opcode & 0xf000 {
            0x1000 => { rv = Ok(Instruction::Jump {address: opcode & 0x0fff}) },
            0x2000 => { rv = Ok(Instruction::JumpToSubroutine {address: opcode & 0x0fff}) },
            0x6000 => { rv = Ok(Instruction::LoadNormalRegisterImmediate {register: ((opcode & 0x0f00) >> 8) as u8, value: (opcode & 0x00ff) as u8}) },
            0x7000 => { rv = Ok(Instruction::AddImmediateToNormalRegister {register: ((opcode & 0x0f00) >> 8) as u8, value: (opcode & 0x00ff) as u8}) },
            0xa000 => { rv = Ok(Instruction::LoadIndexRegisterImmediate {value: opcode & 0x0fff}) },
            0xD000 => { rv = Ok(Instruction::DrawSpriteToScreen{x : ((opcode & 0x0f00) >> 8) as u8, y: ((opcode & 0x00f0) >> 4) as u8, n: ((opcode & 0x000f) >> 0) as u8}) },
            _ => { }
        };
        rv
    }

    pub fn step(&mut self) -> Option<DrawAction> {
        match self.decode() {
            Ok(Instruction::ClearScreen) => {
                println!("Clearing screen");
            }
            Ok(Instruction::LoadNormalRegisterImmediate { register, value }) => {
                println!("Loading normal register: {:2} with value {:#04X}", register, value);
                self.cpu.v[register as usize] = value;
            }
            Ok(Instruction::LoadIndexRegisterImmediate{ value}) => {
                println!("Loading index register: {:#06X}", value);
                self.cpu.index = value;
            }
            Ok(Instruction::AddImmediateToNormalRegister { register, value }) => {
                print!("Add {:#04X} to v[{}] ({:#04X})", value, register, self.cpu.v[register as usize]);
                self.cpu.v[register as usize] = self.cpu.v[register as usize].wrapping_add(value);
                println!(" => {:#04X}", self.cpu.v[register as usize]);
            }
            Ok(Instruction::DrawSpriteToScreen{x, y, n}) => {
                println!("Drawing sprite to screen v[{}] = {:#04X}, v[{}] = {:#04X}, n = {}", x, self.cpu.v[x as usize], y, self.cpu.v[y as usize], n);
                let mut output: DrawAction = DrawAction::new();
                output.x = self.cpu.v[x as usize];
                output.y = self.cpu.v[y as usize];
                output.pixels = vec![[false; 8]; n as usize];
                for sprite_row in 0..n {
                    let cur_addr = self.cpu.index + sprite_row as u16;
                    let cur_pixel_row = self.memory[cur_addr as usize];
                    println!("Processing {:#04x}", cur_pixel_row);
                    for pixel_idx in 0..8 {
                        if cur_pixel_row & (1 << (7 - pixel_idx)) != 0 {
                            output.pixels[sprite_row as usize][pixel_idx as usize] = true;
                        }
                    }
                }
                return Some(output);
            }
            Ok(Instruction::Jump { address }) => {
                println!("Jumping to {:#04X}", address);
                self.cpu.pc = address;
            },
            Ok(Instruction::JumpToSubroutine { address }) => {
                println!("Jumping to subroutine {:#04x}", address);
                self.cpu.stack.push_front(self.cpu.pc + 2);
                self.cpu.pc = address;
            },
            Ok(Instruction::ReturnFromSubroutine) => {
                let stack_front = self.cpu.stack.pop_front().unwrap();
                println!("Return from subroutine. Stack front: {:#04X}", stack_front);
                self.cpu.pc = stack_front;
            }
            Err(msg) => {
                println!("Error decoding: {}", msg)
            }
        }
        None
    }
}
