Rudimentary Forth environment, heavily based on [Jonesforth](https://github.com/nornagon/jonesforth/).

The Rust VM has 45 opcodes corresponding to Forth words, mostly for arithmetic, memory manipulation and code generation. The rest of the vocabulary is implemented in Forth.

This was a learning project and is unlikely to be useful for any practical purpose. In particular, error handling is pretty minimal and only ASCII is supported.

You can try it out by running the following, but you'll need to specify the `--verbose` flag to view the result on the data stack because numeric output is not yet implemented:

```
: fac dup 1 > if dup 1- recurse * then ;
fac 10
```
