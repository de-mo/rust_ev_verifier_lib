# E-Voting Verifier in Rust

## Introduction

This crate is the main library for the E-Voting system of Swiss Post.

It is based on the specifications of Swiss Post, according to the following document versions:

- [Crypo-primitives](https://gitlab.com/swisspost-evoting/crypto-primitives/crypto-primitives), version 1.3.1
- [Verifier](https://gitlab.com/swisspost-evoting/verifier/verifier), version 1.4.0

The verifier is implemented for the version 1.4.3 of the E-Voting system of Swiss Post.

## Information about the project

###  Structure of the project

The library contains the following modules:

- [data_structures](src/data_structures/mod.rs): The implementation of structure of the data used for the Verifier. The data are red from the files using [serde](https://docs.rs/serde/latest/serde/)
- [file_structure](src/file_structure/mod.rs): The necessary functions to implement the files and directories
- [verification](src/verification/mod.rs): The implementation of each verification.
- [application_runner](src/application_runner/mod.rs): The runner that can be used by a gui or an application to run all the verifications.

###  Difference to the Swiss Post implementation

The implementation not used any code of Swiss Post. It is only based on the published documentation.

A major difference with the Swiss Post Verifier is that the verifications does not return true or false, but return all the errors and failures found, with the necessary information to regarding the position of the element, that generates the error. In this case it helps a better granularity for the analysis of the errors and failures.

###  Future works

The Verifier is not ready for production.

- Not all the verifications are implemented
- For most of the verifications, the negative unit tests are not implemented. A mechanisms of mocks is implemented
- The manual verifications are not fully implemented
- The report of the verifications is not generated
- XML Files: Read, decode and control of signature must be implemented

## Development guide

The library depends from the following crates
- [rust_ev_crypto_primitives](https://github.com/de-mo/rust_ev_crypto_primitives)
- [rust_ev_system_library](https://github.com/de-mo/rust_ev_system_library)

Copy the directoy `/datasets/direct-trust` to the root.

## Licence

Open source License Apache 2.0

See [LICENSE](LICENSE)

