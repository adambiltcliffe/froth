use crate::{align_addr, Op, ADDR_BASE, ADDR_HERE, ADDR_LATEST, ADDR_STATE, VM};

impl VM {
    // note: add_builtin_word and add_colon_word don't return
    // a Result because they should only ever be called at init
    fn add_builtin_word(&mut self, word: &str, op: Op) -> u32 {
        self.buffer_word(word);
        self.create().unwrap();
        let xt = self.read_u32(ADDR_HERE).unwrap();
        self.write_u8_here(op.into()).unwrap();
        xt
    }

    fn add_colon_word(&mut self, word: &str, def: Vec<u32>) -> u32 {
        self.buffer_word(word);
        self.create().unwrap();
        let xt = self.read_u32(ADDR_HERE).unwrap();
        self.write_u8_here(Op::DoColonDef.into()).unwrap();
        self.align().unwrap();
        for item in def {
            self.write_u32_here(item).unwrap();
        }
        xt
    }

    fn set_entry_point(&mut self, xt: u32) {
        assert!(self.read_u8(xt).unwrap() == 0);
        self.pc = align_addr(xt + 1);
    }

    pub(crate) fn init(&mut self) {
        let dup = self.add_builtin_word("dup", Op::Dup);
        let add = self.add_builtin_word("add", Op::Add);
        let lit = self.add_builtin_word("lit", Op::Lit);
        let _find = self.add_builtin_word("find", Op::Find);
        let exit = self.add_builtin_word("exit", Op::Exit);

        let one = self.add_colon_word("one", vec![lit, 1, exit]);
        let seven = self.add_colon_word("seven", vec![lit, 7, exit]);
        let _base = self.add_colon_word("base", vec![lit, ADDR_BASE, exit]);
        let _here = self.add_colon_word("here", vec![lit, ADDR_HERE, exit]);
        let _latest = self.add_colon_word("latest", vec![lit, ADDR_LATEST, exit]);
        let _state = self.add_colon_word("state", vec![lit, ADDR_STATE, exit]);

        let test = self.add_colon_word("test", vec![one, dup, add, exit]);
        let begin = self.add_colon_word("begin", vec![one, test, add, seven, exit]);
        self.set_entry_point(begin);
    }
}
