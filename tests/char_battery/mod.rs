use crate::*;

use std::{collections::HashSet, ops::RangeBounds};

fn char_coverage_stress_test<R>(n: usize, range: R)
where
    R: Iterator<Item = char> + RangeBounds<char> + Clone,
{
    let all: HashSet<char> = range.clone().collect();
    let mut covered = HashSet::new();

    let rng = Rng::default();

    for _ in 0..n {
        let c = rng.char(range.clone());
        assert!(all.contains(&c), "Invalid character, received {}", &c);
        covered.insert(c);
    }

    assert_eq!(covered, all, "Missing coverage in output");
}

#[cfg(feature = "std")]
#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
fn char_battery_tests() {
    // ASCII control chars.
    let nul = 0u8 as char;
    let soh = 1u8 as char;
    let stx = 2u8 as char;
    // Some undefined Hangul Jamo codepoints just before
    // the surrogate area.
    let last_jamo = char::from_u32(0xd7ffu32).unwrap();
    let penultimate_jamo = char::from_u32(last_jamo as u32 - 1).unwrap();
    // Private-use codepoints just after the surrogate area.
    let first_private = char::from_u32(0xe000u32).unwrap();
    let second_private = char::from_u32(first_private as u32 + 1).unwrap();
    // Private-use codepoints at the end of Unicode space.
    let last_private = char::MAX;
    let penultimate_private = char::from_u32(last_private as u32 - 1).unwrap();

    char_coverage_stress_test(100, nul..stx);
    char_coverage_stress_test(100, nul..=soh);

    char_coverage_stress_test(400, penultimate_jamo..second_private);
    char_coverage_stress_test(400, penultimate_jamo..=second_private);

    char_coverage_stress_test(100, penultimate_private..=last_private);

    char_coverage_stress_test(1200, 'a'..='Ç');
}
