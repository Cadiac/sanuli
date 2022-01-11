# Changelog
All notable changes to this project will be documented in this file.

## v1.1 - 2022-01-11

### Added

### Changed

### Fixed
- Fix a bug where on current tile state border highlights the count of known characters wasn't considered. Even if the exact count of characters was already known the absent tiles were still displayed with yellow "present" border instead of as known absent. Thanks /u/Allium_Senescens for the bug report!

## v1.0 - 2022-01-09

### Added
- Added version number and changelog

### Changed
- Refactor project structure into components

### Fixed
- Fix board sometimes not rerendering after keypresses
