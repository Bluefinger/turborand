# Migration Notes

This document will keep tabs on migration paths between breaking change versions of `turborand`.

## Migration from 0.5 to 0.6

Version 0.6 introduces a major reworking of the crate, with code reorganised and also exposed more granularly via features. First things to note:

- All major exports for the crate are now in the `prelude` module. Top level only exports the new traits for `turborand`.
- `Rng` is now split into `Rng` and `AtomicRng`, no more top level generics that required exporting internal traits. `State` trait is now made private and no longer available to be implemented, as this was an internal implementation detail for `WyRand`.
- All previous methods for `Rng` are now implemented in `TurboCore`, `GenCore`, `SeededCore` and `TurboRand` traits. These are part of the `prelude` so as long as they are included, all existing methods will work as expected.
- `Rng` is now under a feature flag, `wyrand`. This is enabled by default however, unless `default-features = false` is applied on the dependency declaration in Cargo.toml.
- _Yeet_ the `rng!`, `atomic_rng!` macros, as these are no longer needed to manage the generics spam that has since been refactored out. Instead, use `::new()`, `::default()` or `::with_seed(seed)` methods instead.

## Migration from 0.6 to 0.7

Version 0.7 hasn't changed much except that the internals module is now fully private (so the `State` traits and `CellState`/`AtomicState` structs are no longer public). They are not accessible from the prelude any more. The removal of these from the public API thus constitutes a breaking change, leading to a new major version.

Also, the serialisation format of `ChaChaRng` has changed, so 0.7 is not compatible with older serialised structs. The plus side is also a flatter serialised format for `ChaChaRng`. Also, `ChaChaRng` is no longer backed by a `Vec` for caching generated entropy, now preferring to use an aligned array for better random number generation at the slight cost of initialisation/cloning performance and increased struct size. This means that the single heap allocation `ChaChaRng` needed is now reduced to zero.

## Migration from 0.7 to 0.8

Version 0.8 seperates the old `Clone` behaviour into two: standard `Clone` which maintains the original state and clones it to the new instance as is (and so both old and new equal to each other), and `ForkableCore` which mutates the state of the original to _fork_ a new instance with a random state generated from the original. **Previous usage of `.clone()` now should make use of `.fork()` instead**. Cloning now should be used where preserving the state of the original to the cloned instance is required.

## Migration from 0.8 to 0.9

Version 0.9 introduces `no-std` compatibility with more granular features as well as minor changes to `weighted_sample`.

For `no-std` compatibility, new feature flags have been created. By default, `std` feature flag is enabled and with `fmt` providing `Debug` implementations. Without default features, `turborand` will expose only methods and implementations that are compatible with `no-std` environments. `alloc` is provided as a feature flag for enabling some methods like `sample_multiple`, which require at least `alloc` crate support. Traits like `Default` and methods like `new()` are only supported in `std` environments.

For `weighted_sample` and `weighted_sample_mut`, the `weight_sampler` signature has changed from `Fn(&T) -> f64` to `Fn((&T, usize)) -> f64`. The tuple provides not just a reference to the sampled item, but the index as well. There's also a correction to the `weighted_sample` and `weighted_sample_mut` lifetimes which should fix some typing issues.

Other minor changes include some removal of `unsafe` that are no longer necessary with some internal refactors, as well as `sample_iter` and `sample_multiple_iter` methods.

## Migration from 0.9 to 0.10

Version 0.10 introduces `GenCore::GEN_KIND` associated constant, needed to be able to toggle between different algorithms for some methods which have different optimum profiles based on the speed of the PRNG and the algorithm itself. `Rng` and `ChaChaRng` now use different shuffling algorithms, with `ChaChaRng` changing compared to previous releases. The internal implementation of `ChaChaRng` has also changed, enabling better perf with a more standard ChaCha implementation.