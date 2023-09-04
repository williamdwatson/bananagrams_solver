# Bananagrams solver

## Usage
The simplest way to run is to download one of the prebuilt installers under the `Releases` section of GitHub. Alternatively, build from source:
### Setup
1. Install [Node.js](https://nodejs.org/en)
2. Install [Rust](https://www.rust-lang.org/)
3. In the repository directory, run `npm install`
### Running
Run `npm run tauri dev` to launch the program in development mode (with code watching/hot reloading); run `npm run tauri build` to compile the standalone program/installer in release mode.

## Code Layout
The `src` folder holds the frontend code, written in Typescript React. `App.tsx` is the parent of the frontend components (technically `main.tsx` is the parent, but it is essentially a wrapper); the components include `letter_input.tsx` for inputing which letters are in the hand and `results_display.tsx` which dislays the results as a table.

The `src-tauri` folder holds the backend code, including the `icons`, the actual source under `src`, and debug and release builds (including installers) under `target`. The entirety of the business code is located in `src/main.rs`. `dictionary.txt` is the entire Scrabble dictionary, capitalized and sorted from longest to shortest; `short_dictionary.txt` contains the most common words in English, filtered to only include those in the Scrabble dictionary - other words that would probably be frowned upon have been manually removed from this one as well.

### Sources
* `short_dictionary.txt` was derived from [MIT's 10000 word list](https://www.mit.edu/~ecprice/wordlist.10000), as well as [one derived from Google](https://github.com/first20hours/google-10000-english/blob/d0736d492489198e4f9d650c7ab4143bc14c1e9e/20k.txt).
* `dictionary.txt` was taken from [here](https://github.com/redbo/scrabble/blob/05748fb060b6e20480424b9113c1610066daca3c/dictionary.txt).

## How It Works
*TODO*
