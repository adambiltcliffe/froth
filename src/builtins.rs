use crate::{align_addr, Op, VM};

impl VM {
    // note: add_builtin_word and add_colon_word don't return
    // a Result because they should only ever be called at init
    fn add_builtin_word(&mut self, word: &str, op: Op) {
        self.buffer_word(word);
        self.create().unwrap();
        self.write_u8_here(op.into()).unwrap()
    }

    fn add_colon_word(&mut self, word: &str, def: Vec<&str>) {
        self.buffer_word(word);
        self.create().unwrap();
        self.write_u8_here(Op::DoColonDef.into()).unwrap();
        self.align().unwrap();
        for s in def {
            let a = self.buffer_and_find_word(s).unwrap();
            let cfa = self.header_addr_to_cfa(a).unwrap();
            self.write_u32_here(cfa).unwrap();
        }
        let a = self.buffer_and_find_word("exit").unwrap();
        let cfa = self.header_addr_to_cfa(a).unwrap();
        self.write_u32_here(cfa).unwrap();
    }

    fn set_entry_point(&mut self, word: &str) {
        let addr = self.buffer_and_find_word(word).unwrap();
        let cfa = self.header_addr_to_cfa(addr).unwrap();
        assert!(self.read_u8(cfa).unwrap() == 0);
        self.pc = align_addr(cfa + 1);
    }

    pub(crate) fn init(&mut self) {
        self.add_builtin_word("dup", Op::Dup);
        self.add_builtin_word("one", Op::One);
        self.add_builtin_word("add", Op::Add);
        self.add_builtin_word("find", Op::Find);
        self.add_builtin_word("exit", Op::Exit);
        self.add_colon_word("test", vec!["one", "dup", "add"]);
        self.add_colon_word("begin", vec!["one", "test", "add"]);
        self.set_entry_point("begin");
    }
}
