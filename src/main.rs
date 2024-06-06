use num_enum::{FromPrimitive, IntoPrimitive};
use std::borrow::Cow;
use std::fs::File;
use std::io::{BufReader, Read, Write};
use std::iter::once;

mod builtins;

const ADDR_LATEST: u32 = 0;
const ADDR_BASE: u32 = 4;
const ADDR_STATE: u32 = 8;
const ADDR_HERE: u32 = 12;
const ADDR_WORD_BUFFER: u32 = 16;
const INITIAL_HERE: u32 = 48;

const MAX_EXTEND: u32 = 64;

const HIDDEN_FLAG: u8 = 32;
const IMMEDIATE_FLAG: u8 = 64;
const LENGTH_MASK: u8 = 31;

#[derive(FromPrimitive, IntoPrimitive, Copy, Clone)]
#[repr(u8)]
enum Op {
    DoColonDef = 0,
    Dup,
    Drop,
    Swap,
    Depth,
    ToR,
    FromR,
    Fetch,
    CFetch,
    Store,
    CStore,
    Align,
    Add,
    Subtract,
    Multiply,
    DivMod,
    Equals,
    LessThan,
    GreaterThan,
    And,
    Or,
    Xor,
    Invert,
    Lit,
    LitString,
    Key,
    Word,
    Emit,
    Find,
    Number,
    ToCFA,
    LBracket,
    RBracket,
    Create,
    Comma,
    CComma,
    Immediate,
    Hidden,
    Tick,
    Execute,
    Branch,
    BranchIfZero,
    Exit,
    Reset,
    Interpret,
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
    IOError,
    UnknownWord(String),
    Terminated,
}

fn error_name(err: &VMError) -> Cow<'static, str> {
    match err {
        VMError::IllegalAddress => "illegal address".into(),
        VMError::UnknownOpcode => "unknown opcode".into(),
        VMError::DataStackUnderflow => "data stack underflow".into(),
        VMError::ReturnStackUnderflow => "return stack underflow".into(),
        VMError::UnalignedAccess => "unaligned memory access".into(),
        VMError::IOError => "i/o error".into(),
        VMError::UnknownWord(s) => format!("unknown word {}", s).into(),
        VMError::Terminated => "input terminated".into(),
    }
}

fn align_addr(addr: u32) -> u32 {
    ((addr + 3) / 4) * 4
}

fn digit_val(digit: char) -> u32 {
    if digit >= '0' && digit <= '9' {
        return (digit as u32).wrapping_sub('0' as u32);
    }
    (digit as u32).wrapping_sub('a' as u32).wrapping_add(10)
}

type VMResult<T> = Result<T, VMError>;
type VMSuccess = VMResult<()>;

struct VM {
    memory: Vec<u8>,
    data_stack: Vec<u32>,
    return_stack: Vec<u32>,
    pc: u32,
    entry: u32,
    lit: u32,
    input: Box<dyn Iterator<Item = std::io::Result<u8>>>,
    running: bool,
    line: bool,
    errors: Vec<VMError>,
}

impl VM {
    fn new() -> Self {
        let f = File::open("prelude.f").expect("Could not open prelude");
        let input = Box::new(
            BufReader::new(f)
                .bytes()
                .map(|b| if matches!(b, Ok(13)) { Ok(32) } else { b })
                .chain(once(Ok(13)))
                .chain(std::io::stdin().bytes()),
        );
        let mut me = Self {
            memory: vec![0; INITIAL_HERE as usize],
            data_stack: Vec::new(),
            return_stack: Vec::new(),
            pc: 0,
            entry: 0,
            lit: 0,
            input,
            running: true,
            line: false,
            errors: Vec::new(),
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
            if self.read_u8(search_addr + 4)? & (LENGTH_MASK | HIDDEN_FLAG) == len {
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

    fn number(&mut self) -> VMSuccess {
        let len = self.pop_data()?;
        let addr = self.pop_data()?;
        let (value, error) = self.parse_number(addr, len)?;
        self.push_data(value);
        self.push_data(error);
        Ok(())
    }

    fn parse_number(&self, addr: u32, len: u32) -> VMResult<(u32, u32)> {
        let base = self.read_u32(ADDR_BASE)?;
        let mut offs = 0;
        let sym = self.read_u8(addr)?;
        let sign = if sym as char == '-' && len > 1 {
            offs += 1;
            -1i32
        } else {
            1i32
        };
        let mut result = 0;
        while offs < len {
            let sym = self.read_u8(addr + offs)? as char;
            let val = digit_val(sym);
            if val < base {
                result *= base;
                result += val;
                offs += 1
            } else {
                break;
            }
        }
        if offs == 1 && sign == -1 {
            // only character parsed was '-'
            return Ok((0, len)); // no characters consumed, indicating error
        } else {
            let value = (result as i32 * sign) as u32;
            let error = len - offs;
            return Ok((value, error));
        }
    }

    fn header_addr_to_cfa(&self, addr: u32) -> VMResult<u32> {
        let len = self.read_u8(addr + 4)? & LENGTH_MASK;
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

    fn immediate(&mut self) -> VMSuccess {
        let header_addr = self.read_u32(ADDR_LATEST)?;
        let byte = self.read_u8(header_addr + 4)?;
        let byte = byte ^ IMMEDIATE_FLAG;
        self.write_u8(header_addr + 4, byte)
    }

    fn hidden(&mut self) -> VMSuccess {
        let header_addr = self.pop_data()?;
        let byte = self.read_u8(header_addr + 4)?;
        let byte = byte ^ HIDDEN_FLAG;
        self.write_u8(header_addr + 4, byte)
    }

    fn input_byte(&mut self) -> VMResult<u8> {
        if self.line {
            self.prompt();
            self.line = false
        }
        match self.input.next() {
            None => {
                self.running = false;
                Err(VMError::Terminated)
            }
            Some(Err(_)) => {
                self.running = false;
                Err(VMError::IOError)
            }
            Some(Ok(b)) => {
                if b == 13 {
                    self.line = true
                }
                Ok(b)
            }
        }
    }

    fn input_word(&mut self) -> VMResult<(u32, u32)> {
        let mut i = 0;
        loop {
            let b = self.input_byte()?;
            if b.is_ascii_whitespace() {
                if i > 0 {
                    break;
                }
            } else {
                if i < 31 {
                    self.write_u8(ADDR_WORD_BUFFER + i, b)?;
                    i += 1;
                }
            }
        }
        Ok((ADDR_WORD_BUFFER, i))
    }

    fn word(&mut self) -> VMSuccess {
        let (addr, len) = self.input_word()?;
        self.push_data(addr);
        self.push_data(len);
        Ok(())
    }

    fn exec_pc(&mut self) -> VMSuccess {
        let xt = self.read_u32(self.pc)?;
        self.pc += 4;
        self.exec(xt)
    }

    fn step(&mut self) {
        match self.exec_pc() {
            Ok(()) => (),
            Err(e) => {
                if self.errors.len() < 10 {
                    self.errors.push(e);
                    // attempt recovery
                    self.pc = self.entry
                } else {
                    for e in &self.errors {
                        println!("{}", error_name(e));
                    }
                    println!("too many errors, aborting");
                    self.running = false;
                }
            }
        }
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
            Op::Drop => {
                self.pop_data()?;
            }
            Op::Swap => {
                let a = self.pop_data()?;
                let b = self.pop_data()?;
                self.push_data(a);
                self.push_data(b);
            }
            Op::Depth => {
                let a = self.data_stack.len() as u32;
                self.push_data(a);
            }
            Op::ToR => {
                let val = self.pop_data()?;
                self.push_return(val);
            }
            Op::FromR => {
                let val = self.pop_return()?;
                self.push_data(val);
            }
            Op::Fetch => {
                let addr = self.pop_data()?;
                let data = self.read_u32(addr)?;
                self.push_data(data);
            }
            Op::CFetch => {
                let addr = self.pop_data()?;
                let data = self.read_u8(addr)?;
                self.push_data(data as u32);
            }
            Op::Store => {
                let addr = self.pop_data()?;
                let val = self.pop_data()?;
                self.write_u32(addr, val)?;
            }
            Op::CStore => {
                let addr = self.pop_data()?;
                let val = self.pop_data()? as u8;
                self.write_u8(addr, val)?;
            }
            Op::Align => self.align()?,
            Op::Add => {
                let b = self.pop_data()?;
                let a = self.pop_data()?;
                self.push_data(a.wrapping_add(b));
            }
            Op::Subtract => {
                let b = self.pop_data()?;
                let a = self.pop_data()?;
                self.push_data(a.wrapping_sub(b));
            }
            Op::Multiply => {
                let b = self.pop_data()?;
                let a = self.pop_data()?;
                self.push_data(a.wrapping_mul(b));
            }
            Op::DivMod => {
                let b = self.pop_data()?;
                let a = self.pop_data()?;
                self.push_data(a.wrapping_rem(b));
                self.push_data(a.wrapping_div(b));
            }
            Op::Equals => {
                let b = self.pop_data()?;
                let a = self.pop_data()?;
                self.push_data(if a == b { 1 } else { 0 });
            }
            Op::LessThan => {
                let b = self.pop_data()? as i32;
                let a = self.pop_data()? as i32;
                self.push_data(if a < b { 1 } else { 0 });
            }
            Op::GreaterThan => {
                let b = self.pop_data()? as i32;
                let a = self.pop_data()? as i32;
                self.push_data(if a > b { 1 } else { 0 });
            }
            Op::And => {
                let b = self.pop_data()?;
                let a = self.pop_data()?;
                self.push_data(a & b);
            }
            Op::Or => {
                let b = self.pop_data()?;
                let a = self.pop_data()?;
                self.push_data(a | b);
            }
            Op::Xor => {
                let b = self.pop_data()?;
                let a = self.pop_data()?;
                self.push_data(a ^ b);
            }
            Op::Invert => {
                let a = self.pop_data()?;
                self.push_data(!a);
            }
            Op::Lit => {
                let value = self.read_u32(self.pc)?;
                self.push_data(value);
                self.pc += 4;
            }
            Op::LitString => {
                let len = self.read_u32(self.pc)?;
                self.pc += 4;
                self.push_data(self.pc);
                self.push_data(len);
                self.pc = align_addr(self.pc + len);
            }
            Op::Find => self.find()?,
            Op::Number => self.number()?,
            Op::ToCFA => {
                let header_addr = self.pop_data()?;
                self.push_data(self.header_addr_to_cfa(header_addr)?);
            }
            Op::LBracket => self.write_u32(ADDR_STATE, 0)?,
            Op::RBracket => self.write_u32(ADDR_STATE, 1)?,
            Op::Immediate => self.immediate()?,
            Op::Hidden => self.hidden()?,
            Op::Key => {
                let data = self.input_byte()? as u32;
                self.push_data(data)
            }
            Op::Word => self.word()?,
            Op::Emit => print!("{}", self.pop_data()? as u8 as char),
            Op::Create => self.create()?,
            Op::Comma => {
                let val = self.pop_data()?;
                self.write_u32_here(val)?;
            }
            Op::CComma => {
                let val = self.pop_data()? as u8;
                self.write_u8_here(val)?;
            }
            Op::Tick => {
                let xt = self.read_u32(self.pc)?;
                self.pc += 4;
                self.push_data(xt);
            }
            Op::Execute => {
                let xt = self.pop_data()?;
                self.exec(xt)?;
            }
            Op::Branch => {
                let offs = self.read_u32(self.pc)?;
                self.pc = self.pc.wrapping_sub(4).wrapping_add(offs);
            }
            Op::BranchIfZero => {
                let condition = self.pop_data()?;
                let offs = self.read_u32(self.pc)?;
                if condition == 0 {
                    self.pc = self.pc.wrapping_sub(4).wrapping_add(offs);
                } else {
                    self.pc += 4;
                }
            }
            Op::Exit => self.pc = self.pop_return()?,
            Op::Reset => self.return_stack.clear(),
            Op::Interpret => {
                let compiling = self.read_u32(ADDR_STATE)? != 0;
                let (addr, len) = self.input_word()?;
                let header_addr = self.find_word(addr, len as u8)?;
                if header_addr > 0 {
                    let immediate = (self.read_u8(header_addr + 4)? & IMMEDIATE_FLAG) != 0;
                    let xt = self.header_addr_to_cfa(header_addr)?;
                    if compiling && !immediate {
                        self.write_u32_here(xt)?;
                    } else {
                        self.exec(xt)?;
                    }
                } else {
                    let (value, error) = self.parse_number(addr, len)?;
                    if error == 0 {
                        if compiling {
                            self.write_u32_here(self.lit)?;
                            self.write_u32_here(value)?;
                        } else {
                            self.push_data(value)
                        }
                    } else {
                        let mut result = String::with_capacity(len as usize);
                        for a in addr..(addr + len) {
                            result.push(self.read_u8(a)? as char);
                        }
                        return Err(VMError::UnknownWord(result));
                    }
                }
            }
            Op::Unknown => {
                return Err(VMError::UnknownOpcode);
            }
        }
        Ok(())
    }

    fn prompt(&mut self) {
        if self.line {
            if self.errors.len() == 0 {
                println!(" ok")
            } else {
                for err in self.errors.drain(..) {
                    println!(" {}", error_name(&err));
                }
            }
            print!(">");
            std::io::stdout().flush().expect("io error");
            self.line = false;
        }
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
    }

    fn dump(&self) {
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
    use std::env::args;
    let verbose = args().any(|s| s == "--verbose");
    let dump = args().any(|s| s == "--dump");
    let mut vm = VM::new();
    vm.init();
    println!("[loading prelude]");
    while vm.running {
        if dump {
            vm.dump();
        }
        if verbose {
            vm.display();
            if vm.read_u32(ADDR_STATE).unwrap() != 0 {
                println!("[compile mode]");
            }
        }
        vm.step();
    }
}
