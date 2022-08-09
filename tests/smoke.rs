use std::{collections::BTreeMap, iter::repeat_with, ops::RangeBounds};
use turborand::prelude::*;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen_test::*;

#[cfg(target_arch = "wasm32")]
wasm_bindgen_test::wasm_bindgen_test_configure!(run_in_browser);

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
fn range_determinism_testing() {
    let rng = Rng::with_seed(Default::default());

    let value = rng.u64(1..10);

    assert_eq!(value, 8, "Not the same expected value: got {}", value);

    let value = rng.u32(1..10);

    assert_eq!(value, 2, "Not the same expected value: got {}", value);

    let value = rng.u16(1..10);

    assert_eq!(value, 9, "Not the same expected value: got {}", value);

    let value = rng.u8(1..10);

    assert_eq!(value, 5, "Not the same expected value: got {}", value);

    let value = rng.bool();

    assert!(value, "Not the expect boolean: got {}", value);
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
fn range_smoke_testing() {
    let rng = Rng::default();

    for _ in 0..1000 {
        let index = rng.usize(4..10);

        assert!(
            (4..10).contains(&index),
            "Must generate a number within 4 and 10, received: {}",
            index
        );
    }

    for _ in 0..1000 {
        let index = rng.usize(..20);

        assert!(
            (..20).contains(&index),
            "Must generate a number within 0 and 20, received: {}",
            index
        );
    }

    for _ in 0..1000 {
        let index = rng.usize(4..=15);

        assert!(
            (4..=15).contains(&index),
            "Must generate a number within 4 and inclusively 15, received: {}",
            index
        );
    }

    for _ in 0..1000 {
        let index = rng.isize(-10..10);

        assert!(
            (-10..10).contains(&index),
            "Must generate a number within -10 and 10, received: {}",
            index
        );
    }

    for _ in 0..1000 {
        let value = rng.u128(6..61);

        assert!(
            (6..61).contains(&value),
            "Must generate a number within 6 and 61, received: {}",
            value
        );
    }

    for _ in 0..1000 {
        let value = rng.i128(-20..20);

        assert!(
            (-20..20).contains(&value),
            "Must generate a number within -20 and 20, received: {}",
            value
        );
    }
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
fn small_range_smoke_testing() {
    let rng = Rng::with_seed(Default::default());

    let val1 = rng.u64(0..1);
    let val2 = rng.i64(0..1);
    let val3 = rng.u64(0..=1);
    let val4 = rng.i64(-1..=0);

    assert_eq!((val1, val2, val3, val4), (0, 0, 1, -1));
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
fn unbounded_range_smoke_testing() {
    let rng = Rng::default();

    for _ in 0..1000 {
        let index = rng.u8(..);

        assert!((..).contains(&index));
    }

    for _ in 0..1000 {
        let index = rng.u64(..);

        assert!((..).contains(&index));
    }

    for _ in 0..1000 {
        let index = rng.usize(..);

        assert!((..).contains(&index));
    }

    for _ in 0..1000 {
        let index = rng.i64(..);

        assert!((..).contains(&index));
    }

    for _ in 0..1000 {
        let index = rng.isize(..);

        assert!((..).contains(&index));
    }
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
fn unsigned_range_spread_test() {
    let rng = Rng::with_seed(Default::default());

    let actual_histogram: BTreeMap<u32, u32> =
        repeat_with(|| rng.u32(1..=10))
            .take(1000)
            .fold(BTreeMap::new(), |mut histogram, key| {
                *histogram.entry(key).or_default() += 1;

                histogram
            });

    let expected_histogram = BTreeMap::from_iter(vec![
        (1, 97),
        (2, 105),
        (3, 98),
        (4, 113),
        (5, 109),
        (6, 80),
        (7, 99),
        (8, 86),
        (9, 102),
        (10, 111),
    ]);

    assert_eq!(
        actual_histogram, expected_histogram,
        "unsigned samples should match in frequency to the expected histogram"
    );
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
fn signed_range_spread_test() {
    let rng = Rng::with_seed(Default::default());

    let actual_histogram: BTreeMap<i32, u32> =
        repeat_with(|| rng.i32(-5..=5))
            .take(1000)
            .fold(BTreeMap::new(), |mut histogram, key| {
                *histogram.entry(key).or_default() += 1;

                histogram
            });

    let expected_histogram = BTreeMap::from_iter(vec![
        (-5, 91),
        (-4, 89),
        (-3, 97),
        (-2, 90),
        (-1, 105),
        (0, 94),
        (1, 73),
        (2, 81),
        (3, 81),
        (4, 101),
        (5, 98),
    ]);

    assert_eq!(
        actual_histogram, expected_histogram,
        "signed samples should match in frequency to the expected histogram"
    );
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
fn character_smoke_testing() {
    let rng = Rng::default();

    for _ in 0..1000 {
        let character = rng.alphabetic();

        assert!(
            "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz".contains(character),
            "Must output an alphabetic character within range, received '{}'",
            character
        );
    }

    for _ in 0..1000 {
        let character = rng.alphanumeric();

        assert!(
            "0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz".contains(character),
            "Must output an alphanumeric character within range, received '{}'",
            character
        );
    }

    for _ in 0..1000 {
        let character = rng.lowercase();

        assert!(
            "abcdefghijklmnopqrstuvwxyz".contains(character),
            "Must output a lowercase character within range, received '{}'",
            character
        );
    }

    for _ in 0..1000 {
        let character = rng.uppercase();

        assert!(
            "ABCDEFGHIJKLMNOPQRSTUVWXYZ".contains(character),
            "Must output an uppercase character within range, received '{}'",
            character
        );
    }
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
fn digit_smoke_testing() {
    let rng = Rng::default();

    for _ in 0..1000 {
        let digit = rng.digit(10);

        assert!(
            "0123456789".contains(digit),
            "Must output a digit within radix, received '{}'",
            digit
        );
    }

    for _ in 0..1000 {
        let digit = rng.digit(2);

        assert!(
            "01".contains(digit),
            "Must output a digit within radix, received '{}'",
            digit
        );
    }

    for _ in 0..1000 {
        let digit = rng.digit(8);

        assert!(
            "01234567".contains(digit),
            "Must output a digit within radix, received '{}'",
            digit
        );
    }

    for _ in 0..1000 {
        let digit = rng.digit(16);

        assert!(
            "0123456789abcdef".contains(digit),
            "Must output a digit within radix, received '{}'",
            digit
        );
    }

    for _ in 0..1000 {
        let digit = rng.digit(36);

        assert!(
            "0123456789abcdefghijklmnopqrstuvwxyz".contains(digit),
            "Must output a digit within radix, received '{}'",
            digit
        );
    }
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
fn fill_bytes_smoke_testing() {
    let mut bytes = [0u8; 12];
    let rng = Rng::with_seed(Default::default());

    for _ in 0..1000 {
        rng.fill_bytes(&mut bytes);
    }

    assert_eq!(
        &bytes,
        &[8, 211, 110, 129, 82, 24, 169, 243, 239, 156, 64, 162],
        "array bytes should match expected output"
    );

    let mut bytes = vec![0_u8; 12];

    for _ in 0..1000 {
        rng.fill_bytes(bytes.as_mut_slice());
    }

    assert_eq!(
        &bytes,
        &[231, 75, 181, 137, 136, 142, 198, 200, 185, 42, 12, 175],
        "vec bytes should match expected output"
    );
}

#[cfg(feature = "chacha")]
#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
fn secure_rng_smoke_test() {
    let rand = SecureRng::with_seed([0u8; 40]);

    let value = rand.u64(5..=10);

    assert_eq!(&value, &10);

    for _ in 0..128 {
        let value = rand.u64(2..=20);

        assert!((2..=20).contains(&value));
    }

    let value = rand.i64(-5..=5);

    assert_eq!(&value, &-2);

    for _ in 0..128 {
        let value = rand.i64(-8..=8);

        assert!((-8..=8).contains(&value));
    }
}

#[cfg(feature = "chacha")]
#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
fn secure_rng_spread_test() {
    let rng = SecureRng::with_seed([0u8; 40]);

    let actual_histogram: BTreeMap<u32, u32> =
        repeat_with(|| rng.u32(1..=10))
            .take(1000)
            .fold(BTreeMap::new(), |mut histogram, key| {
                *histogram.entry(key).or_default() += 1;

                histogram
            });

    let expected_histogram = BTreeMap::from_iter(vec![
        (1, 101),
        (2, 86),
        (3, 99),
        (4, 103),
        (5, 88),
        (6, 91),
        (7, 109),
        (8, 103),
        (9, 111),
        (10, 109),
    ]);

    assert_eq!(
        actual_histogram, expected_histogram,
        "unsigned samples should match in frequency to the expected histogram"
    );

    let actual_histogram: BTreeMap<i32, i32> =
        repeat_with(|| rng.i32(-5..=5))
            .take(1000)
            .fold(BTreeMap::new(), |mut histogram, key| {
                *histogram.entry(key).or_default() += 1;

                histogram
            });

    let expected_histogram = BTreeMap::from_iter(vec![
        (-5, 99),
        (-4, 88),
        (-3, 93),
        (-2, 77),
        (-1, 93),
        (0, 104),
        (1, 77),
        (2, 73),
        (3, 111),
        (4, 97),
        (5, 88),
    ]);

    assert_eq!(
        actual_histogram, expected_histogram,
        "unsigned samples should match in frequency to the expected histogram"
    );
}

#[test]
#[cfg(target_pointer_width = "64")]
fn sample_spread_testing() {
    let rng = Rng::with_seed(Default::default());

    let indexes: [usize; 8] = [0, 1, 2, 3, 4, 5, 6, 7];
    let mut sampled = [0; 8];

    for _ in 0..2000 {
        let index = rng.sample(&indexes).unwrap();

        sampled[*index] += 1;
    }

    assert_eq!(
        &sampled,
        &[214, 238, 267, 241, 237, 276, 261, 266],
        "samples will occur across all array items at statistically equal chance"
    );
}

#[test]
#[cfg(target_pointer_width = "64")]
fn sample_multiple_spread_testing() {
    let rng = Rng::with_seed(Default::default());

    let indexes: [usize; 8] = [0, 1, 2, 3, 4, 5, 6, 7];
    let mut sampled = [0; 8];

    for _ in 0..1000 {
        let selected = rng.sample_multiple(&indexes, 3);

        selected
            .into_iter()
            .for_each(|sample| sampled[*sample] += 1);
    }

    assert_eq!(
        &sampled,
        &[399, 369, 391, 377, 373, 384, 345, 362],
        "samples will occur across all array items at statistically equal chance"
    );
}

#[test]
#[cfg(target_pointer_width = "64")]
fn weighted_sample_spread_testing() {
    let rng = Rng::with_seed(Default::default());

    let samples: [u32; 5] = [0, 1, 2, 3, 4];

    let sample_total_weight = f64::from(samples.iter().sum::<u32>());

    let actual_histogram: BTreeMap<u32, _> = repeat_with(|| {
        // Select items from the array based on their value divided by the total sum to
        // form their weighting.
        rng.weighted_sample(&samples, |&item| f64::from(item) / sample_total_weight)
    })
    .take(1000)
    .flatten()
    .fold(
        BTreeMap::from_iter(vec![(0, 0)]),
        |mut histogram, &individual| {
            *histogram.entry(individual).or_default() += 1;

            histogram
        },
    );

    // Larger values are expected to be selected more often. 0 should never be
    // selected ever.
    let expected_histogram =
        BTreeMap::from_iter(vec![(0, 0), (1, 92), (2, 207), (3, 294), (4, 407)]);

    assert_eq!(
        actual_histogram, expected_histogram,
        "weighted samples should match in frequency to the expected histogram"
    );
}

#[test]
#[cfg(target_pointer_width = "64")]
fn shuffle_smoke_testing() {
    let rng = Rng::with_seed(Default::default());

    let mut values = [1, 2, 3, 4, 5, 6];

    repeat_with(|| &rng)
        .take(100)
        .for_each(|r| r.shuffle(&mut values));

    assert_eq!(&values, &[2, 5, 3, 1, 6, 4]);
}
