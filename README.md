# rtsc

## Introduction

The **rtsc** is a compiler written in rust for the subset of the TypeScript
language that generates ARM32 assembly. It is based on the book written by Vladimir Keleshev titled
"Compiling to Assembly".

It is currently in a *work in progress* state.

## How to use?
There is an exemplary source file (`main.ts`) that can be compiled using the `compile-main.sh` bash script. I compile on the *X86-64* PC.
```
> apt-get install qemu-user
> apt-get install gcc-arm-linux-gnueabihf
> cargo run -- main.ts > main.s
> arm-linux-gnueabihf-gcc -static main.s -o main
> ./main
```

## What are the differences in contrast to the book implementation?
- I used an `enum` to represent different AST nodes, instead of separate classes
that would implement a trait.  
- I created parser combinators using functions only, as opposed to having 
`struct` with methods. This was a bad idea because the resulting code is
terrible and using functions leads to many weird things. In essence, the parser
is terrible but it works. I might rewrite it at some point.
