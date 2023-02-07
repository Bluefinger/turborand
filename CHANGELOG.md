## [unreleased]

### Chore

- Prepare release

## [0.10.0] - 2023-02-07

### Chore

- Dedicated migration notes document

### Feat

- Stable indexing method and sampling
- Optimise shuffle for different algorithms

### Perf

- Tuning and inlining tweaks

### Refactor

- Standardize internal implementation
- Use range for index method
- Tidied char impl with from_32 method

## [0.9.0] - 2023-01-19

### Chore

- Fix formatting
- Prepare release

### Feat

- No-std compatibility and finer-grained features (#29)
- New sampling iterator methods

### Perf

- Weighted sampling improvements & benches

### Refactor

- Remove some unnecessary unsafe

## [0.8.3] - 2022-12-20

### Chore

- Prepare release

### Feat

- Fix f64 codegen issues

### Fix

- Field enum for deserialising

## [0.8.2] - 2022-11-22

### Chore

- Add more unit tests
- Prepare for release

### Perf

- Improve buffer on large or known slice sizes
- Simplify while loop for fill

## [0.8.1] - 2022-11-17

### Chore

- No default criterion features in wasm
- Explicit criterion features for non-wasm targets
- Prepare for release

### Feat

- Implement _mut sampling methods

### Perf

- Improve fill_bytes throughput

## [0.8.0] - 2022-10-26

### Chore

- Format code
- Prepare for release

### Refactor

- Separate forking and cloning behaviours

## [0.7.0] - 2022-09-28

### Chore

- Format code
- Improve safety messages
- Prepare release v0.7

### Refactor

- Revamped EntropyBuffer, better serialization
- Make internals private, fix perf regression
- Use split_at_mut in fill method
- Reorganise internals code

## [0.6.0] - 2022-08-18

### Chore

- Fix Miri flags
- Add note about removal of macros.
- Prepare for release

### Doc

- Improve clarity of feature listing

### Docs

- Added traits & SecureRng docs, plus minor tweaks
- Update description of crate
- Document migration notes in README

### Feat

- Basic implementation of ChaCha8 source.
- Traitification, moved method impls to set of traits
- Clone & Default impl, switch to Cell & clippy lints
- Init macro, fix some docs
- Expose new RNG under feature flag
- Better quality entropy sources with fallback
- Provide borrowed interface for rand compatibility
- Serialise feature & AlignedSeed optimisation
- Serialisation,  plus tidy-ups

### Fix

- Don't leak state via Debug
- Empty buffer on reseed, add safety comments
- Don't reset buffer on block state overflow
- Fix Miri arguments once more
- Fix rand and secure feature flag compilation
- Provide auto-trait implementation, plus tests

### Perf

- Optimise init and fill perf by removing Option
- Add assert for optimising pack_into_u32
- UnsafeCell optimisations, less generic CellState
- Alignment & Vector optimisations

### Refactor

- Make generic for TurboCore PRNGs
- Remove Generics spam, AtomicRng
- Use Option for cleaner increment_counter
- Granular features for more refined conditional compilation
- Prelude module, code reorg & State made private
- Make trait object safe, split out GenCore trait
- Rename from SecureRng

## [0.5.1] - 2022-07-13

### Chore

- Use native endian byte methods for generate_entropy
- Remove unnecessary checks, tidy up docs
- Revise release version

### Doc

- Document added features
- Fix up references

### Feat

- Make generate_entropy more flexible with output
- Create compatibility layer with rand_core
- Implement serialize feature

## [0.5.0] - 2022-07-05

### Chore

- Prepare v0.5.0

### Feat

- Fill_bytes method

### Perf

- Inline more methods
- More inlining

### Refactor

- Use fill_bytes more and remove unneeded macro

## [0.4] - 2022-06-28

### Chore

- Expose some tests to WASM and make them more robust
- Enable entropy generation testing in smoke tests
- Prepare new release version

### Feat

- Implement char range method

### Reat

- Sample multiple method

### Refactor

- Better macros and 128 bit integer range methods

## [0.3.1] - 2022-06-14

### Fix

- Add necessary attributes to build docs

## [0.3.0] - 2022-06-14

### Chore

- Add floating point method benches
- Prepare new version

### Doc

- Add more examples

### Feat

- Benchmarking, optimisations and better panic messages
- Implement shuffle method
- Implement random character methods

### Fix

- Add rustdoc arguments

## [0.2.0] - 2022-06-12

### Chore

- Exclude github directories from package

### Docs

- Use new macro where appropriate

### Feat

- Debug impl and requirement for State trait
- Implement atomic state and feature flag
- Improve rng macro
- Atomic rng macro and more docs

### Fix

- Better handling of signed & unsigned ranges
- Use wrapping_sub for range

## [0.1.0] - 2022-06-10

### Chore

- Create README
- License the project
- Better Cargo.toml details
- Standardised spelling
- Add CI pipeline
- Remove redundant macro declaration
- Better error message on panic
- Improve README
- More badges

### Feat

- Let there be random noise

### Fix

- Point to correct main branch

