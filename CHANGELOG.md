# Changelog
All notable changes to this project will be documented in this file.


## v1.5 - 2022-01-14

### Added
- Word list updates

### Changed
- Changed the keyboard layout to have backspace and submit buttons away from each other - thanks for the feedback!
- Streaks are no longer reset when changing between game modes and settings. The plan is to persist the streaks for each game mode separately, but this should be better for now.

### Fixed


## v1.4 - 2022-01-14

### Added

### Changed

### Fixed
- Fixed a bug where if a user using the common list visited the daily word, the word list would be reset to full list, but still be shown as common words at the menu. Now the selected word list actually persists. Also changed how the word list is stored in state.


## v1.3 - 2022-01-13

### Added

### Changed
- Change the common words list ("Suppea") to be the default one. The feedback I'm receiving seems to suggest that most (vocal) people are expecting less special words, but the original list is still available as "Laaja".
- Refactor how full words list is stored in memory

### Fixed
- Don't reinitialize game twice when word list is changed


## v1.2 - 2022-01-12

### Added
- Added a setting which changes the pool of words to guess from to a smaller hand picked list. This word list should contain less strange unused / dialect words. Guesses can still be submitted from the full list. For 6 character mode the common words list is ~~still mostly unchanged~~ complete.
- The default word list is still the full list, but this setting is persisted

### Changed

### Fixed


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
