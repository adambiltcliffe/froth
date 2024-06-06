: true      1 ;
: false     0 ;
: not       0 = ;
: negate    0 swap - ;
: 0=        0 = ;

: /         /mod swap drop ;
: mod       /mod drop ;

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
: 'a'           [ char a ] literal ;
: '0'           [ char 0 ] literal ;
: '('           [ char ( ] literal ;
: ')'           [ char ) ] literal ;
: '-'           [ char - ] literal ;
: '<'           [ char < ] literal ;
: '>'           [ char > ] literal ;
: '"'           [ char " ] literal ;

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

( ?dup, pick and roll would be much more efficient if defined in the VM,
  but we're aiming to keep it small. )

: ?dup      dup if dup then ;
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

: u.        base @ /mod
            ( Print the quotient )
            ?dup if recurse then
            ( Print the remainder )
            dup 10 < if '0' else 10 - 'a' then + emit ;
: uwidth    base @ / ?dup if recurse 1+ else 1 then ;
: u.r       swap dup uwidth rot swap - spaces u. ;
: .r        swap dup 0< if
                negate 1 swap rot 1-
            else
                0 swap rot
            then swap dup uwidth rot swap - spaces swap
            if '-' emit then u. ;
: .         0 .r space ;
( Note that we shadow the original definition of u. here )
: u.        u. space ;

: .s        depth dup '<' emit 0 .r '>' emit 2 spaces
            begin
                dup 0>
            while
                dup pick u. 1-
            repeat
            drop ;

( Now some string handling operations. )

: s"        state @ if
                ( In compile mode, copy the string into compiled word )
                ' litstring ,
                here @ 0 ,
                begin key dup '"' <> while c, repeat
                drop
                dup here @ swap -
                4- swap ! align
            else
                ( If interpreting, copy to 'here' temporarily )
                here @
                begin key dup '"' <> while over c! 1+ repeat
                drop
                here @ -
                here @
                swap
            then ; immediate

: tell      over + swap
            begin
                2dup >
            while
                dup c@ emit
                1+
            repeat
            2drop ;

: ."        state @ if
                [compile] s" ' tell ,
            else
                begin
                    key dup '"' = if drop exit then
                    emit
                again
            then ; immediate

( Some utilities to display the dictionary )

: id.       4+ dup c@ length-mask and
            begin
                dup 0>
            while
                swap 1+
                dup c@ emit
                swap 1-
            repeat 2drop ;

: ?hidden
            4+ c@ hidden-flag and ;
: ?immediate
            4+ c@ immediate-flag and ;

: words     latest @
            begin ?dup
            while
                dup ?hidden not if
                    dup id. space
                then
                @
            repeat cr ;

( And to finish off with a sense of pride and accomplishment for everything we have made here ... )
: count-words
            0 latest @
            begin ?dup
            while
                dup ?hidden not if
                    swap 1+ swap
                then
                @
            repeat ;

here @ . ." bytes of memory allocated." cr
count-words . ." words defined. Type 'words' to see them." cr
