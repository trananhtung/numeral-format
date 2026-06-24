# Changelog

All notable changes to this project are documented here. The format is based on
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and this project adheres to
[Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.0]

### Added

- Initial release: `format(value, format)` implementing the numeral.js format-string DSL
  (default `en` locale) — separators, decimals, optional decimals, abbreviations, currency,
  percentages, bytes, ordinals, exponential, time, basis points, forced signs, and
  parenthesized negatives. A faithful, zero-dependency port of the `numeral` npm package's
  formatting.
