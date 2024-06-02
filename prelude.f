: true      1 ;
: false     0 ;
: not       0 = ;
: negate    0 swap - ;
: 0=        0 = ;

: 1+        1 + ;
: 1-        1 - ;
: 4+        4 + ;
: 4-        4 - ;

: >dfa      >cfa 1+ align ;
: hide      word find hidden ;

: bl        32 ;
: space     bl emit ;
: cr        10 emit ;
: char      word drop c@ ;

: literal       ' lit , , ;             immediate
: 'A'           [ char A ] literal ;
: '0'           [ char 0 ] literal ;
: '('           [ char ( ] literal ;
: ')'           [ char ) ] literal ;

: [compile]     word find >cfa , ;      immediate
: recurse       latest @ >cfa , ;       immediate

: if            ' 0branch , here @ 0 , ;
                                        immediate
: unless        ' not , [compile] if ;  immediate
: then          dup here @ swap - 4+ swap ! ;
                                        immediate
: else          ' branch , here @ 0 , swap [compile] then ;
                                        immediate
: begin         here @ ;                immediate
: end-loop      here @ - 4+ , ;
: until         ' 0branch , end-loop ;
                                        immediate
: again         ' branch , end-loop ;
                                        immediate
: while         ' 0branch , here @ 0 , ;
                                        immediate
: repeat        ' branch , swap end-loop [compile] then ;
                                        immediate
hide end-loop

: (             1 begin
                    key dup '(' = if
                        drop 1+
                    else
                        ')' = if
                            1-
                        then
                    then
                dup 0= until drop ;     immediate

( The previous block extends the environment to include a comment parser,
so from this point we can actually include comments in the prelude! )

( Define some extended stack manipulation primitives. )

: over      >r dup r> swap ;
: nip       swap drop ;
: tuck      swap over ;
: rot       >r swap r> swap ;
: -rot      swap >r swap r> ;

( pick and roll would be much more efficient if defined in the VM, but
  we're aiming to keep it small. )

: pick      dup 0= if drop dup exit then swap >r 1- recurse r> swap ;
: roll      dup if swap >r 1- recurse r> swap exit then drop ;

: 2drop     drop drop ;
: 2dup      over over ;
: 2swap     rot >r rot r> ;
: 2over     >r >r 2dup r> r> 2swap ;

( And some additional convenience operators. )

: <>        = not ;
: <=        > not ;
: >=        < not ;
: 0<>       0 <> ;
: 0<        0 < ;
: 0>        0 > ;
: 0<=       0 <= ;
: 0>=       0 >= ;

: +!        tuck @ + swap ! ;
: -!        tuck @ swap - swap ! ;

( Now for some output functionality. )

: spaces    begin dup 0> while space 1- repeat drop ;
: decimal   10 base ! ;
: hex       16 base ! ;
