# E-Voting Verifier in Rust

## Introduction

This crate is the main library and the console application for the E-Voting system of Swiss Post.

It is based on the specifications of Swiss Post, according to the following document versions:

- [Crypo-primitives](https://gitlab.com/swisspost-evoting/crypto-primitives/crypto-primitives), version 1.3.0
- [Verifier](https://gitlab.com/swisspost-evoting/verifier/verifier), version 1.4.0

The verifier ist implemented for the version 1.2.3 of the E-Voting system of Swiss Post

## Information about the project

###  Structure of the project

The library contains the following modules:

- data_structures: The implementation of the data used for the verifier. The data can be read from the files using serde
- file_structure: The structure of the files and directory.
- verification: The implementation of each verification.
- application_runner: The runner that can be used by another gui or application

###  Difference to the Swiss Post implementation

The implementation not used any code of Swiss Post.

A major difference to Swiss Post is that the verifications does not return true or false, but return all the errors and failures found. In that case it helps a better granularity to found a reason of a problem.

###  Future works

- Not all the tests are implemented
- For most of the tests, the negative unit tests are not implemented
- Gui is implemented, but look & feel should be improved
- The manual verifications are not implemented
- The report of the tests is not generated
- Add functionality to read zip file (to check hashes)

## Development guide

The library depends from the crate [rust_ev_crypto_primitives](https://github.com/de-mo/rust_ev_crypto_primitives.git).

Generate the doc to see the documentation of the modules:

```shell
cargo doc
```

## Licence

Open source License Apache 2.0

See [LICENSE](LICENSE)

## Third party

See [THIRD_PARTY](THIRD_PARTY)
