# Introduction to Sublang
Sublang is a bare bones assembly-like language consisting of four main elements:
* The **SUBLEQ** instruction
* **Labels** to refer to areas of memory easily
* **Macros** for code reuse
* **Syntax sugar** for common constructs


## Subleq
<!-- Clojure highlighting is used as an approximation for Sublang -->
```clojure
1 2 3 ; This is interpreted as standard subleq: mem[2] -= mem[1] jump to 3 if LEQ otherwise it goes to the next instruction. 
; This syntax is valid, but pointless.

; memory addresses and instructions may be labeled
a -> 1
b -> 2

c ->
    a -= b ; This syntax is used for the SUBLEQ instruction: Subtract b from a
    a -= b c ; Subtract b from a and jump to c if the result is less than or equal to zero

; If no `c` argument is given, the next instruction will always be executed, even if the result is LEQ to zero
; So these two are equivalent
a -= b
a -= b $1 ; `$1` gives a relative address with offset one

a -= b ; is equivalent to
b a $1 ; This syntax works but is not recommended, since it makes it harder for the assembler to give hints

; Other examples, literals and labels may freely be combined
a -= b 0x0000
'\0' -= 0 c

**
    Block comment.
    Important NOTE: Labeled values take up space in memory and will be
    executed if passed so:
**
a -> 2
4 10 5 ; will be executed as 2 4 10, NOT 4 10 5. To prevent this, jump over any
       ; label definitions, or use the assignment syntax
```

## Labels and Literals
```clojure
a -> 123 ; Decimal
b -> 0x4C6 ; Hex
c -> "Hello, World!" ; Strings are null-terminated by the assembler
d -> 'P' ; Character literals

.label ->
    a -= b .label   ; Repeats as long as (a -= b) <= 0
```

## Scopes
Scoping works like in most other languages. Note: Only labels are affected by scopes, macro definitions in scopes will still be globally accessible
```clojure
Z -> 123
X -> 456
Y -> 0
{
    Z -> 789
    {
        X -= Z  ; 456 - 789
    }
}
Y -= Z ; 0 - 123
```


## IO
```clojure

char -> 'a'
input -> 0
W -> 0

-1 -= char ; Prints 'a' to the screen

input -= -1
-1 -= input ; Echoes back users input

W -= W -1 ; Halts execution

```



## Naming conventions
* `@MyMacro` macros in PascalCase
* `my_label` labels in snake_case, with the exception of single character 'registers', like `Z` or `W`
* `MyFile.sbl` files in PascalCase
* `module` modules (folders) in snake_case
### Labels
* `p_value` pointer (not type-checked)
* `p_p_value` pointer to pointer (not type-checked)
* `n_value` negated value (not type-checked)
* `value?`  macro argument in definition
* `.value` a label to jump to
* `CONST_VALUE` constant, can be applied to all of the above and should be applied to macro arguments, but NOT to literals (l_name), since they are always constant by definition
* `Namespace::label` or `Namespace::Macro` for namespacing
* `Namespace::SubNamespace::label`


## Types
The assembler has a simple type-checker, which can be disabled.

* `value` normal label
* `l_value` literal value
* `s_value` scoped value
* `a_value` anything, no type checking
* `b_value` a braced value
* `m_value` a macro call passed as argument, must be braced. In practice it's the same as `b_value`
Currently types are only checked for macro parameters.

## Macros
### Definition
```clojure
@Name {
    ...
}
@Name   ; This is allowed as well, but discouraged
{
    ...
}

; with parameters
@Name a? b? c? {
    ...
}

; It is also possible to define a macro that isn't scoped:
@Name a? [
    ...
]
; This, however, is dangerous when label definitions take place in the macro, so it is generally discouraged.

; Linebreaks are allowed between parameters

@Name a?
      b?
      c?
      d?
      e? {
    ...

}

```
### Expanding
```clojure
!Name
; With arguments
!Name2 a b c
; Linebreaks are NOT allowed between arguments
```

### Hygiene
Macros are hygienic. Variables won't be shadowed.
```clojure
; Macros
a -> 0
@MyMacro b {
    a -= b
    a -> 123
}

!MyMacro a
; Is completely fine, and will become the following:
{
    ?MyMacro?a -= a
    ?MyMacro?a -> 123
}
```

### Macro arguments
You may pass scopes as macro arguments

```clojure
@Mac s_my_scope? {
    s_my_scope?
}


!Mac { a -= b } 
; =>
{
    { a -= b }
}

; If a macro takes multiple scopes, they can be chained as follows:
!Mac {
    ...
} {
    ...
} {
    ...
}
```
If you don't want the argument to be surrounded by scopes, you can use braces

```clojure
@Mac b_my_braced? {
    b_my_braced?
}

!Mac ( a -= b )
; =>
{
    a -= b
}
```
This means that you can 'curry' macros (using that term loosely)
```clojure
@Mac b_some_macro? {
    b_some_macro? 10
    b_some_macro? 3
}

@CurriedMacro l_a? l_b? {
    l_a? -= l_b?
}
!Mac ( !CurriedMacro 5 ) 
; =>
{
    {
        5 -= 10
        5 -= 3
    }
}

```

## Pointers
### Referencing
To create a pointer to a value, the relative address syntax `$1` must be used to get the address of the next token
```clojure
ptr -> $1 0x1234 ; This takes up two words of memory, one for the pointer and one for the value

ptr -> $1 "String"
; or
ptr -> &'A'
ptr -> &"String"
ptr -> &123
; & is equivalent to $1
```

### Dereferencing
```clojure
; Generic sequence for dereferencing. The value that 'ptr' points to will be subtracted from 'a'
!Copy ptr b
a -= (b -> 0)
; If 'ptr' is constant the following is also legal. Note: ptr doesn't have to be constant but this syntax will give unexpected results if it isn't
a -= (b -> PTR)

; The '*' operator may also be used:
a -= *ptr
; This is effectively syntax sugar for
!Copy ptr b
a -= (b -> 0)


; (But in reality it is exactly equivalent to)
_ASM    _ASM    &1
*ID*ptr *ID*ptr &1
ptr     _ASM    &1
_ASM    *ID*ptr &1
a -= (*ID*ptr -> 0)
; *ID*ptr is a safe and automatically generated name

```
Remember that because of how Subleq works, what are called 'Labels' here, are also just pointers! But since Subleq dereferences them, we can think of them as values. But keep in mind that literals require indirection `a -= 10` doesn't subtract 10 from a


## Inclusions
The `#` symbol may be used to include another .sbl file anywhere
```Clojure
#MySblFile.sbl
; You may leave out the .sbl extension:
#MySblFile

; You can also do this:
...
Z -= Z
P -= Z
#IncludeMe
!Macro P
...
; But of course beware of the contents of the included file
```
If you want to create a module (a set of .sbl files in a folder) you must create a folder with the name of the module (for example 'sublib'). And in that folder create a Lib.sbl file. Whenever the 'sublib' folder is imported, this is automatically resolved to 'sublib/Lib.sbl'. In this .sbl file you may include any other files you might need. Includes are initially resolved relative to the file
that is including, and if the target isn't found in the *LIBS* folder, defined using the `-l` command line argument.


```clojure
; ./subleq/MyFile.sbl
#math/FastSqrt
```
The order in which files are checked is as follows. The first one that exists will be included.
* `./subleq/math/FastSqrt/Lib.sbl`
* `./subleq/math/FastSqrt.sbl`
* `LIBS/math/FastSqrt/Lib.sbl`
* `LIBS/math/FastSqrt.sbl`


See subleq/libs/sublib for an example.

## Miscellaneous Syntax sugar
### Mult operator
When the '*' is placed before a literal `n`, the previous token is repeated `n` times
```clojure
label * 3 ; =>
label label label

0x123 * 0x4 ; =>
0x123 0x123 0x123 0x123

; mind that 3 * label will dereference `label`!
```

### Assignments
The `=` operator can be used to both declare a label and assign it a value every time execution passes it. You can assign a label to another label or a literal
```clojure

.loop ->
    a = 2 ; a will be set to 2 every iteration of the loop
    a -= a .loop

; Label to literal
a = 2 
; is equivalent to
_ASM -= _ASM
a -= a .assign
a -> 0 ; declaration
.assign -> {
    lit -> 2
    _ASM -= lit
    name? -= _ASM
}

; Label to zero (special optimised case)
b = 0
; is equivalent to
b -= b .fin
b -> 0 ; declaration
.fin ->

; Label to Label
b = a
_ASM -= _ASM .assign
b -> 0 ; declaration
.assign ->
b -= b
_ASM -= a
b -= _ASM

; Note that there is a small memory and performance cost to assignments
```




## Namespacing
The format `Namespace::Macro` or `Namespace::label` should be used. This is solely a naming convention and not enforced in any way. This means that module authors must decide what namespace their macros or labels should have. This is obviously bad design, but it is simple.

## Sublib
Sublib is the standard library. It has a range of very basic features (Prelude.sbl, IO.sbl and Symbols.sbl) to quite advanced ones like functions and control flow.

## Style guide
Adhere to the naming conventions and type system and make sure it looks good :), ideally you should follow the style of the Sublib


## Examples
### Basic
```clojure
; Very basic Sublang, without using the standard library Sublib
; Output: Hello, World!

!Print p_string ; Call the macro Print


Z -= Z -1 ; Jumping to -1 halts, equivalent to !Halt



p_string -> &"Hello, World!\n"
Z -> 0 ; Temp register
N_ONE -> -1 ; Store the literal negative one

**
    Pure no dependency implementation of print
**
@Print P_STRING? {


    ; Copy the pointer into the local variable ptr, because we don't want to
    ; modify the original pointer
    Z   -= Z ; clear Z
    Z   -= P_STRING? ; Z = -P_STRING?
    ptr -= Z ; ptr = -Z = --P_STRING? = P_STRING?

    Z -= Z
    .loop ->
        char -= char ; Clear char
        Z -= (ptr -> 0) ; Z -= *ptr, dereferences ptr to get the actual character

        char -= Z .fin ; Flip the character, since it is negative, and jump if
                       ; result is LEQ zero (i.e. finish if it is a ZERO/NULL)
        -1 -= char ; Writes the character to the screen. -1 is a special register used
                   ; for IO operations

        ptr -= N_ONE ; Increment the pointer to go to the next character
        Z -= Z .loop ; Infinite loop

    char -> 0 ; This point is never reached, so it is safe to define the
              ; label 'char' here. It is very important to keep in mind
              ; that, in this case the zero, will be put in memory in
              ; this exact place and, if execution crosses it, it will
              ; be interpreted as an instruction. To define values in between
              ; instructions, use the '=' operator

    .fin ->
}
```

### Sublib

```clojure
; This is how Sublang could should be written, making extensive use of macros
; Output: Hello, Sublang!

#sublib
#sublib/Control

p_string -> &"Hello, Sublang!\n"

**
   Print a string using macros from standard lib
**
@PrintStdLib P_STRING? {
    p_local = P_STRING?
    char = 0

    !Loop {
        !DerefAndCopy p_local char ; char = *p_local
        !IfFalse char {
            !Break
        }
        !IO -= char
        !Inc p_local
    }
}

; Executing starts here
.main -> {
    !PrintStdLib p_string
    !Halt
}
```
```clojure
; Or you can just use one of the Print macros from sublib/IO
#sublib

.main -> {
    IO::PrintLnLit "Hello, Sublib!"
    !Halt
}
```

### Conway's Game of Life
[./subleq/examples/GameOfLife.sbl](github.com/Kat9-123/asa/tree/master/subleq/examples/GameOfLife.sbl)




## Conclusion
For many more examples see Sublib or the end-to-end tests, though they are messy and not idiomatic
