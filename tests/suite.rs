use std::{iter::repeat_with, ops::RangeBounds};

use std::collections::BTreeMap;

use turborand::prelude::*;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen_test::*;

#[cfg(target_arch = "wasm32")]
wasm_bindgen_test::wasm_bindgen_test_configure!(run_in_browser);

#[cfg(all(feature = "wyrand", feature = "std"))]
mod char_battery;
#[cfg(any(feature = "wyrand", feature = "chacha"))]
mod smoke;
