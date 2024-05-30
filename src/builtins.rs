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
        let lit = self.add_builtin_word("lit", Op::Lit);
        self.lit = lit; // store it for use in compilation

        let _dup = self.add_builtin_word("dup", Op::Dup);
        let _drop = self.add_builtin_word("drop", Op::Drop);
        let _swap = self.add_builtin_word("swap", Op::Swap);
        let _tor = self.add_builtin_word(">r", Op::ToR);
        let _fromr = self.add_builtin_word("r>", Op::FromR);
        let fetch = self.add_builtin_word("@", Op::Fetch);
        let _cfetch = self.add_builtin_word("c@", Op::CFetch);
        let _cstore = self.add_builtin_word("!", Op::Store);
        let align = self.add_builtin_word("align", Op::Align);
        let _store = self.add_builtin_word("c!", Op::CStore);
        let _add = self.add_builtin_word("+", Op::Add);
        let _subtract = self.add_builtin_word("-", Op::Subtract);
        let _multiply = self.add_builtin_word("*", Op::Multiply);
        let _divide = self.add_builtin_word("/", Op::Divide);
        let _modulus = self.add_builtin_word("%", Op::Modulus);
        let _equals = self.add_builtin_word("=", Op::Equals);
        let _lt = self.add_builtin_word("<", Op::LessThan);
        let _gt = self.add_builtin_word(">", Op::GreaterThan);
        let _and = self.add_builtin_word("and", Op::And);
        let _or = self.add_builtin_word("or", Op::Or);
        let _xor = self.add_builtin_word("xor", Op::Xor);
        let _invert = self.add_builtin_word("invert", Op::Invert);
        let _key = self.add_builtin_word("key", Op::Key);
        let word = self.add_builtin_word("word", Op::Word);
        let _emit = self.add_builtin_word("emit", Op::Emit);
        let _find = self.add_builtin_word("find", Op::Find);
        let _number = self.add_builtin_word("number", Op::Number);
        let _to_cfa = self.add_builtin_word(">cfa", Op::ToCFA);
        let lbracket = self.add_builtin_word("[", Op::LBracket);
        let rbracket = self.add_builtin_word("]", Op::RBracket);
        let create = self.add_builtin_word("create", Op::Create);
        let comma = self.add_builtin_word(",", Op::Comma);
        let ccomma = self.add_builtin_word("c,", Op::CComma);
        let _immediate = self.add_builtin_word("immediate", Op::Immediate);
        self.immediate().unwrap(); // 'immediate' is an immediate word
        let hidden = self.add_builtin_word("hidden", Op::Hidden);
        let _execute = self.add_builtin_word("execute", Op::Execute);
        let branch = self.add_builtin_word("branch", Op::Branch);
        let _branchif0 = self.add_builtin_word("0branch", Op::BranchIfZero);
        let exit = self.add_builtin_word("exit", Op::Exit);
        let reset = self.add_builtin_word("reset", Op::Reset);
        let prompt = self.add_builtin_word("prompt", Op::Prompt);
        let interpret = self.add_builtin_word("interpret", Op::Interpret);

        let _base = self.add_colon_word("base", vec![lit, ADDR_BASE, exit]);
        let _here = self.add_colon_word("here", vec![lit, ADDR_HERE, exit]);
        let latest = self.add_colon_word("latest", vec![lit, ADDR_LATEST, exit]);
        let _state = self.add_colon_word("state", vec![lit, ADDR_STATE, exit]);
        let _colon = self.add_colon_word(
            ":",
            vec![
                word,
                create,
                lit,
                Op::DoColonDef as u32,
                ccomma,
                align,
                latest,
                fetch,
                hidden,
                rbracket,
                exit,
            ],
        );
        let _semicolon = self.add_colon_word(
            ";",
            vec![lit, exit, comma, latest, fetch, hidden, lbracket, exit],
        );
        self.immediate().unwrap(); // ';' is an immediate word
        let quit = self.add_colon_word(
            "quit",
            vec![reset, prompt, interpret, branch, -12i32 as u32],
        );

        self.set_entry_point(quit);

        // need to do these in the VM or explicitly in here
        // litstring

        // we can implement these in pure Forth once we have compilation working:
        // rot, -rot, ?dup, 1+, 1-, 4+, 4-, <>, <=, >=, 0=, 0<>, 0<, 0>, 0<=, 0>=
        // not, negate, +!, -!, >dfa, /mod, hide, char, ', tell
        // 2drop, 2dup, 2swap
    }
}
