# E-Voting Verifier in Rust

## Introduction

This crate is the main library for the E-Voting system of Swiss Post.

It is based on the specifications of Swiss Post, according to the following document versions:

- [Crypo-primitives Specifications](https://gitlab.com/swisspost-evoting/crypto-primitives/crypto-primitives/-/blob/master/Crypto-Primitives-Specification.pdf?ref_type=heads)
- [System Specifications](https://gitlab.com/swisspost-evoting/e-voting/e-voting-documentation/-/blob/master/System/System_Specification.pdf)
- [Verifier Specifications](https://gitlab.com/swisspost-evoting/e-voting/e-voting-documentation/-/blob/master/System/Verifier_Specification.pdf?ref_type=heads)

The verifier is implemented for the version 1.4.3 of the E-Voting system of Swiss Post.

This crate is used as basis for a GUI application.

Following application are implemented:
- A console application [rust_ev_verifier_console](https://github.com/de-mo/rust_ev_verifier_console)
- A GUI application based on tauri ([backend](https://github.com/de-mo/rust_ev_verifier_gui_backend) / [frontend](https://github.com/de-mo/rust_ev_verifier_gui))

## Information about the project

###  Difference to the Swiss Post implementation

The implementation not used any code of Swiss Post. It is only based on the published documentation.

A major difference with the Swiss Post Verifier is that the verifications does not return true or false, but return all the errors and failures found, with the necessary information in regard to the position of the element, which generates the error. In this case it helps a better granularity for the analysis of the errors and failures.

###  Future works

The Verifier is not ready for production.

- The verification of the signature of XML files is missing
- The verification of the data in the file eCH-0222 is missing
- For most of the verifications, the negative unit tests are not implemented. A mechanisms of mocks is implemented
- The report of the verifications is not generated

## Usage

See the [crate documentation](https://docs.rs/crate/rust_ev_verifier_lib/0.2.2)

## Licence

Open source License Apache 2.0

See [LICENSE](LICENSE)

