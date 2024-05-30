: over      >r dup r> swap ;
: tuck      swap over ;
: rot       >r swap r> swap ;
: -rot      swap >r swap r> ;

: 2drop     drop drop ;
: 2dup      over over ;
: 2swap     rot >r rot r> ;
: 2over     >r >r 2dup r> r> 2swap ;

: true      1 ;
: false     0 ;
: not       0 = ;
: negate    0 swap - ;
: <>        = not ;
: <=        > not ;
: >=        < not ;
: 0=        0 = ;
: 0<>       0 <> ;
: 0<        0 < ;
: 0>        0 > ;
: 0<=       0 <= ;
: 0>=       0 >= ;

: 1+        1 + ;
: 1-        1 - ;
: 4+        4 + ;
: 4-        4 - ;

: +!        tuck @ + swap ! ;
: -!        tuck @ swap - swap ! ;

: >dfa      >cfa 1+ align ;
: hide      word find hidden ;

: bl        32 ;
: space     bl emit ;
: cr        10 emit ;
: char      word drop c@ ;

: literal       ' lit , , ;             immediate
: 'A'           [ char A ] literal ;
: '0'           [ char 0 ] literal ;

: [compile]     word find >cfa , ;      immediate
: recurse       latest @ >cfa , ;       immediate
