# Lexum-compiler

Frontend and compilation pipeline for the Lexum programming language.

## Overview

`Lexum-compiler` transforms Lexum source code into executable bundles for the
Lexum runtime.

The compiler pipeline is expected to include:

- lexical analysis and parsing
- abstract syntax tree construction
- semantic validation and authority checks
- goal graph and convergence analysis
- lowering into deterministic intermediate representation
- bytecode generation and bundle packaging

Lexum programs describe control-plane behaviour rather than application logic.
The compiler therefore emphasizes correctness, structural clarity, and
deterministic execution semantics.

## Design Goals

- Strong static validation of orchestration intent
- Explicit authority and scope hierarchy enforcement
- Deterministic control-flow normalization
- Support for persistent state schema evolution
- Clear diagnostics for long-running system behaviour

## Status

Language syntax and semantics are under active design.

Initial milestones include:

- parser skeleton
- AST representation
- IR design validation
- minimal bytecode emitter

## Contributing

Compiler architecture decisions are evolving.
Discussion via issues is encouraged before submitting large PRs.

## License

Apache License 2.0
