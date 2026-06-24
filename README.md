# numeral-format

[![All Contributors](https://img.shields.io/badge/all_contributors-1-orange.svg?style=flat-square)](#contributors-)

[![crates.io](https://img.shields.io/crates/v/numeral-format.svg)](https://crates.io/crates/numeral-format)
[![docs.rs](https://docs.rs/numeral-format/badge.svg)](https://docs.rs/numeral-format)
[![CI](https://github.com/trananhtung/numeral-format/actions/workflows/ci.yml/badge.svg)](https://github.com/trananhtung/numeral-format/actions/workflows/ci.yml)
[![license](https://img.shields.io/crates/l/numeral-format.svg)](#license)

**Format numbers with the [numeral.js](http://numeraljs.com/) format-string DSL.**

A familiar mini-language for turning numbers into display strings — thousands separators,
fixed and optional decimals, abbreviations, currency, percentages, bytes, ordinals, forced
signs, and parenthesized negatives:

```rust
use numeral_format::format;

assert_eq!(format(1000.0, "0,0"), "1,000");
assert_eq!(format(1234.567, "0,0.00"), "1,234.57");
assert_eq!(format(1_234_567.0, "0.0a"), "1.2m");
assert_eq!(format(0.45, "0.0%"), "45.0%");
assert_eq!(format(1234.56, "$0,0.00"), "$1,234.56");
assert_eq!(format(1048576.0, "0.0ib"), "1.0MiB");
assert_eq!(format(3.0, "0o"), "3rd");
assert_eq!(format(-1000.0, "($0,0)"), "($1,000)");
```

A faithful Rust port of the formatting half of the widely-used
[`numeral`](https://www.npmjs.com/package/numeral) npm package (default `en` locale), which
has no Rust equivalent.

- **Zero dependencies**
- Differential-tested against the reference `numeral` implementation across the full DSL

## Install

```toml
[dependencies]
numeral-format = "0.1"
```

## The format DSL

| Format       | `format(n, …)`                         | Result        |
| ------------ | -------------------------------------- | ------------- |
| `0,0`        | `1000`                                 | `1,000`       |
| `0,0.00`     | `1000.234`                             | `1,000.23`    |
| `0[.]0`      | `3` / `3.5`                            | `3` / `3.5`   |
| `0.0a`       | `1234567`                              | `1.2m`        |
| `$0,0.00`    | `1000.234`                             | `$1,000.23`   |
| `0.000%`     | `0.974878`                             | `97.488%`     |
| `0b`         | `1000`                                 | `1KB`         |
| `0.0ib`      | `1048576`                              | `1.0MiB`      |
| `0o`         | `2`                                    | `2nd`         |
| `+0,0`       | `1.5`                                  | `+2`          |
| `(0,0)`      | `-1000`                                | `(1,000)`     |
| `00:00:00`   | `3661`                                 | `1:01:01`     |

The default format (used for an empty format string) is `0,0`.

## Scope

This crate ports number → string **formatting** with the default `en` locale. It does not
implement parsing formatted strings back to numbers (`unformat`), arithmetic helpers
(`add`/`subtract`/…), or alternate locales. `NaN` is treated as numeral's `null` and
formats as zero.

## Precision

Like numeral.js, formatting goes through `f64`. For values whose formatted result exceeds
~15–17 significant digits (for example a percentage of a number larger than ~10¹⁰, or any
result past ~10¹²), the last digit may differ by one unit — this is the inherent limit of
double-precision floating point, and is exactly where numeral.js itself becomes unreliable.
For typical display magnitudes the output matches numeral.js byte-for-byte.

## Contributors ✨

This project follows the [all-contributors](https://github.com/all-contributors/all-contributors) specification. Contributions of any kind are welcome — code, docs, bug reports, ideas, reviews! See the [emoji key](https://allcontributors.org/docs/en/emoji-key) for how each contribution is recognized, and open a PR or issue to get involved.

Thanks goes to these wonderful people:

<!-- ALL-CONTRIBUTORS-LIST:START - Do not remove or modify this section -->
<!-- prettier-ignore-start -->
<!-- markdownlint-disable -->
<table>
  <tbody>
    <tr>
      <td align="center" valign="top" width="14.28%"><a href="https://github.com/trananhtung"><img src="https://avatars.githubusercontent.com/u/30992229?v=4?s=100" width="100px;" alt="Tung Tran"/><br /><sub><b>Tung Tran</b></sub></a><br /><a href="https://github.com/trananhtung/./commits?author=trananhtung" title="Code">💻</a> <a href="#maintenance-trananhtung" title="Maintenance">🚧</a></td>
    </tr>
  </tbody>
</table>

<!-- markdownlint-restore -->
<!-- prettier-ignore-end -->

<!-- ALL-CONTRIBUTORS-LIST:END -->

## License

Licensed under either of [MIT](LICENSE-MIT) or [Apache-2.0](LICENSE-APACHE) at your option.
