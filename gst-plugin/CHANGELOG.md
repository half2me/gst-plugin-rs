# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](http://keepachangelog.com/en/1.0.0/)
and this project adheres to [Semantic Versioning](http://semver.org/spec/v2.0.0.html),
specifically the [variant used by Rust](http://doc.crates.io/manifest.html#the-version-field).

## [0.1.2] - 2018-01-03
### Fixed
- BaseTransform::transform_caps() caps parameter is not owned when chainging
  to the parent class' implementation either

## [0.1.1] - 2018-01-03
### Fixed
- BaseTransform::transform_caps() caps parameter is not owned

## [0.1.0] - 2017-12-22
- Initial release of the `gst-plugin` crate.

[Unreleased]: https://github.com/sdroege/gstreamer-rs/compare/0.1.1...HEAD
[0.1.0]: https://github.com/sdroege/gstreamer-rs/compare/0.1.0...0.1.1
