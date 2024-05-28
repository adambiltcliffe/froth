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
        let addr = align_addr(xt + 1);
        self.entry = addr;
        self.pc = addr;
    }

    pub(crate) fn init(&mut self) {
        let _dup = self.add_builtin_word("dup", Op::Dup);
        let _drop = self.add_builtin_word("drop", Op::Drop);
        let _swap = self.add_builtin_word("swap", Op::Swap);
        let _tor = self.add_builtin_word(">r", Op::ToR);
        let _fromr = self.add_builtin_word("r>", Op::FromR);
        let _fetch = self.add_builtin_word("@", Op::Fetch);
        let _store = self.add_builtin_word("!", Op::Store);
        let _add = self.add_builtin_word("+", Op::Add);
        let _subtract = self.add_builtin_word("-", Op::Subtract);
        let _multiply = self.add_builtin_word("*", Op::Multiply);
        let _divide = self.add_builtin_word("/", Op::Divide);
        let _modulus = self.add_builtin_word("%", Op::Modulus);
        let lit = self.add_builtin_word("lit", Op::Lit);
        let _key = self.add_builtin_word("key", Op::Key);
        let _word = self.add_builtin_word("word", Op::Word);
        let _emit = self.add_builtin_word("emit", Op::Emit);
        let _find = self.add_builtin_word("find", Op::Find);
        let _number = self.add_builtin_word("number", Op::Number);
        let _to_cfa = self.add_builtin_word(">cfa", Op::ToCFA);
        let _create = self.add_builtin_word("create", Op::Create);
        let _execute = self.add_builtin_word("execute", Op::Execute);
        let branch = self.add_builtin_word("branch", Op::Branch);
        let exit = self.add_builtin_word("exit", Op::Exit);
        let reset = self.add_builtin_word("reset", Op::Reset);
        let prompt = self.add_builtin_word("prompt", Op::Prompt);
        let interpret = self.add_builtin_word("interpret", Op::Interpret);

        let _one = self.add_colon_word("one", vec![lit, 1, exit]); // temporary
        let _base = self.add_colon_word("base", vec![lit, ADDR_BASE, exit]);
        let _here = self.add_colon_word("here", vec![lit, ADDR_HERE, exit]);
        let _latest = self.add_colon_word("latest", vec![lit, ADDR_LATEST, exit]);
        let _state = self.add_colon_word("state", vec![lit, ADDR_STATE, exit]);
        let quit = self.add_colon_word(
            "quit",
            vec![reset, prompt, interpret, branch, -12i32 as u32],
        );

        self.set_entry_point(quit);

        // need to do these in the VM or explicitly in here
        // =, <, >, and, or, xor, invert, c@, c!
        // :, ,, c,, ;, [, ], immediate, hidden, hide, ', 0branch, litstring, tell, char

        // we can implement these in pure Forth once we have compilation working:
        // over, rot, -rot, ?dup, 1+, 1-, 4+, 4-, <>, <=, >=, 0=, 0<>, 0<, 0>, 0<=, 0>=
        // not, negate, +!, -!, >dfa, /, mod
        // 2drop, 2dup, 2swap
    }
}
