# Ownership and Borrowing in Flame

Flame's memory management and safety guarantees are powered by its Rust-like ownership and borrowing model. This system ensures memory safety without needing a garbage collector and guarantees that data races cannot occur.

## Ownership Rules

1. Each value in Flame has a variable that’s called its **owner**.
2. There can only be one owner at a time.
3. When the owner goes out of scope, the value will be dropped.

```flame
let s = String.from("hello") // s is the owner
take_ownership(s)
    
// print(s) // Error: `s` moved, no longer valid here!

fn take_ownership(some_string: String) {
    print(some_string)
}
```

## References and Borrowing

Instead of transferring ownership, you can **borrow** values by passing a reference. A reference allows you to refer to a value without taking ownership of it.

### Immutable References (`&T`)

You can create an immutable reference using `&`. You can have any number of immutable references to a value simultaneously.

```flame
fn boo(s: String) -> String {
    s
}
let s1 = "hello"
let len = boo(&s1) // Passed as an immutable reference
print($"The boo of {s1} is {len}")
```

### Mutable References (`&mut T`)

To modify a borrowed value, you must use a mutable reference using `&mut`.

**Important Rule:** You can only have **one** mutable reference to a particular piece of data in a particular scope at a time.

```flame
fn change(&mut some_string: String) {
    some_string.push_str(", world")
}
let mut s = ("hello")
change(&mut s)
print(s) // Prints: hello, world

```

## The Borrowing Rules

At any given time, you can have *either*:
- One mutable reference (`&mut T`).
- Any number of immutable references (`&T`).

References must always be valid (no dangling pointers). flame's compiler guarantees this at compile time.

## `mut` and Function Signatures

When declaring a variable, it is immutable by default. You must use `mut` to make it mutable.

```flame
let a = 5
// a = 6 // Error!

let mut b = 5
b = 6 // OK
```

When passing parameters to a function, if the function needs to modify the value in-place, the parameter must be defined as `&mut T`.

```flame
fn process_data(data: &mut Vec<U64>) {
    data.push(42)
}
```
