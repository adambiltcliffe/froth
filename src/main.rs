use num_enum::{FromPrimitive, IntoPrimitive};

mod builtins;

const ADDR_LATEST: u32 = 0;
const ADDR_BASE: u32 = 4;
const ADDR_STATE: u32 = 8;
const ADDR_HERE: u32 = 12;
const ADDR_WORD_BUFFER: u32 = 16;
const INITIAL_HERE: u32 = 48;

const MAX_EXTEND: u32 = 64;

#[derive(FromPrimitive, IntoPrimitive)]
#[repr(u8)]
enum Op {
    DoColonDef = 0,
    Dup,
    Add,
    Lit,
    Find,
    Exit,
    #[num_enum(default)]
    Unknown,
}

#[derive(Debug)]
enum VMError {
    IllegalAddress,
    UnknownOpcode,
    DataStackUnderflow,
    ReturnStackUnderflow,
    UnalignedAccess,
}

fn align_addr(addr: u32) -> u32 {
    ((addr + 3) / 4) * 4
}

type VMResult<T> = Result<T, VMError>;
type VMSuccess = VMResult<()>;

struct VM {
    memory: Vec<u8>,
    data_stack: Vec<u32>,
    return_stack: Vec<u32>,
    pc: u32,
}

impl VM {
    fn new() -> Self {
        let mut me = Self {
            memory: vec![0; INITIAL_HERE as usize],
            data_stack: Vec::new(),
            return_stack: Vec::new(),
            pc: 0,
        };
        me.write_u32(ADDR_BASE, 10).unwrap();
        me.write_u32(ADDR_HERE, INITIAL_HERE).unwrap();
        me
    }

    fn push_data(&mut self, data: u32) {
        self.data_stack.push(data)
    }

    fn pop_data(&mut self) -> VMResult<u32> {
        self.data_stack.pop().ok_or(VMError::DataStackUnderflow)
    }

    fn push_return(&mut self, addr: u32) {
        self.return_stack.push(addr)
    }

    fn pop_return(&mut self) -> VMResult<u32> {
        self.return_stack.pop().ok_or(VMError::ReturnStackUnderflow)
    }

    fn align(&mut self) -> VMSuccess {
        let mut here = self.read_u32(ADDR_HERE)?;
        here = align_addr(here);
        self.write_u32(ADDR_HERE, here)
    }

    fn read_u8(&self, addr: u32) -> VMResult<u8> {
        self.memory
            .get(addr as usize)
            .copied()
            .ok_or(VMError::IllegalAddress)
    }

    fn write_u8(&mut self, addr: u32, data: u8) -> VMSuccess {
        let addr = addr as usize;
        if addr + 1 > self.memory.len() + MAX_EXTEND as usize {
            return Err(VMError::IllegalAddress);
        }
        if addr + 1 > self.memory.len() {
            self.memory.extend(vec![0; addr + 1 - self.memory.len()]);
        }
        self.memory[addr] = data;
        Ok(())
    }

    fn write_u8_here(&mut self, data: u8) -> VMSuccess {
        let here = self.read_u32(ADDR_HERE)?;
        self.write_u8(here, data)?;
        self.write_u32(ADDR_HERE, here + 1)
    }

    fn read_u32(&self, addr: u32) -> VMResult<u32> {
        let addr = addr as usize;
        if addr % 4 != 0 {
            return Err(VMError::UnalignedAccess);
        }
        if self.memory.len() < addr + 4 {
            return Err(VMError::IllegalAddress);
        }
        Ok(u32::from_le_bytes(
            self.memory[addr..(addr + 4)].try_into().unwrap(),
        ))
    }

    fn write_u32(&mut self, addr: u32, data: u32) -> VMSuccess {
        let addr = addr as usize;
        if addr % 4 != 0 {
            return Err(VMError::UnalignedAccess);
        }
        if addr + 4 > self.memory.len() + MAX_EXTEND as usize {
            return Err(VMError::IllegalAddress);
        }
        if addr + 4 > self.memory.len() {
            self.memory.extend(vec![0; addr + 4 - self.memory.len()]);
        }
        let slice: &mut [u8; 4] = (&mut self.memory[addr..(addr + 4)]).try_into().unwrap();
        *slice = data.to_le_bytes();
        Ok(())
    }

    fn write_u32_here(&mut self, data: u32) -> VMSuccess {
        let here = self.read_u32(ADDR_HERE)?;
        self.write_u32(here, data)?;
        self.write_u32(ADDR_HERE, here + 4)
    }

    fn buffer_word(&mut self, word: &str) {
        let bytes = str_to_bytes(word);
        let mut addr = ADDR_WORD_BUFFER;
        let n = 32.min(bytes.len());
        for b in &bytes[0..n] {
            self.memory[addr as usize] = *b;
            addr += 1;
        }
        self.push_data(ADDR_WORD_BUFFER);
        self.push_data(n as u32);
    }

    fn find_word(&self, addr: u32, len: u8) -> VMResult<u32> {
        let mut search_addr = self.read_u32(ADDR_LATEST)?;
        while search_addr != 0 {
            if self.read_u8(search_addr + 4)? == len {
                let mut found = true;
                for i in 0u32..len as u32 {
                    if self.read_u8(search_addr + 5 + i)? != self.read_u8(addr + i)? {
                        found = false;
                        break;
                    }
                }
                if found {
                    return Ok(search_addr);
                }
            }
            search_addr = self.read_u32(search_addr)?;
        }
        Ok(0)
    }

    fn find(&mut self) -> VMSuccess {
        let len = self.pop_data()? as u8;
        let addr = self.pop_data()?;
        self.push_data(self.find_word(addr, len)?);
        Ok(())
    }

    fn header_addr_to_cfa(&self, addr: u32) -> VMResult<u32> {
        let len = self.read_u8(addr + 4)?;
        Ok(addr + len as u32 + 5)
    }

    fn create(&mut self) -> VMSuccess {
        self.align()?;
        let mut here = self.read_u32(ADDR_HERE)?;
        let latest = self.read_u32(ADDR_LATEST)?;
        self.write_u32(ADDR_LATEST, here)?;
        self.write_u32(here, latest)?;
        here += 4;
        let word_len = self.pop_data()?;
        let word_addr = self.pop_data()?;
        self.write_u8(here, word_len as u8)?;
        here += 1;
        for i in 0..word_len {
            self.write_u8(here, self.read_u8(word_addr + i)?)?;
            here += 1;
        }
        self.write_u32(ADDR_HERE, here)?;
        Ok(())
    }

    fn step(&mut self) -> VMSuccess {
        let xt = self.read_u32(self.pc)?;
        self.pc += 4;
        self.exec(xt)
    }

    fn exec(&mut self, addr: u32) -> VMSuccess {
        let op: Op = self.read_u8(addr)?.into();
        match op {
            Op::DoColonDef => {
                self.push_return(self.pc);
                self.pc = align_addr(addr + 1);
            }
            Op::Dup => {
                let a = self.pop_data()?;
                self.push_data(a);
                self.push_data(a);
            }
            Op::Add => {
                let a = self.pop_data()?;
                let b = self.pop_data()?;
                self.push_data(a.wrapping_add(b));
            }
            Op::Lit => {
                self.push_data(self.read_u32(self.pc)?);
                self.pc += 4;
            }
            Op::Find => self.find()?,
            Op::Exit => self.pc = self.pop_return()?,
            Op::Unknown => return Err(VMError::UnknownOpcode),
        }
        Ok(())
    }

    fn display(&self) {
        println!("Current word address: {:x}", self.pc);
        println!(
            "Data stack ({} items): {:?}",
            self.data_stack.len(),
            self.data_stack
        );
        println!(
            "Return stack ({} items): {:?}",
            self.return_stack.len(),
            self.return_stack
                .iter()
                .map(|n| format!("{:x}", n))
                .collect::<Vec<_>>()
        );
        print!("Contents of memory:");
        let mut line = String::with_capacity(16);
        for (i, byte) in self.memory.iter().enumerate() {
            if i % 16 == 0 {
                println!("  {}", line);
                line = String::with_capacity(16);
                print!("({:04x}) ", i)
            }
            print!("{:02x} ", byte);
            line.push(match byte {
                0 => ' ',
                n if *n < 32 => '?',
                n if *n < 127 => *n as char,
                _ => '?',
            });
        }
        for _ in 0..((16 - self.memory.len() % 16) % 16) {
            print!("   ");
        }
        println!("  {}", line);
    }
}

fn str_to_bytes(s: &str) -> Vec<u8> {
    s.chars()
        .filter(|c| c.is_ascii())
        .map(|c| c as u8)
        .collect()
}

fn main() {
    println!("Hello, world!");
    let mut vm = VM::new();
    vm.init();
    loop {
        vm.display();
        vm.step().unwrap();
    }
}
