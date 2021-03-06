#[derive(PartialEq)]
#[derive(Debug)]
pub struct Opcode {
    pub raw: u16,
    pub x: u8,
    pub y: u8,
    pub n: u8,
    pub nnn: u16,
    pub kk: u8,
}

#[derive(PartialEq)]
#[derive(Debug)]
pub enum Instruction {
    End(Opcode),
    /// Clear screen
    I00E0(Opcode),
    /// Return subroutine. i.e: pc = stack.pop()
    I00EE(Opcode),

    /// Jump to instruction ~ pc = nnn
    I1NNN(Opcode),

    /// Create subroutine ~ stack.push(pc) & pc = nnn
    I2NNN(Opcode),

    /// Skip next instruction if x = nn
    I3XNN(Opcode),

    /// Skip next instruction if x != nn
    I4XNN(Opcode),

    /// Skip next instruction if x == y
    I5XY0(Opcode),

    /// Set v[x] = nn
    I6XNN(Opcode),

    /// Add nn to v[x] ~ v[x] += nn
    I7XNN(Opcode),

    /// Set v[x] = v[y]
    I8XY0(Opcode),

    /// v[x] = v[x] OR v[y]
    I8XY1(Opcode), 

    /// v[x] = v[x] AND v[y]
    I8XY2(Opcode), 

    /// v[x] = v[x] XOR v[y]
    I8XY3(Opcode), 

    /// v[x] = v[x] + v[y], set v[f] = 1 if overflow
    I8XY4(Opcode), 

    /// v[x] = v[x] - v[y], set v[f] = 0 if underflow
    I8XY5(Opcode), 

    /// v[x] = v[y] - v[x], set v[f] = 0 if underflow
    I8XY7(Opcode), 

    /// if old_shift: v[x] = v[y], v[x] >> 1 
    I8XY6(Opcode), 

    /// if old_shift: v[x] = v[y], v[x] << 1 
    I8XYE(Opcode), 

    /// Skip next instruction if x != y
    I9XY0(Opcode),

    /// Set vi = nnn
    IANNN(Opcode),

    /// Jump to address nnn + v[0]
    IBNNN(Opcode),

    /// vx = rand() & nn
    ICXNN(Opcode),

    /// vi += v[x], if vi > 0xFFF, set v[f] =1
    IFX1E(Opcode),

    /// Draw
    IDXYN(Opcode),
}

impl Instruction {
    pub fn from_raw_opcode(raw_opcode: u16) -> Result<Instruction, String> {
        let opcode = Opcode::new(raw_opcode);
        if raw_opcode == 0x00 {
            return Ok(Instruction::End(opcode));
        }
        if raw_opcode == 0x00E0 {
            return Ok(Instruction::I00E0(opcode));
        }
        if raw_opcode == 0x00EE {
            return Ok(Instruction::I00EE(opcode));
        }
        if raw_opcode >> 12 == 0x1 {
            return Ok(Instruction::I1NNN(opcode));
        }
        if raw_opcode >> 12 == 0x2 {
            return Ok(Instruction::I2NNN(opcode));
        }
        if raw_opcode >> 12 == 0x3 {
            return Ok(Instruction::I3XNN(opcode));
        }
        if raw_opcode >> 12 == 0x4 {
            return Ok(Instruction::I4XNN(opcode));
        }
        if raw_opcode >> 12 == 0x5 {
            return Ok(Instruction::I5XY0(opcode));
        }
        if raw_opcode >> 12 == 0x6 {
            return Ok(Instruction::I6XNN(opcode));
        }
        if raw_opcode >> 12 == 0x7 {
            return Ok(Instruction::I7XNN(opcode));
        }
        if raw_opcode >> 12 == 0x8 {
            let last_nible = raw_opcode & 0xF;
            if last_nible == 0 {
                return Ok(Instruction::I8XY0(opcode));
            }
            if last_nible == 1 {
                return Ok(Instruction::I8XY1(opcode));
            }
            if last_nible == 2 {
                return Ok(Instruction::I8XY2(opcode));
            }
            if last_nible == 3 {
                return Ok(Instruction::I8XY3(opcode));
            }
            if last_nible == 4 {
                return Ok(Instruction::I8XY4(opcode));
            }
            if last_nible == 5 {
                return Ok(Instruction::I8XY5(opcode))
            }
            if last_nible == 6 {
                return Ok(Instruction::I8XY6(opcode));
            }
            if last_nible == 7 {
                return Ok(Instruction::I8XY7(opcode))
            }
            if last_nible == 0xE {
                return Ok(Instruction::I8XYE(opcode))
            }
        }
        if raw_opcode >> 12 == 0xA {
            return Ok(Instruction::IANNN(opcode));
        }
        if raw_opcode >> 12 == 0x9 {
            return Ok(Instruction::I9XY0(opcode));
        }
        if raw_opcode >> 12 == 0xD {
            return Ok(Instruction::IDXYN(opcode))
        }

        Err(String::from("Cannot decode instruction"))
    }
}

impl Opcode {
    pub fn new(raw: u16) -> Opcode {
        let x = take_param_x(raw);
        let y = take_param_y(raw);
        let n = take_param_n(raw);
        let nnn = take_param_nnn(raw);
        let kk = take_param_kk(raw);

        Opcode {raw, x, y, n, nnn, kk}
    }
}

impl PartialEq<u16> for Opcode {
    fn eq(&self, raw: &u16) -> bool {
        self.raw == *raw
    }
}

fn take_param_n(opcode: u16) -> u8 {
    (opcode & 0x000F) as u8
}

fn take_param_nnn(opcode: u16) -> u16 {
    opcode & 0x0FFF
}

fn take_param_kk(opcode: u16) -> u8 {
    (opcode & 0x00FF) as u8
}

fn take_param_x(opcode: u16) -> u8 {
    ((opcode & 0x0F00) >> 8) as u8
}

fn take_param_y(opcode: u16) -> u8 {
    ((opcode & 0x00F0) >> 4) as u8
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_take_param_n() {
        assert_eq!(take_param_n(0x8C74), 0x04);
        assert_eq!(take_param_n(0x8B74), 0x04);
    }

    #[test]
    fn test_take_param_nnn() {
        assert_eq!(take_param_nnn(0x8C74), 0x0C74);
        assert_eq!(take_param_nnn(0xAB12), 0x0B12);
    }

    #[test]
    fn test_take_param_kk() {
        assert_eq!(take_param_kk(0x8C74), 0x74);
        assert_eq!(take_param_kk(0xAB12), 0x12);
    }

    #[test]
    fn test_take_param_x() {
        assert_eq!(take_param_x(0x8C74), 0x0C);
        assert_eq!(take_param_x(0xAB12), 0x0B);
    }

    #[test]
    fn test_take_param_y() {
        assert_eq!(take_param_y(0x8C74), 0x07);
        assert_eq!(take_param_y(0xAB12), 0x01);
    }

    #[test]
    fn test_instruction_from_raw_code() {
        assert_eq!(Instruction::from_raw_opcode(0xE0).unwrap(), Instruction::I00E0(Opcode::new(0xE0)));
        assert_eq!(Instruction::from_raw_opcode(0xEE).unwrap(), Instruction::I00EE(Opcode::new(0xEE)));
        assert_eq!(Instruction::from_raw_opcode(0x1234).unwrap(), Instruction::I1NNN(Opcode::new(0x1234)));
        assert_eq!(Instruction::from_raw_opcode(0x2234).unwrap(), Instruction::I2NNN(Opcode::new(0x2234)));
        assert_eq!(Instruction::from_raw_opcode(0x3234).unwrap(), Instruction::I3XNN(Opcode::new(0x3234)));
        assert_eq!(Instruction::from_raw_opcode(0x4234).unwrap(), Instruction::I4XNN(Opcode::new(0x4234)));
        assert_eq!(Instruction::from_raw_opcode(0x5230).unwrap(), Instruction::I5XY0(Opcode::new(0x5230)));
        assert_eq!(Instruction::from_raw_opcode(0x8230).unwrap(), Instruction::I8XY0(Opcode::new(0x8230)));
        assert_eq!(Instruction::from_raw_opcode(0x8231).unwrap(), Instruction::I8XY1(Opcode::new(0x8231)));
        assert_eq!(Instruction::from_raw_opcode(0x8232).unwrap(), Instruction::I8XY2(Opcode::new(0x8232)));
        assert_eq!(Instruction::from_raw_opcode(0x8233).unwrap(), Instruction::I8XY3(Opcode::new(0x8233)));
        assert_eq!(Instruction::from_raw_opcode(0x8234).unwrap(), Instruction::I8XY4(Opcode::new(0x8234)));
        assert_eq!(Instruction::from_raw_opcode(0x8235).unwrap(), Instruction::I8XY5(Opcode::new(0x8235)));
        assert_eq!(Instruction::from_raw_opcode(0x8236).unwrap(), Instruction::I8XY6(Opcode::new(0x8236)));
        assert_eq!(Instruction::from_raw_opcode(0x8237).unwrap(), Instruction::I8XY7(Opcode::new(0x8237)));
        assert_eq!(Instruction::from_raw_opcode(0x823E).unwrap(), Instruction::I8XYE(Opcode::new(0x823E)));
    }

    #[test]
    fn test_opcode() {
        let op = Opcode::new(0xFABC);
        assert_eq!(op.kk, 0xBC);
        assert_eq!(op.nnn, 0xABC);
        assert_eq!(op.x, 0xA);
        assert_eq!(op.y, 0xB);
        assert_eq!(op.n, 0xC);
    }
}