# simple-compiler [![Travis Build Status](https://api.travis-ci.org/marionauta/strategies.svg)][0]

A simple compiler for a very simple toy language. This is a **learning project**
that focuses on simplicity to keep things **easy to understand**. There will be
things that could be done more efficiently, but that's not the point.

What I want is to write a little compiler with _hopefully_ nothing more than
[Rust][1]'s standard library.

## The language

I didn't name it, as that isn't important. The aim is to make a simple language
that transpiles to C, making type definition a bit easier. This little snippet:

```
tipo Circulo(centro: Punto, radio: Real);
tipo Punto(x: Entero, y: Entero);
```

Will produce this output:

```c
typedef struct Punto {
    long x;
    long y;
} Punto;

typedef struct Circulo {
    Punto centro;
    double radio;
} Circulo;
```

Note that the order in the input language doesn't matter, it will produce C code
that compiles nicely.

## Installing

To build `simple-compiler` you need:

- Rust `1.31.0` or better.
- Cargo

And just run `cargo build --release`

## Inspiration

* [**The Super Tiny Compiler**][2]: Inspired me to do this.
* [**Writing An Interpreter In Go**][3]: Using only the tools in the standard
library is an interesting concept.

[0]: https://travis-ci.org/marionauta/simple-compiler
[1]: https://www.rust-lang.org
[2]: https://github.com/thejameskyle/the-super-tiny-compiler
[3]: https://interpreterbook.com/
