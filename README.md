Rudimentary Forth environment, heavily based on [Jonesforth](https://github.com/nornagon/jonesforth/).

The Rust VM has 44 opcodes corresponding to Forth words, mostly for arithmetic, memory manipulation and code generation. The rest of the vocabulary (a further 75 words at present) is implemented in Forth.

This was a learning project and is unlikely to be useful for any practical purpose. In particular, error handling is pretty minimal and only ASCII is supported.

You can try it out by running the following:

```
: fac dup 1 > if dup 1- recurse * then ;
10 fac .
```

Or, if you prefer iteration to recursion:

```
: fac 1 swap begin dup 1 > while tuck * swap 1- repeat drop ;
10 fac .
```
