# Changelog
All notable changes to this project will be documented in this file.

## v1.13 - 2022-02-13
### Added
- New game mode: Neluli! Inspired by https://www.quordle.com/, solve four words simultaneously

### Fixed
- Fix max streaks not being persisted if the user never changed their settings before refreshin the page. This had been causing the bug that some users were reporting of their max streak being repoted wrong on the modal - unfortunately some data streaks may already be lost.

### Changed
- Different game modes with special rules can now be supported, as long as they implement the `Game` trait.


## v1.12 - 2022-02-03
### Added
- Game can now be shared as a link, and the party receiving the link can either just view the solution or try it by themselves.

### Fixed
- Color descriptions on the help page were wrong when the colorblind theme was selected.

### Changed
- Some refactoring, state.rs had grown too big and is now split to game and manager.


## v1.11 - 2022-01-29

### Fixed
- Fixed a bug causing some players to get stuck in daily word game and being unable to change the gamemode or return to their previous game with "Takaisin" button.

## v1.10 - 2022-01-25

### Added
- Sharing the daily sanuli solutions as emojis

## v1.9 - 2022-01-24

### Fixed
- Fixed issues related to the daily word when date changes
- When refreshing the page new daily word is shown if the daily word game was open and date has changed


## v1.8 - 2022-01-22

### Added
- User can now switch between different game modes and settings without losing the old game state or streaks
- Lots of internal refactoring to how game state is persisted

### Changed
- Profanities are now disallowed by default
- Streaks are now per game mode, and you can no longer cheat and restart the word by changing settings
    - This unfortunately also means that some users will lose their current streak if daily game mode was the last selected mode. Sorry.

### Fixed
- Animation when sixth word is correct and new game is started


## v1.7 - 2022-01-16

### Added
- Add theme for colorblind mode, available at settings


## v1.6 - 2022-01-15

### Added
- Profanities filter, which removes some words from the pool of words to guess from.
- Some statistics of past games added to menu

## v1.5 - 2022-01-14

### Added
- Word list updates

### Changed
- Changed the keyboard layout to have backspace and submit buttons away from each other - thanks for the feedback!
- Streaks are no longer reset when changing between game modes and settings. The plan is to persist the streaks for each game mode separately, but this should be better for now.


## v1.4 - 2022-01-14

### Fixed
- Fixed a bug where if a user using the common list visited the daily word, the word list would be reset to full list, but still be shown as common words at the menu. Now the selected word list actually persists. Also changed how the word list is stored in state.


## v1.3 - 2022-01-13

### Changed
- Change the common words list ("Suppea") to be the default one. The feedback I'm receiving seems to suggest that most (vocal) people are expecting less special words, but the original list is still available as "Laaja".
- Refactor how full words list is stored in memory

### Fixed
- Don't reinitialize game twice when word list is changed


## v1.2 - 2022-01-12

### Added
- Added a setting which changes the pool of words to guess from to a smaller hand picked list. This word list should contain less strange unused / dialect words. Guesses can still be submitted from the full list. For 6 character mode the common words list is ~~still mostly unchanged~~ complete.
- The default word list is still the full list, but this setting is persisted

## v1.1 - 2022-01-11

### Fixed
- Fix a bug where on current tile state border highlights the count of known characters wasn't considered. Even if the exact count of characters was already known the absent tiles were still displayed with yellow "present" border instead of as known absent. Thanks /u/Allium_Senescens for the bug report!


## v1.0 - 2022-01-09

### Added
- Added version number and changelog

### Changed
- Refactor project structure into components

### Fixed
- Fix board sometimes not rerendering after keypresses
