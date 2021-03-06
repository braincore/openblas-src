# openblas-src [![Package][package-img]][package-url] [![Documentation][documentation-img]][documentation-url] [![Build][build-img]][build-url]

The package provides a source of [BLAS] and [LAPACK] via [OpenBLAS].

The usage of the package is explained [here][usage].

The following Cargo features are supported:

* `cblas` to build CBLAS (enabled by default),
* `lapacke` to build LAPACKE (enabled by default),
* `static` to link to OpenBLAS statically, and
* `system` to skip building the bundled OpenBLAS.

## Cross Compilation

To this end, one has to specify the [cross-compilation variables for OpenBLAS](
https://github.com/xianyi/OpenBLAS#cross-compile) but with the `OPENBLAS_`
prefix: `OPENBLAS_CC`, `OPENBLAS_FC`, `OPENBLAS_HOSTCC`, and `OPENBLAS_TARGET`.
These can be set as environment variables for `cargo build`.

## Contribution

Your contribution is highly appreciated. Do not hesitate to open an issue or a
pull request. Note that any contribution submitted for inclusion in the project
will be licensed according to the terms given in [LICENSE.md](LICENSE.md).

[blas]: https://en.wikipedia.org/wiki/BLAS
[lapack]: https://en.wikipedia.org/wiki/LAPACK
[openblas]: http://www.openblas.net/
[usage]: https://blas-lapack-rs.github.io/usage

[build-img]: https://travis-ci.org/blas-lapack-rs/openblas-src.svg?branch=master
[build-url]: https://travis-ci.org/blas-lapack-rs/openblas-src
[documentation-img]: https://docs.rs/openblas-src/badge.svg
[documentation-url]: https://docs.rs/openblas-src
[package-img]: https://img.shields.io/crates/v/openblas-src.svg
[package-url]: https://crates.io/crates/openblas-src
