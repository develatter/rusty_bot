# Dependency Licenses

**The `rusty_bot` project itself is licensed under the GNU General Public License, version 3 (GPLv3). A copy of the GPLv3 can be found in the `LICENSE` file in the root of this project.**

This project uses the following third-party dependencies. Their respective licenses are detailed below.

## Main Licenses (MIT or MIT OR Apache-2.0)

The following crates are licensed under the terms of the MIT license or a choice between the MIT license and the Apache-2.0 license:

*   dioxus - MIT OR Apache-2.0
*   kalosm - MIT OR Apache-2.0
*   tokio - MIT
*   web-sys - MIT OR Apache-2.0
*   wasm-bindgen - MIT OR Apache-2.0
*   futures - MIT OR Apache-2.0
*   serde - MIT OR Apache-2.0

## Specific Licenses

### comrak

*   **License:** BSD-2-Clause

### surrealdb

*   **License:** Non-standard.
    Source code for SurrealDB is variously licensed under a number of different licenses. A copy of each license can be found in each repository.

    *   Libraries and SDKs, each located in its own distinct repository, are released under either the Apache License 2.0 or MIT License.
    *   Certain core database components, each located in its own distinct repository, are released under the Apache License 2.0.
    *   Core database code for SurrealDB, located in this repository, is released under the Business Source License 1.1.
        The BSL 1.1 allows free use, modification, and scaling of SurrealDB, including embedding it in applications for production use. The primary restriction is against offering a commercial version of SurrealDB itself as a managed Database-as-a-Service (DBaaS). After four years from release, BSL-licensed code converts to the Apache License 2.0.

    For more information, see the [SurrealDB licensing information](https://github.com/surrealdb/license).
