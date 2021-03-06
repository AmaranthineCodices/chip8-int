const MEM_SIZE: usize = 0x1000;
const GFX_SIZE_X: usize = 64;
const GFX_SIZE_Y: usize = 32;

#[derive(Debug, PartialEq)]
pub enum Opcode {
    // Not defined: opcode 0NNN (call RCA 1802 program).
    ClearDisplay,
    Return,
    // Jump to memory address
    Jump { address: u16 },
    // Call subroutine at memory address
    Call { address: u16 },
    // Skip if register value == constant
    SkipIfEqual { register: usize, value: u8 },
    // Skip if register value != constant
    SkipIfNotEqual { register: usize, value: u8 },
    // Skip if register 1 value == register 2 value
    SkipIfRegistersEqual { register1: usize, register2: usize },
    // Set register value to a constant
    SetRegister { register: usize, value: u8 },
    // Add a constant to a register value
    AddConstant { register: usize, value: u8 },
    // Assign a register's value to another register
    CopyRegister { source: usize, target: usize },
    // Sets the target's value to target | other
    BitOr { target: usize, other: usize },
    // Sets the target's value to target & other
    BitAnd { target: usize, other: usize },
    // Sets the target's value to target ^ other
    BitXor { target: usize, other: usize },
    // Add a register's value to another register's value. VF is set to 0 if no carry and 1 if carry.
    AddRegister { target: usize, other: usize },
    // Subtract a register's value from another register's value. VF is set to 0 if borrow and 1 if no borrow.
    SubtractRegister { target: usize, other: usize },
    // Alternative semantics for subtraction.
    AltSubtractRegister { target: usize, other: usize },
    // Shift a register's value to the left by one and store the result in another register.
    LeftShift { target: usize, source: usize },
    // Shift a register's value to the right by one and store the result in another register.
    RightShift { target: usize, source: usize },
    // Skip if register value != other register value
    SkipIfRegistersNotEqual { register1: usize, register2: usize },
    // 0xAnnn: Sets the index register to a value
    SetIndexRegister { value: u16 },
    // Jump to address offset by register value of V0
    OffsetJump { address: u16 },
    // Generate a random number, take the bitwise AND of it, and store the result in a register
    Rand { mask: u8, register: usize },
    // Display a sprite from memory.
    Display { x: usize, y: usize, height: u8 },
    // Skip if a key is pressed.
    SkipIfKeyPressed { key: usize },
    // Skip if a key is not pressed.
    SkipIfKeyNotPressed { key: usize },
    // Get delay timer value and store it in a register.
    GetDelayTimer { register: usize },
    // Await a key press.
    AwaitKeypress { register: usize },
    // Set delay timer.
    SetDelayTimer { value: usize },
    // Set sound timer.
    SetSoundTimer { value: usize },
    // Increment index register by a register's value.
    IncrementIndexRegister { register: usize },
    // Set index register to a font character in a register
    SetIndexToFont { register: usize },
    // Store binary-coded decimal repr. of a register based on the index register
    StoreDecimal { register: usize },
    // Dump registers to memory starting at the index register
    MemDump { max_register: usize },
    // Load registers from memory
    MemLoad { max_register: usize },
}

fn decode_opcode(opcode: u16) -> Option<Opcode> {
    // 0x00E0: Clear screen
    if opcode == 0x00E0 {
        return Some(Opcode::ClearDisplay);
    }
    // 0x00EE: Return
    else if opcode == 0x00EE {
        return Some(Opcode::Return);
    }
    // 0x1nnn: Jump
    else if opcode & 0xF000 == 0x1000 {
        return Some(Opcode::Jump { address: opcode & 0x0FFF });
    }
    // 0x2nnn: Call at address
    else if opcode & 0xF000 == 0x2000 {
        return Some(Opcode::Call { address: opcode & 0x0FFF });
    }
    // 0x3rnn: Skip if register Vr == nn
    else if opcode & 0xF000 == 0x3000 {
        return Some(Opcode::SkipIfEqual {
            register: ((opcode & 0x0F00) >> 8) as usize,
            value: (opcode & 0x00FF) as u8 }
        );
    }
    // 0x4rnn: Skip if register Vr != nn
    else if opcode & 0xF000 == 0x4000 {
        return Some(Opcode::SkipIfNotEqual {
            register: ((opcode & 0x0F00) >> 8) as usize,
            value: (opcode & 0x00FF) as u8,
        });
    }
    // 0x5xy0: Skip if register Vx == register Vy
    else if opcode & 0xF000 == 0x5000 {
        return Some(Opcode::SkipIfRegistersEqual {
            register1: ((opcode & 0x0F00) >> 8) as usize,
            register2: ((opcode & 0x00F0) >> 4) as usize,
        });
    }
    // 0x6rnn: Set register Vr to nn
    else if opcode & 0xF000 == 0x6000 {
        return Some(Opcode::SetRegister {
            register: ((opcode & 0x0F00) >> 8) as usize,
            value: (opcode & 0x00FF) as u8,
        })
    }
    // 0x7rnn: Add value to register
    else if opcode & 0xF000 == 0x7000 {
        return Some(Opcode::AddConstant {
            register: ((opcode & 0x0F00) >> 8) as usize,
            value: (opcode & 0x00FF) as u8,
        })
    }
    // 0x8xy0: Set register Vx's value to register Vy's value
    else if opcode & 0xF00F == 0x8000 {
        return Some(Opcode::CopyRegister {
            target: ((opcode & 0x0F00) >> 8) as usize,
            source: ((opcode & 0x00F0) >> 4) as usize,
        })
    }
    // 0x8xy1: Bitwise OR on Vx and Vy; result stored in Vx
    else if opcode & 0xF00F == 0x8001 {
        return Some(Opcode::BitOr {
            target: ((opcode & 0x0F00) >> 8) as usize,
            other: ((opcode & 0x00F0) >> 4) as usize,
        })
    }
    // 0x8xy2: Bitwise AND on Vx and Vy; result stored in Vx
    else if opcode & 0xF00F == 0x8002 {
        return Some(Opcode::BitAnd {
            target: ((opcode & 0x0F00) >> 8) as usize,
            other: ((opcode & 0x00F0) >> 4) as usize,
        })
    }
    // 0x8xy3: Bitwise XOR on Vx and Vy; result stored in Vx
    else if opcode & 0xF00F == 0x8003 {
        return Some(Opcode::BitXor {
            target: ((opcode & 0x0F00) >> 8) as usize,
            other: ((opcode & 0x00F0) >> 4) as usize,
        })
    }
    // 0x8xy4: Add Vy to Vx; set VF to 1 if carry, otherwise 0
    else if opcode & 0xF00F == 0x8004 {
        return Some(Opcode::AddRegister {
            target: ((opcode & 0x0F00) >> 8) as usize,
            other: ((opcode & 0x00F0) >> 4) as usize,
        })
    }
    // 0x8xy5: Subtract Vy from Vx; set VF to 1 if borrow, otherwise 0
    else if opcode & 0xF00F == 0x8005 {
        return Some(Opcode::SubtractRegister {
            target: ((opcode & 0x0F00) >> 8) as usize,
            other: ((opcode & 0x00F0) >> 4) as usize,
        })
    }
    // 0x8xy6: Shift Vy right by one, store result in Vx, set VF to least sig. bit of Vy *before* shift
    else if opcode & 0xF00F == 0x8006 {
        return Some(Opcode::RightShift {
            target: ((opcode & 0x0F00) >> 8) as usize,
            source: ((opcode & 0x00F0) >> 4) as usize,
        })
    }
    // 0x8xy7: Subtract Vx from Vy, store result in Vx, set VF to 1 if borrow, otherwise 0
    else if opcode & 0xF00F == 0x8007 {
        return Some(Opcode::AltSubtractRegister {
            target: ((opcode & 0x0F00) >> 8) as usize,
            other: ((opcode & 0x00F0) >> 4) as usize,
        })
    }
    // 0x8xy8: Shift Vy left by one, store result in Vx, set VF to most sig. bit of Vy *before* shift
    else if opcode & 0xF00F == 0x8008 {
        return Some(Opcode::LeftShift {
            target: ((opcode & 0x0F00) >> 8) as usize,
            source: ((opcode & 0x00F0) >> 4) as usize,
        })
    }
    // 0x9xy: Skip if registers are not equal
    else if opcode & 0xF000 == 0x9000 {
        return Some(Opcode::SkipIfRegistersNotEqual {
            register1: ((opcode & 0x0F00) >> 8) as usize,
            register2: ((opcode & 0x00F0) >> 4) as usize,
        })
    }
    // 0xAnnn: Set index register
    else if opcode & 0xF000 == 0xA000 {
        return Some(Opcode::SetIndexRegister { value: opcode & 0x0FFF });
    }
    // 0xBnnn: Offset jump to address nnn + V0
    else if opcode & 0xF000 == 0xB000 {
        return Some(Opcode::OffsetJump { address: opcode & 0xFFF });
    }
    // 0xCxnn: Store the bitwise AND of a random u8 and nn in Vx
    else if opcode & 0xF000 == 0xC000 {
        return Some(Opcode::Rand {
            mask: (opcode & 0x00FF) as u8,
            register: ((opcode & 0x0F00) >> 8) as usize,
        })
    }
    // 0xDxyn: Display sprite (location determined by index_register) at coord Vx, Vy and height n.
    else if opcode & 0xF000 == 0xD000 {
        return Some(Opcode::Display {
            x: ((opcode & 0x0F00) >> 8) as usize,
            y: ((opcode & 0x00F0) >> 4) as usize,
            height: (opcode & 0x000F) as u8,
        })
    }
    // 0xEx9E: Skip if key stored in Vx is pressed
    else if opcode & 0xF0FF == 0xE09E {
        return Some(Opcode::SkipIfKeyPressed {
            key: ((opcode & 0x0F00) >> 8) as usize,
        })
    }
    // 0xExA1: Skip if key stored in Vx is not pressed
    else if opcode & 0xF0FF == 0xE0A1 {
        return Some(Opcode::SkipIfKeyNotPressed {
            key: ((opcode & 0x0F00) >> 8) as usize,
        })
    }
    // 0xFx07: Get delay timer value and store in Vx
    else if opcode & 0xF0FF == 0xF007 {
        return Some(Opcode::GetDelayTimer {
            register: ((opcode & 0x0F00) >> 8) as usize,
        })
    }
    // 0xFx0A: Block until a key is pressed; store pressed key in Vx
    else if opcode & 0xF0FF == 0xF00A {
        return Some(Opcode::AwaitKeypress {
            register: ((opcode & 0x0F00) >> 8) as usize,
        })
    }
    // 0xFx15: Set delay timer to Vx
    else if opcode & 0xF0FF == 0xF015 {
        return Some(Opcode::SetDelayTimer {
            value: ((opcode & 0x0F00) >> 8) as usize,
        })
    }
    // 0xFx18: Set sound timer to Vx
    else if opcode & 0xF0FF == 0xF018 {
        return Some(Opcode::SetSoundTimer {
            value: ((opcode & 0x0F00) >> 8) as usize,
        })
    }
    // 0xFx1E: Increment index_register by Vx
    else if opcode & 0xF0FF == 0xF01E {
        return Some(Opcode::IncrementIndexRegister {
            register: ((opcode & 0x0F00) >> 8) as usize,
        })
    }
    // 0xFx29: Set index_register to the index of a font glyph
    else if opcode & 0xF0FF == 0xF029 {
        return Some(Opcode::SetIndexToFont {
            register: ((opcode & 0x0F00) >> 8) as usize,
        })
    }
    // 0xFx33: Store binary-coded repr. of Vx in memory, starting at index_register
    else if opcode & 0xF0FF == 0xF033 {
        return Some(Opcode::StoreDecimal {
            register: ((opcode & 0x0F00) >> 8) as usize,
        })
    }
    // 0xFx55: Dump registers to memory
    else if opcode & 0xF0FF == 0xF055 {
        return Some(Opcode::MemDump {
            max_register: ((opcode & 0x0F00) >> 8) as usize,
        })
    }
    // 0xFx66: Load registers from memory
    else if opcode & 0xF0FF == 0xF065 {
        return Some(Opcode::MemLoad {
            max_register: ((opcode & 0x0F00) >> 8) as usize,
        })
    }

    None
}

pub struct Chip8 {
    pub memory: [u8; MEM_SIZE],
    pub registers: [u8; 16],
    pub index_register: u16,
    pub program_counter: u16,
    // false -> black
    // true -> white
    pub gfx_memory: [bool; GFX_SIZE_X * GFX_SIZE_Y],
    pub delay_timer: u8,
    pub sound_timer: u8,
    pub stack: [u8; 16],
    pub stack_pointer: u8,
    pub keys: [bool; 16],
}

impl Chip8 {
    pub fn new() -> Chip8 {
        Chip8 {
            memory: [0; MEM_SIZE],
            registers: [0; 16],
            index_register: 0,
            program_counter: 0,
            gfx_memory: [false; GFX_SIZE_X * GFX_SIZE_Y],
            delay_timer: 0,
            sound_timer: 0,
            stack: [0; 16],
            stack_pointer: 0,
            keys: [false; 16],
        }
    }

    fn execute_opcode(&mut self, opcode: Opcode) {
        match opcode {
            Opcode::Jump { address } => self.program_counter = address,
            Opcode::SkipIfEqual { register, value } => {
                if register > 15 {
                    panic!("Register index out of range: {} > 15", register);
                }
                
                let register_value = self.registers[register];

                if register_value == value {
                    // Skip the next instruction
                    self.program_counter += 2;
                }
            },
            Opcode::SkipIfNotEqual { register, value } => {
                if register > 15 {
                    panic!("Register index out of range: {} > 15", register);
                }
                
                let register_value = self.registers[register];

                if register_value != value {
                    // Skip the next instruction
                    self.program_counter += 2;
                }
            },
            Opcode::SkipIfRegistersEqual { register1, register2 } => {
                if register1 > 15 {
                    panic!("Register index out of range: {} > 15", register1);
                }

                if register2 > 15 {
                    panic!("Register index out of range: {} > 15", register2);
                }

                let r1_value = self.registers[register1];
                let r2_value = self.registers[register2];
                
                if r1_value == r2_value {
                    self.program_counter += 2;
                }
            },
            Opcode::SetRegister { register, value } => {
                if register > 15 {
                    panic!("Register index out of range: {} > 15", register);
                }

                self.registers[register] = value;
            },
            Opcode::AddConstant { register, value } => {
                if register > 15 {
                    panic!("Register index out of range: {} > 15", register);
                }

                let register_value = self.registers[register];
                // Unsure: Is wrapping_add or clamping at max the correct behavior?
                let sum = register_value.wrapping_add(value);
                self.registers[register] = sum;
            },
            Opcode::CopyRegister { target, source } => {
                if target > 15 {
                    panic!("Register index out of range: {} > 15", target);
                }

                if source > 15 {
                    panic!("Register index out of range: {} > 15", source);
                }

                self.registers[target] = self.registers[source];
            },
            Opcode::BitOr { target, other } => {
                if target > 15 {
                    panic!("Register index out of range: {} > 15", target);
                }

                if other > 15 {
                    panic!("Register index out of range: {} > 15", other);
                }

                self.registers[target] = self.registers[target] | self.registers[other];
            },
            Opcode::BitAnd { target, other } => {
                if target > 15 {
                    panic!("Register index out of range: {} > 15", target);
                }

                if other > 15 {
                    panic!("Register index out of range: {} > 15", other);
                }

                self.registers[target] = self.registers[target] & self.registers[other];
            },
            Opcode::BitXor { target, other } => {
                if target > 15 {
                    panic!("Register index out of range: {} > 15", target);
                }

                if other > 15 {
                    panic!("Register index out of range: {} > 15", other);
                }

                self.registers[target] = self.registers[target] ^ self.registers[other];
            },
            Opcode::AddRegister { target, other } => {
                if target > 15 {
                    panic!("Register index out of range: {} > 15", target);
                }

                if other > 15 {
                    panic!("Register index out of range: {} > 15", other);
                }

                let target_value = self.registers[target];
                let other_value = self.registers[other];

                // Unsigned binary arithmetic; overflow means a carry.
                if let Some(result) = target_value.checked_add(other_value) {
                    // No carry.
                    self.registers[target] = result;
                    self.registers[0xF] = 0;
                }
                else {
                    // Carry occurred.
                    self.registers[target] = target_value.wrapping_add(other_value);
                    self.registers[0xF] = 1;
                }
            },
            Opcode::SubtractRegister { target, other } => {
                if target > 15 {
                    panic!("Register index out of range: {} > 15", target);
                }

                if other > 15 {
                    panic!("Register index out of range: {} > 15", other);
                }

                let target_value = self.registers[target];
                let other_value = self.registers[other];

                // Unsigned binary arithmetic; underflow means a borrow.
                if let Some(result) = target_value.checked_sub(other_value) {
                    // No borrow.
                    self.registers[target] = result;
                    self.registers[0xF] = 0;
                }
                else {
                    // Borrow occurred.
                    self.registers[target] = target_value.wrapping_sub(other_value);
                    self.registers[0xF] = 1;
                }
            },
            Opcode::SetIndexRegister { value } => self.index_register = value,
            _ => panic!("unimplemented opcode {:?}", opcode),
        }
    }

    fn process_next_opcode(&mut self) {
        // Fetch latest opcode.
        // Opcode is located in memory at the program_counter index
        // Is a u16 value - fetch two u8s and merge them.
        let opcode_upper = self.memory[self.program_counter as usize] as u16;
        let opcode_lower = self.memory[self.program_counter as usize + 1] as u16;
        // Combine them: shift opcode_upper into the upper 8 bits of the u16
        // (remember, opcode_upper is only 8 significant bits - it was originally a u8)
        // Then binary-or the lower value into the space that opcode_upper used to occupy
        let opcode = opcode_upper << 8 | opcode_lower;

        // Increment the program counter so we move past the instruction
        // Do this *here* so that if program_counter is changed, this change is overwritten
        self.program_counter += 2;

        // decode_opcode can return None; in the interests of making testing, etc. easier
        // this is not handled at all.
        if let Some(decoded_opcode) = decode_opcode(opcode) {
            self.execute_opcode(decoded_opcode);
        }
    }

    /// Steps the chip8 VM.
    /// This does two things (in order):
    /// * Decodes and executes the current opcode
    /// * Decrements the delay and sound timers
    pub fn step(&mut self) {
        // Process the current instruction
        self.process_next_opcode();

        // Decrement timers
        if self.delay_timer > 0 {
            self.delay_timer -= 1;
        }

        if self.sound_timer > 0 {
            self.sound_timer -= 1;
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn step_decrements_timers() {
        let mut vm = Chip8::new();
        vm.delay_timer = 30;
        vm.sound_timer = 19;
        vm.step();
        assert_eq!(vm.delay_timer, 29);
        assert_eq!(vm.sound_timer, 18);

        // Make sure we don't panic due to subtract w/ overflow:
        vm.sound_timer = 0;
        vm.step();
        assert_eq!(vm.sound_timer, 0);
    }

    mod opcode_executing {
        use super::*;

        #[test]
        fn jump() {
            let mut vm = Chip8::new();
            vm.execute_opcode(Opcode::Jump { address: 0x09DE });
            assert_eq!(vm.program_counter, 0x09DE);
        }

        #[test]
        fn set_idx_reg() {
            let mut vm = Chip8::new();
            vm.execute_opcode(Opcode::SetIndexRegister { value: 0x0387 });
            assert_eq!(vm.index_register, 0x0387);
        }

        #[test]
        fn skip_if_eq_const() {
            let mut vm = Chip8::new();
            vm.execute_opcode(Opcode::SkipIfEqual { register: 0xA, value: 0x32 });
            // Scenario 1: register A is 0, but we expect 0x32.
            // This will not skip the next instruction. The program
            // counter can thus be expected to be 0x0000.
            assert_eq!(vm.program_counter, 0x0000);
            
            // Reset the program counter.
            vm.program_counter = 0x0000;
            // Scenario 2: register A is now 0x32, and we expect
            // 0x32. This *will* skip the next instruction. The
            // program counter should be 0x0002.
            vm.registers[0x0A] = 0x32;
            vm.execute_opcode(Opcode::SkipIfEqual { register: 0xA, value: 0x32 });
            assert_eq!(vm.program_counter, 0x0002);
        }

        #[test]
        fn skip_if_not_eq_const() {
            // This test is the reverse of skip_if_eq_const.
            let mut vm = Chip8::new();
            vm.execute_opcode(Opcode::SkipIfNotEqual { register: 0xA, value: 0x32 });
            // Scenario 1: register A is 0, but we expect 0x32.
            // This will skip the next instruction. The program
            // counter can thus be expected to be 0x0002.
            assert_eq!(vm.program_counter, 0x0002);
            
            // Reset the program counter.
            vm.program_counter = 0x0000;
            // Scenario 2: register A is now 0x32, and we expect
            // 0x32. This will not skip the next instruction. The
            // program counter should be 0x0000.
            vm.registers[0x0A] = 0x32;
            vm.execute_opcode(Opcode::SkipIfNotEqual { register: 0xA, value: 0x32 });
            assert_eq!(vm.program_counter, 0x0000);
        }

        #[test]
        fn skip_if_registers_eq() {
            let mut vm = Chip8::new();
            vm.registers[0xA] = 0x0;
            vm.registers[0xB] = 0xF;
            vm.execute_opcode(Opcode::SkipIfRegistersEqual { register1: 0xA, register2: 0xB });
            // Scenario 1: register A is 0 and register B is 0x0F.
            // The next instruction should not be skipped; program_counter
            // should be 0x0000.
            assert_eq!(vm.program_counter, 0x0000);
            
            // Reset the program counter.
            vm.program_counter = 0x0000;
            // Scenario 2: register A is now 0x0F, the same as
            // register B. This *will* skip the next instruction - the
            // program counter should be 0x0002.
            vm.registers[0xA] = 0x0F;
            vm.execute_opcode(Opcode::SkipIfRegistersEqual { register1: 0xA, register2: 0xB });
            assert_eq!(vm.program_counter, 0x0002);
        }

        #[test]
        fn set_register() {
            let mut vm = Chip8::new();
            vm.execute_opcode(Opcode::SetRegister { register: 0x0, value: 0xFF });
            assert_eq!(vm.registers[0], 0xFF);
        }

        #[test]
        fn add_const() {
            let mut vm = Chip8::new();
            vm.registers[0] = 0x13;
            vm.execute_opcode(Opcode::AddConstant { register: 0, value: 0x23 });
            assert_eq!(vm.registers[0], 0x23 + 0x13);
        }

        #[test]
        fn copy_register() {
            let mut vm = Chip8::new();
            vm.registers[0] = 0x13;
            vm.registers[1] = 0xFF;
            vm.execute_opcode(Opcode::CopyRegister { source: 1, target: 0 });
            assert_eq!(vm.registers[0], 0xFF);
        }

        #[test]
        fn bit_or() {
            let mut vm = Chip8::new();
            vm.registers[0] = 0x13;
            vm.registers[1] = 0xC4;
            vm.execute_opcode(Opcode::BitOr { target: 0, other: 1 });
            assert_eq!(vm.registers[0], 0x13 | 0xC4);
        }

        #[test]
        fn bit_and() {
            let mut vm = Chip8::new();
            vm.registers[0] = 0x13;
            vm.registers[1] = 0xC4;
            vm.execute_opcode(Opcode::BitAnd { target: 0, other: 1 });
            assert_eq!(vm.registers[0], 0x13 & 0xC4);
        }

        #[test]
        fn bit_xor() {
            let mut vm = Chip8::new();
            vm.registers[0] = 0x13;
            vm.registers[1] = 0xC4;
            vm.execute_opcode(Opcode::BitXor { target: 0, other: 1 });
            assert_eq!(vm.registers[0], 0x13 ^ 0xC4);
        }

        #[test]
        fn register_add() {
            let mut vm = Chip8::new();
            vm.registers[0] = 0x13;
            vm.registers[1] = 0xC4;
            vm.registers[2] = 0xFF;
            vm.registers[3] = 0xD9;
            vm.execute_opcode(Opcode::AddRegister { target: 0, other: 1 });
            assert_eq!(vm.registers[0], 0x13 + 0xC4);
            assert_eq!(vm.registers[0xF], 0);
            vm.execute_opcode(Opcode::AddRegister { target: 2, other: 3 });
            assert_eq!(vm.registers[2], 0xD8);
            assert_eq!(vm.registers[0xF], 1);
        }

        #[test]
        fn register_sub() {
            let mut vm = Chip8::new();
            vm.registers[0] = 0x13;
            vm.registers[1] = 0xC4;
            vm.registers[2] = 0x13;
            vm.registers[3] = 0x11;
            vm.execute_opcode(Opcode::SubtractRegister { target: 0, other: 1 });
            assert_eq!(vm.registers[0], 0x4F);
            assert_eq!(vm.registers[0xF], 1);
            vm.execute_opcode(Opcode::SubtractRegister { target: 2, other: 3 });
            assert_eq!(vm.registers[2], 0x02);
            assert_eq!(vm.registers[0xF], 0);
        }
    }

    mod opcode_decoding {
        use super::*;

        macro_rules! decodes_to {
            ($opcode:expr => $expected:expr) => (
                {
                    match decode_opcode($opcode) {
                        Some(decoded) => assert_eq!(decoded, $expected, "expected {:?} to decode to {:?}, but got {:?}", $opcode, $expected, decoded),
                        None => panic!("couldn't decode opcode {}", $opcode),
                    }
                }
            );
            ($opcode:expr => $expected:expr, $($chain_opcode:expr => $chain_expected:expr),+$(,)*) => {{
                decodes_to!($opcode => $expected);
                decodes_to! { $($chain_opcode => $chain_expected),+ };
            }}
        }

        #[test]
        fn test_decoding() {
            decodes_to! {
                0x19DE => Opcode::Jump { address: 0x09DE },
                0x27A9 => Opcode::Call { address: 0x07A9 },
                0x342F => Opcode::SkipIfEqual { register: 0x4, value: 0x2F },
                0x461F => Opcode::SkipIfNotEqual { register: 0x6, value: 0x1F },
                0x5A30 => Opcode::SkipIfRegistersEqual { register1: 0xA, register2: 0x3 },
                0x6E72 => Opcode::SetRegister { register: 0xE, value: 0x72 },
                0x72EE => Opcode::AddConstant { register: 0x2, value: 0xEE },
                0x8370 => Opcode::CopyRegister { target: 0x3, source: 0x7 },
                0x8371 => Opcode::BitOr { target: 0x3, other: 0x7 },
                0x8372 => Opcode::BitAnd { target: 0x3, other: 0x7 },
                0x8373 => Opcode::BitXor { target: 0x3, other: 0x7 },
                0x8374 => Opcode::AddRegister { target: 0x3, other: 0x7 },
                0x8375 => Opcode::SubtractRegister { target: 0x3, other: 0x7 },
                0x8376 => Opcode::RightShift { target: 0x3, source: 0x7 },
                0x8377 => Opcode::AltSubtractRegister { target: 0x3, other: 0x7 },
                0x8378 => Opcode::LeftShift { target: 0x3, source: 0x7 },
                0x9370 => Opcode::SkipIfRegistersNotEqual { register1: 0x3, register2: 0x7 },
                0xA428 => Opcode::SetIndexRegister { value: 0x0428 },
                0xB3FC => Opcode::OffsetJump { address: 0x03FC },
                0xC1F0 => Opcode::Rand { register: 0x1, mask: 0xF0 },
                0xD01E => Opcode::Display { x: 0x0, y: 0x1, height: 0xE },
                0xE29E => Opcode::SkipIfKeyPressed { key: 0x2 },
                0xE2A1 => Opcode::SkipIfKeyNotPressed { key: 0x2 },
                0xF21E => Opcode::IncrementIndexRegister { register: 0x2 },
                0xF307 => Opcode::GetDelayTimer { register: 0x3 },
                0xF829 => Opcode::SetIndexToFont { register: 0x8 },
                0xF833 => Opcode::StoreDecimal { register: 0x8 },
                0xF855 => Opcode::MemDump { max_register: 0x8 },
                0xF90A => Opcode::AwaitKeypress { register: 0x9 },
                0xF965 => Opcode::MemLoad { max_register: 0x9 },
                0xFE15 => Opcode::SetDelayTimer { value: 0xE },
                0xFE18 => Opcode::SetSoundTimer { value: 0xE },
            }
        }
    }
}