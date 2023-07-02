# E-Voting Verifier in Rust

## Introduction

This application implements a Verifier for the E-Voting system of Swiss Post. It is based on the specifications of Swiss Post, according to the following document versions:

- [Crypo-primitives](https://gitlab.com/swisspost-evoting/crypto-primitives/crypto-primitives), version 1.3.0
- [Verifier](https://gitlab.com/swisspost-evoting/verifier/verifier), version 1.4.0

The verifier ist implemented for the version 1.2.3 of the E-Voting system of Swiss Post

## Information about the project

###  Structure of the project

The project contains the following crates:

- crypto_rimitives: The implementation of necessary methods, incl. a wrapper to opennssl
- rust_verifier_lib (in directory ./lib): The implementation of the functionalities and verifications
  - data_structures: The implementation of the data used for the verifier. The data can be read from the files using serde
  - file_structure: The structure of the files and directory.
  - verification: The implementation of each verification.
  - The handling of xml files is at the beginning.
- rust_verifier_console (in directory ./console): The implementation of a console application to execute the verification
- rust_verifier_gui (in directory ./gui): The implementation of a gui application with the framework [Tauri](https://tauri.app/) to execute the verification

###  Difference to the Swiss Post implementation

The implementation not used any code of Swiss Post.

A major difference to Swiss Post is that the verifications does not return true or false, but return all the errors and failures found. In that case it helps a better granularity to found a reason of a problem.

###  Future works

- Not all the tests are implemented
- For many tests, the negative unit tests are not implemented
- The tests are running sequentially. A parallelism should be introduced (e.g. using Rayon)
- Gui is started but functionalities must be implemented
- The manual verifications are not implemented
- The report of the tests is not generated

## Development guide

The application is tested with the version 1.69.1 of rust.

The application use the crate openssl to wrap the functions of the library openssl. Please check the installation guide of the create.

Generate the doc to see the documentation of the modules:

```shell
cargo doc
```

## Licence

Open source License Apache 2.0

See [LICENSE](LICENSE)

## Third party

See [THIRD_PARTY](THIRD_PARTY)
