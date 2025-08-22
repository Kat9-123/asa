# Introduction to Sublang
Sublang is a bare bones assembly languages consisting of three main elements:
* The SUBLEQ instruction
* Labels
* Macros
It also has some syntax sugar.


### Subleq

```clojure
1 2 3 ; This is interpreted as standard subleq: mem[2] -= mem[1] jump to 3 if LEQ otherwise it goes to the next instruction.
; This syntax is valid, but pointless.

; memory addresses and instructions may be labeled
a -> 1
b -> 2
c ->
    a -= b ; Subtract b from a
    a -= b c ; Subtract b from a and jump to c if the result is less than or equal to zero

; If no `c` argument is given, the next instruction will always be executed, even if the result is LEQ
; So these two are equivalent
a -= b
a -= b $1 ; `$1` gives a relative address with offset one

a -= b ; is equivalent to
b a $1 ; This syntax works but is not recommended, since it makes it harder for the assembler to give hints

; Other examples, literals and labels may freely be combined
a -= b 0x0000
'\0' -= 0 c
```

### Labels and Literals
```clojure
a -> 123 ; Decimal
b -> 0x4C6 ; Hex
c -> "Hello, World!" ; Strings are null-terminated by the assembler
d -> 'P' ; Character literals

.label ->
    a -= b .label   ; Repeats as long as (a -= b) <= 0
```

### Scopes
Scoping works like in most other languages. Note: Only labels are effected by scopes, macro definitions in scopes will still be globally accessible
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


### Types
The assembler has a simple type-checker, which can be disabled.

* `value` normal label
* `l_value` literal value
* `s_value` scoped value
* `a_value` anything, no type checking
* `m_value` a macro call passed as argument, must be braced.

Currently types are only checked checked for macro parameters.

### Naming conventions
* `@MyMacro` macros in CamelCase
* `my_label` labels in snake_case, with the exception of single character 'registers', like `Z` or `W`
* `MyModule.sbl` modules (files) are in CamelCase
#### Labels
* `p_value` pointer (not type-checked)
* `p_p_value` pointer to pointer (not type-checked)
* `n_value` negated value (not type-checked)
* `value?`  macro argument in definition
* `.value` label to jump to
* `CONST_VALUE` constant, can be applied to all of the above and should be applied to macro arguments, but NOT to literals (l_name), since they are always constant by definition
* `Module::Value` or `!Module::Macro` namespacing
* `Module::SubModule::value`


### Macros
#### Definition
```clojure
@Name {
    ...
}
@Name   ; This is allowed as well, but discouraged
{

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

; There may be linebreaks between parameters

@Name a?
      b?
      c?
      d?
      e? {
    ...

}

```
#### Expanding
```clojure
!Name
; With arguments
!Name2 a b c
; There may NOT be linebreaks between arguments
```

#### Hygiene
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

#### Macro arguments
You may pass scopes as macro arguments

```clojure
!Mac s_my_scope? {
    s_my_scope?
}


!Mac { a -= b } 
; =>
{
    { a -= b }
}

; If a macro takes multiple scopes they can be chained as follow:
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

### Pointers
#### Referencing
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

#### Dereferencing
```clojure
; Generic sequence for dereferencing. The value that 'b' points to will be subtracted from 'a'
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
Remember that because of how Subleq works, what are called 'Labels' here, are also just pointers! But since Subleq dereferences them, we can think of them as values. But mind that literals need indirection `a -= 10` doesn't subtract 10 from a


### Inclusions
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
If you want to create a module (a set of .sbl files in a folder) you must create a folder with the name of the module (for example 'Sublib'). And in that folder create a Lib.sbl. Whenever the 'Sublib' folder is imported, this is automatically resolved to 'Sublib/Lib.sbl'. In this .sbl file you may include any other files you might need.

See subleq/libs/Sublib for an example.

### Syntax sugar
#### Mult operator
When the '*' is placed before a literal ?? The previous token is repeated n times
```clojure
label * 3 ; =>
label label label

0x123 * 0x4 ; =>
0x123 0x123 0x123 0x123

; mind that 3 * label will dereference `label`!
```
#### Dereference operator

#### Assignment
The '=' can be used to declare a label, and assign it a value. The value may be a literal or another label. every time the assignment is executed the value is reset to the given value
```clojure
label = 10
; =>
_ASM -= _ASM
label -= label $3
label -> 0
{
    n_lit -> 10
    _ASM -= n_lit
    label -= _ASM
}

; Zero is a special case
label2 = 0
; =>
label2 -= label2 $2
label2 -> 0


; Another label
label3 = label2
; =>
_ASM -= _ASM $2
label3 -> 0
label3 -= label3

_ASM -= label2
label3 -= _ASM
```


## Style guide
Just make sure it looks good :), or follow the style of the Sublib

## Namespacing
The format `Namespace::Macro` or `Namespace::label` should be used. This is solely a naming convention and not enforced in any way. This means that module authors must decide what namespace their macros or labels should have. This is obviously bad design, but it is simple.

## Conclusion
For many more examples see the standard library, called Sublib



# Sublib
Sublib is the standard library. It has a range of very basic features (Base.sbl, IO.sbl and Symbols.sbl) to quite advanced ones like functions and control flow.
