# multi-rpc Workspace

This repository contains the `multi-rpc` Rust project, a library that allows you to define a service trait once and serve it over multiple RPC protocols simultaneously.

## Crates

* **`multi-rpc`**: The primary library crate that you will use as a dependency. It provides the core builder and procedural macros to set up your services. For detailed usage and examples, please see its dedicated README.
    * [**➡️ Go to the `multi-rpc` README](./multi-rpc/README.md)**
* **`multi-rpc-macros`**: An internal crate that implements the procedural macros used by `multi-rpc`. You should not need to use this crate directly.
* **`examples/`**: Contains several example crates demonstrating how to set up a server and clients for each supported protocol.

For instructions on how to use the library, please refer to the README in the `multi-rpc` crate directory.