## Introduction to .sbl

### Basics

```clojure
1 2 3 ; Standard subleq, eq to mem[2] -= mem[1] jump to 3 if leq otherwise next instruction
; memory adresses may be labeled
a -> 1
b -> 2
a -= b ; Subtract b from a
a -= b c ; Subtract b from a and jump to c if the result is less than or equal to zero

; If no `c` is given, the next instruction will always be executed, even if the result is leq

; These two are equivalent
a -= b
a -= b &1 ; `&1` gives a relative address with offset one

a -= b ; is equivalent to
b a &1 ; This syntax works but is not recommended, since it makes it harder for the assembler to give hints

; Other examples
a -= b 0x0000
0x0000 -= 0x0000 c

```

### Labels
```clojure
a -> 123
b -> 0x456
c -> 9

.label ->
    a -= b .label

```

### Scopes
Scoping works like in most other languages. Note: Only labels can be scoped. Macro definitions can't.
```clojure
Z -> 123
X -> 456
Y -> 0
{
    Z -> 789
    X -= Z  ; 456 - 789
}
Y -= Z ; 0 - 123



```


### Types
The assembler has a simple type-checker, which can be disabled.

* `value` normal label
* `p_value` pointer (not type-checked)
* `p_p_value` pointer to pointer (not type-checked)
* `n_value` negated value (not type-checked)
* `l_value` literal value
* `s_value` scoped value
* `a_value` anything, no type checking
* `m_value` (advanced) a macro call passed as argument, must be braced.

These naming conventions are, were possible, checked by the assembler.

### Naming conventions
* `@MyMacro` macros in CamelCase
* `my_label` labels in snake_case, with the exception of single character 'registers', like `Z` or `W`
* `MyModule.sbl` modules (files) are in CamelCase
#### Labels
* `value?`  macro argument in definition
* `.value` label to jump to
* `CONST_VALUE`constant, can be applied to all of the above, and should be applied to macro arguments*
* `Module::Value` or `!Module::Macro`
* `Module::SubModule::value`


### Macros
#### Definition
```clojure
@Name {
    ...
}
; with args
@Name a? b? c? {
    ...
}

; It is also possible to define a macro that isn't scoped:
@Name a? [
    ...
]
; This, however, is dangerous when label definitions take place in the macro, , so it is discouraged.


```
#### Calling
```clojure
!Name
!Name a b c
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
?MyMacro?a -= a
?MyMacro?a -> 123
```
### Pointers
#### Referencing
To create a pointer to a value

```clojure
ptr -> &1 0x1234

ptr -> &1 "String"
; or
ptr -> &'A'
ptr -> &"String" ; Doesn't work for Hex or Dec literals, for those there must be a space in beteeen the & and the number
pre -> & 123
; because &123 would be a relative address +123
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
Remember that because of how Subleq works, what are called 'Labels' here, are also just pointers! But since Subleq dereferences them, we can think of them as values.


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
If you want to create a module (a set of .sbl files in a folder) you must create a folder with the name of the module (for example 'Sublib'). And in that folder create a .sbl file with the module name (like 'Sublib.sbl'). Whenever someone imports the 'Sublib' folder, this is automatically resolved to 'Sublib/Sublib.sbl'. In this .sbl folder you may include any other files you might need.

See subleq/Sublib for an example.

### Miscellaneous
#### Mult operator
When the '*' is placed before a literal ?? The previous token is repeated n times
```clojure
label * 3 ; =>
label label label

0x123 * 0x2 ; =>
0x123 0x123

```

### Sublib
Sublib is the standard library. It has a range of very basic features (Base.sbl, IO.sbl and Symbols.sbl) to quite advanced ones like functions and control flow.

### Advanced
#### Macro 'currying'
It is possible to 'curry' (using that term loosely) macros