# TODO

- [ ] Improve assertions quality, align message output.
- [ ] Add assertions for additional Rust ecosystem types.
- [ ] Make runtime assertions (non panicking) more efficient (fewer allocation when possible).
- [ ] Is our current architecture regarding overlapping types (Vec, slice, String, str) sound.
- [ ] Is our approach for data-extracting-assertions sound? ok assertion on result automatically mapping to Ok value on
  success to allow other fluent assertions.
- [ ] Should / can we establish a better strategy for dealing with inverted ("not" X) assertions?
