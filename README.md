# orbbec-sdk-sys

This crate provides low-level bindings to [Orbbec SDK v2](https://github.com/orbbec/OrbbecSDK_v2), more specifically the C API. The library provides support for Orbbec RGBD and TOF cameras. The version numbers correspond to the version numbers of the underlying library.

The crate builds the Orbbec SDK from source. Contributions to optionally link to the system installation instead are welcome.

## Installation

This crate can be added with `cargo add orbbec-sdk-sys`. The Orbbec SDK will be built from source. Once per computer, [environment setup](https://github.com/orbbec/OrbbecSDK_v2/blob/main/docs/tutorial/building_orbbec_sdk.md#environment-setup) may need to be performed.

## Usage

There are examples in the [examples/](./examples/) folder. The examples here correspond to the [C examples in upstream](https://github.com/orbbec/OrbbecSDK_v2/tree/main/examples/c_examples).


## Licence

This crate is published under the MIT license, just as the underlying Orbbec SDK v2. For more details, see [LICENSE.md](./LICENSE.md).