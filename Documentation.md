
The assembler has a simple type-checker, which can be disabled.

## Introduction


### Basics

```clojure
a -= b ; Subtract b from a
a -= b c ; Subtract b from a and jump to c if the result is less than or equal to zero

; If no `c` is given, the next instruction will always be executed

; These two are equivalent
a -= b
a -= b &1 ; `&1` gives a relative address with offset one

a -= b ; is equivalent to
b a &1 ; This syntax works but is not recommended

; Other examples
a -= b 0x0000
0x0000 -= 0x000 c

```

### Labels
```clojure
a -> 123
b -> 0x456
c -> 9

.label ->
    a -= b .label

```


### Types
* `p_value` pointer
* `p_p_value` pointer to pointer
* `n_value` negated value
* `l_value` literal value
* `s_value` scoped value
* `a_value` anything, no type checking

### Naming conventions
* `value?`  macro argument in definition
* `.value` label to jump to
* `CONST_VALUE`constant, can be applied to all of the above
* `Module::Value` or `!Module::Macro`
* `Module::SubModule::Value`


### Macros
#### Definition
```clojure
@Name {


}
; with args
@Name a? b? c? {

}

```
#### Calling
```clojure
!Name
!Name a b c
```

### Pointers
#### Referencing
To create a pointer to a value

```clojure
ptr -> &1 0x1234

ptr -> &1 "String"
; or
ptr -> &'A'
ptr -> &"String" ; Doesn't work for Hex or Dec literals
```

#### Dereferencing
```clojure
; Generic method for dereferencing. The value that 'b' points to will be subtracted from 'a'
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


### Miscellaneous
#### Mult operator
When the '*' is placed before a literal ?? The previous token is repeated n times
```clojure
label * 3 ; =>
label label label

0x123 * 0x2 ; =>
0x123 0x123

```