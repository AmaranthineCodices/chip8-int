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

            // Increment the program counter so we move past the instruction
            // Do this *here* so that if program_counter is changed, this change is overwritten
            self.program_counter += 2;

            // decode_opcode can return None; in the interests of making testing, etc. easier
            // this is not handled at all.
            if let Some(decoded_opcode) = decode_opcode(opcode) {
                match decoded_opcode {
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
                    Opcode::SetIndexRegister { value } => self.index_register = value,
                    _ => panic!("unimplemented opcode {:?} (raw: {})", decoded_opcode, opcode),
                }
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
                vm.memory[0] = 0x19;
                vm.memory[1] = 0xDE;
                vm.program_counter = 0x0000;
                vm.step();

                assert_eq!(vm.program_counter, 0x09DE);
            }

            #[test]
            fn set_idx_reg() {
                let mut vm = Chip8::new();
                vm.memory[0] = 0xA3;
                vm.memory[1] = 0x87;
                vm.program_counter = 0x0000;
                vm.step();

                assert_eq!(vm.index_register, 0x0387);
            }

            #[test]
            fn skip_if_eq_const() {
                let mut vm = Chip8::new();
                vm.memory[0] = 0x3A;
                vm.memory[1] = 0x32;
                // Scenario 1: register A is 0, but we expect 0x32.
                // This will not skip the next instruction. The program
                // counter can thus be expected to be 0x0002.
                vm.program_counter = 0x0000;
                vm.step();
                assert_eq!(vm.program_counter, 0x0002);
                
                // Reset the program counter.
                vm.program_counter = 0x0000;
                // Scenario 2: register A is now 0x32, and we expect
                // 0x32. This *will* skip the next instruction. The
                // program counter should be 0x0004.
                vm.registers[0x0A] = 0x32;
                vm.step();
                assert_eq!(vm.program_counter, 0x0004);
            }

            #[test]
            fn skip_if_not_eq_const() {
                // This test is the reverse of skip_if_eq_const.
                let mut vm = Chip8::new();
                vm.memory[0] = 0x4A;
                vm.memory[1] = 0x32;
                // Scenario 1: register A is 0, but we expect 0x32.
                // This will skip the next instruction. The program
                // counter can thus be expected to be 0x0004.
                vm.program_counter = 0x0000;
                vm.step();
                assert_eq!(vm.program_counter, 0x0004);
                
                // Reset the program counter.
                vm.program_counter = 0x0000;
                // Scenario 2: register A is now 0x32, and we expect
                // 0x32. This will not skip the next instruction. The
                // program counter should be 0x0002.
                vm.registers[0x0A] = 0x32;
                vm.step();
                assert_eq!(vm.program_counter, 0x0002);
            }

            #[test]
            fn skip_if_registers_eq() {
                let mut vm = Chip8::new();
                vm.memory[0] = 0x5A;
                vm.memory[1] = 0xB0;
                vm.registers[0x0B] = 0x0F;
                // Scenario 1: register A is 0 and register B is 0x0F.
                // The next instruction should not be skipped; program_counter
                // should be 0x0002.
                vm.program_counter = 0x0000;
                vm.step();
                assert_eq!(vm.program_counter, 0x0002);
                
                // Reset the program counter.
                vm.program_counter = 0x0000;
                // Scenario 2: register A is now 0x0F, the same as
                // register B. This *will* skip the next instruction - the
                // program counter should be 0x0004.
                vm.registers[0x0A] = 0x0F;
                vm.step();
                assert_eq!(vm.program_counter, 0x0004);
            }

            #[test]
            fn set_register() {
                let mut vm = Chip8::new();
                vm.memory[0] = 0x60;
                vm.memory[1] = 0xFF;
                vm.program_counter = 0x0000;
                vm.step();
                assert_eq!(vm.registers[0], 0xFF);
            }
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
            fn skip_if_not_eq_const() {
                let opcode = 0x461F;
                let decoded_opcode = decode_opcode(opcode);

                match decoded_opcode {
                    Some(Opcode::SkipIfNotEqual { register, value }) => {
                        assert_eq!(register, 0x06);
                        assert_eq!(value, 0x1F);
                    },
                    _ => panic!("decoded wrong opcode {:?}", decoded_opcode)
                }
            }

            #[test]
            fn skip_if_registers_eq() {
                let opcode = 0x5A30;
                let decoded_opcode = decode_opcode(opcode);

                match decoded_opcode {
                    Some(Opcode::SkipIfRegistersEqual { register1, register2 }) => {
                        assert_eq!(register1, 0x0A);
                        assert_eq!(register2, 0x03);
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

            #[test]
            fn set_register() {
                let opcode = 0x6E72;
                let decoded_opcode = decode_opcode(opcode);

                match decoded_opcode {
                    Some(Opcode::SetRegister { register, value }) => {
                        assert_eq!(register, 0x0E);
                        assert_eq!(value, 0x72);
                    },
                    _ => panic!("decoded wrong opcode {:?}", decoded_opcode)
                }
            }
        }
    }
}
