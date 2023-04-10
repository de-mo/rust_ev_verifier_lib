# E-Voting Verifier in Rust

## Introduction

This application implements a Verifier for the E-Voting system of Swiss Post. It is based on the specifications of Swiss Post, according to the following system versions:

- [Crypo-primitives](https://gitlab.com/swisspost-evoting/crypto-primitives/crypto-primitives), version 1.2.1.0
- [Verifier](https://gitlab.com/swisspost-evoting/verifier/verifier), version 1.3.3.0

## Information about the project

###  Structure of the project

The project contains the following main modules:

- crypto_rimitives: The implementation of necessary methods, incl. a wrapper to opennssl
- data_structures: The implementation of the data used for the verifier. The data can be read from the files using serde
- file_structure: The structure of the files and directory.
- verification: The implementation of each verification

###  Difference to the Swiss Post implementation

The implementation not used any code of Swiss Post.

A major difference to Swiss Post is that the verifications does not return true or false, but return all the errors and failures found. In that case it helps a better granularity to found a reason of a problem.

## Development guide

The application is implemented with the version 1.67.0 of rust.

Generate the doc to see the documentation of the modules:

```shell
cargo doc
```

## Licence

Copyright 2023 Denis Morel

Licensed under the Apache License, Version 2.0 (the "License"); you may not use this file except in compliance with the License. You may obtain a copy of the License at

<http://www.apache.org/licenses/LICENSE-2.0>

Unless required by applicable law or agreed to in writing, software distributed under the License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied. See the License for the specific language governing permissions and limitations under the License.

## Third party

See [THIRD_PARTY](THIRD_PARTY)
