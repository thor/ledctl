# Changelog

## [0.1.1](https://github.com/thor/ledctl/compare/v0.1.0...v0.1.1) (2026-05-13)


### Bug Fixes

* **ci:** get release-please working ([22e61c0](https://github.com/thor/ledctl/commit/22e61c0c4b23fa1b2da15a58049b154a6e583032))

## 0.1.0 (2026-05-13)


### Features

* add CLI argument parsing (parse_args) ([2b38c73](https://github.com/thor/ledctl/commit/2b38c73d9a66dbd93acbb7c40114b9b946f7b521))
* add HidSession and keyboard enumeration ([41f4875](https://github.com/thor/ledctl/commit/41f4875769298bab1b098458c10a78edb75d36b0))
* add IOKit FFI declarations and CF helper utilities ([f00554d](https://github.com/thor/ledctl/commit/f00554df452953a184f00cb941d58675b1b7fe90))
* add Led enum, LedState, LedError types ([be7b988](https://github.com/thor/ledctl/commit/be7b988adebaf12298369bdfceb5b97da686b858))
* add LED read operations (get_led_state) ([ff33227](https://github.com/thor/ledctl/commit/ff33227c1b5ee1b16cb3f2ae24235c374da2a8c6))
* add LED write and toggle operations ([5d6f925](https://github.com/thor/ledctl/commit/5d6f925a23be2ff1db2a81d650c2abc2d41f9cc7))
* implement CLI execution (list, set, toggle) ([e4a4263](https://github.com/thor/ledctl/commit/e4a42636c19d3c350a36f9e4d2f155f595f698d8))
* implement egui GUI with device picker and LED toggle buttons ([fa843b9](https://github.com/thor/ledctl/commit/fa843b9552c20f07b5327cd23be1fb97c92eec34))
* replace egui/eframe with iced with elm architecture ([cc9caf0](https://github.com/thor/ledctl/commit/cc9caf09fd18951788eb0f638190b9ed270b3e34))


### Bug Fixes

* --device filters --list output; --count rejected for on/off actions ([1bc9a07](https://github.com/thor/ledctl/commit/1bc9a0751b6c084dbb807df0bf183153bf259d95))
* resize down to the contents of the main window ([0b9f2bb](https://github.com/thor/ledctl/commit/0b9f2bbaba52c54706bb9ed3ecbbf24878e5d884))
* use the 2024 edition of rust ([58077ee](https://github.com/thor/ledctl/commit/58077eef9432f5d330135119dd5c0a02d39baadf))


### Performance Improvements

* perform harsher release-type optimizations ([8f5ed23](https://github.com/thor/ledctl/commit/8f5ed23d9e69216985a8a17d2a2760e74e81532b))
