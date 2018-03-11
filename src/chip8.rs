mod chip8 {
    const MEM_SIZE: usize = 0x1000;
    const GFX_SIZE_X: usize = 64;
    const GFX_SIZE_Y: usize = 32;

    #[derive(Debug)]
    pub enum Opcode {
        // Not defined: opcode 0NNN (call RCA 1802 program).
        ClearDisplay,
        Return,
        // Jump to memory address
        Jump { address: u16 },
        // Call subroutine at memory address
        Call { address: u16 },
        // Skip if register value == constant
        SkipIfEqual { register: u8, value: u8 },
        // Skip if register value != constant
        SkipIfNotEqual { register: u8, value: u8 },
        // Skip if register 1 value == register 2 value
        SkipIfRegistersEqual { register1: u8, register2: u8 },
        // Set register value to a constant
        SetRegister { register: u8, value: u8 },
        // Add a constant to a register value
        AddConstant { register: u8, value: u8 },
        // Assign a register's value to another register
        Assign { source: u8, target: u8 },
        // Sets the target's value to target | other
        BitOr { target: u8, other: u8 },
        // Sets the target's value to target & other
        BitAnd { target: u8, other: u8 },
        // Sets the target's value to target ^ other
        BitXor { target: u8, other: u8 },
        // Add a register's value to another register's value. VF is set to 0 if no carry and 1 if carry.
        AddRegister { target: u8, other: u8 },
        // Subtract a register's value from another register's value. VF is set to 0 if borrow and 1 if no borrow.
        SubtractRegister { target: u8, other:u8 },
        // 0xAnnn: Sets the index register to a value
        SetIndexRegister { value: u16 },
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
            return Some(Opcode::SkipIfEqual { register: ((opcode & 0x0F00) >> 8) as u8, value: (opcode & 0x00FF) as u8 });
        }
        // 0xAnnn: Set index register
        else if opcode & 0xF000 == 0xA000 {
            return Some(Opcode::SetIndexRegister { value: opcode & 0x0FFF });
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

            // decode_opcode can return None; in the interests of making testing, etc. easier
            // this is not handled at all.
            if let Some(decoded_opcode) = decode_opcode(opcode) {
                match decoded_opcode {
                    Opcode::Jump { address } => self.program_counter = address,
                    Opcode::SetIndexRegister { value } => self.index_register = value,
                    _ => panic!("unimplemented opcode {:?} (raw: {})", decoded_opcode, opcode),
                }
            }

            // Increment the program counter so we move past the instruction
            self.program_counter += 2;
        }

        /// Steps the chip8 VM.
        /// This does two things (in order):
        /// * Decodes and executes the current opcode
        /// * Decrements the delay and sound timers
        pub fn step(&mut self) {
            // Process the current instruction
            self.process_next_opcode();

            // Decrement timers
            self.delay_timer -= 1;
            self.sound_timer -= 1;
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
        }

        mod opcode_decoding {
            use super::*;

            #[test]
            fn jump() {
                let opcode = 0x19DE;
                let decoded_opcode = decode_opcode(opcode);

                match decoded_opcode {
                    Some(Opcode::Jump { address }) => assert_eq!(address, 0x09DE),
                    _ => panic!("decoded wrong opcode {:?}", decoded_opcode)
                }
            }

            #[test]
            fn call() {
                let opcode = 0x27A9;
                let decoded_opcode = decode_opcode(opcode);

                match decoded_opcode {
                    Some(Opcode::Call { address }) => assert_eq!(address, 0x07A9),
                    _ => panic!("decoded wrong opcode {:?}", decoded_opcode)
                }
            }

            #[test]
            fn skip_if_eq_const() {
                let opcode = 0x342F;
                let decoded_opcode = decode_opcode(opcode);

                match decoded_opcode {
                    Some(Opcode::SkipIfEqual { register, value }) => {
                        assert_eq!(register, 0x04);
                        assert_eq!(value, 0x2F);
                    },
                    _ => panic!("decoded wrong opcode {:?}", decoded_opcode)
                }
            }

            #[test]
            fn set_idx_reg() {
                let opcode = 0xA428;
                let decoded_opcode = decode_opcode(opcode);

                match decoded_opcode {
                    Some(Opcode::SetIndexRegister { value }) => assert_eq!(value, 0x0428),
                    _ => panic!("decoded wrong opcode {:?}", decoded_opcode)
                }
            }
        }
    }
}
