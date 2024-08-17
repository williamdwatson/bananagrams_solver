# Bananagrams solver

This is a standalone [Bananagrams](https://bananagrams.com/) solving program (not affiliated in any way with the official Bananagrams), complete with GUI. It was built using TypeScript React for the frontend and [Tauri](https://tauri.app/) Rust for the backend. (Note that this is my first Rust project, so it may well be suboptimal or non-idiomatic)

The underlying algorithm has a few heuristics - it favors longer words and boards that alternate horizontally-vertically - but ultimately is exhaustive if the right settings are provided (if there is a solution, it will be found). Note that the solver is multithreaded and hence is not deterministic (i.e. multiple runs with the same hand can lead to different solutions).

![Screenshot of a solution of a board using all 144 standard Bananagrams tiles](example.png)
*Example solution of a board using all 144 standard Bananagrams tiles - smaller boards are much faster*

## Usage
The simplest way to run is to download one of the prebuilt installers under the `Releases` section of GitHub. Alternatively, build from source:
### Setup
1. Install [Node.js](https://nodejs.org/en)
2. Install [Rust](https://www.rust-lang.org/)
3. In the repository directory, run `npm install`
### Running
Run `npm run tauri dev` to launch the program in development mode (with code watching/hot reloading); run `npm run tauri build` to compile the standalone program/installer in release mode.
### Documentation
Code documentation can be found [here](https://williamdwatson.github.io/bananagrams_solver/doc/bananagrams_solver/index.html).
### Browser deployment
A purely in-browser version can be found [here](https://williamdwatson.github.io/bananagrams_solver_web/). Note that this is usually much slower than the Rust version.

## Performance
For performance metrics, a random selection of _n_ letters was taken from the set of standard Bananagrams letters (of which there are 144 in total, so when _n_=144 all letters are being used); five trials were run using full optimizations. Overall, as the hand size (number of letters) increases, the time taken to find a solution increases while the board density decreases. However, this can vary dramatically depending on which letters were randomly chosen. Note that in one trial with 72 letters, no solution was found (this result was discarded and a new run was performed). The actual numbers are available in `stats.xlsx`, and `generate_graphs.ipynb` holds the analysis code.

| Number of letters  | Average time to solution (ms) | Average board density (letters/square) |
| ------------------ | ----------------------------- | -------------------------------------- |
|         10         |           2.0 ± 0.0           |              0.346 ± 0.05              |
|         15         |          65.6 ± 128.6         |              0.320 ± 0.02              |
|         21         |          11.0 ± 13.1          |              0.298 ± 0.02              |
|         35         |          28.4 ± 6.58          |              0.190 ± 0.04              |
|         60         |        1086.6 ± 1249.3        |              0.176 ± 0.02              |
|         72         |         207.6 ± 52.4          |              0.194 ± 0.05              |
|         100        |         290.2 ± 67.5          |              0.160 ± 0.03              |
|         144        |         763.8 ± 23.3          |              0.150 ± 0                 |

<img src="time_to_solution.png" alt="Time to solution in milliseconds versus hand size" width="500">
<img src="average_time_to_solution.png" alt="Average time to solution in milliseconds versus hand size" width="500">

*Time taken to find a solution in milliseconds versus number of letters in the hand*

<img src="board_density.png" alt="Board density in letters per square versus hand size" width="500">
<img src="average_board_density.png" alt="Average board density in letters per square versus hand size" width="500">

*Density of tiles played in letters per square (i.e. number of letters in hand/dimensions of board)*

All statistics were collected on my machine (Windows 10, 8GB RAM, Intel i7-6700HQ CPU).

## Code Layout
The `src` folder holds the frontend code, written in Typescript/React. `App.tsx` is the parent of the frontend components (technically `main.tsx` is the parent, but it is essentially a wrapper); the components include `letter_input.tsx` for inputing which letters are in the hand and `results_display.tsx` which dislays the results as a table.

The `src-tauri` folder holds the backend code, including the `icons`, the actual source under `src`, and debug and release builds (including installers) under `target`. The entirety of the business code is located in `src/main.rs`. `dictionary.txt` and `updated_short_dictionary.txt` contain the word lists used by the program.

### Sources
All source dictionary text files are stored in `dictionaries.tar.gz`; as described above, the short and full dictionary files are present under `src-tauri/src` as `dictionary.txt` and `updated_short_dictionary.txt`, respectively.
#### Full dictionary
The full Scrabble dictionary was taken from https://github.com/redbo/scrabble/blob/05748fb060b6e20480424b9113c1610066daca3c/dictionary.txt, with minimal manual editing performed to remove a few slurs (some may have been missed, so if notificed please open an issue).
#### Short dictionary
Several English dictionaries of "common" words were combined to generate a dictionary of acceptable words. These include:
* Lists from https://people.sc.fsu.edu/~jburkardt/datasets/words/words.html under the [LGPL license](https://www.gnu.org/licenses/lgpl-3.0.en.html#license-text):
    * basic_english_850.txt
    * basic_english_2000.txt
    * doublet_words.txt
    * globish.txt
    * simplified_english.txt
    * special_english.txt
    * unique_grams.txt
* Lists from https://github.com/MichaelWehar/Public-Domain-Word-Lists in the public domain:
    * 200-less-common.txt
    * 5000-more-common.txt
* MIT's 10000 word list (https://www.mit.edu/~ecprice/wordlist.10000)
    * wordlist.10000.txt
* 10000 word list derived from Google (https://github.com/first20hours/google-10000-english)
    * google-10000-english.txt

These dictionaries were combined to form `new_short_dictionary.txt` (the code to do so is [in another repo](https://github.com/williamdwatson/bananagrams_gan/blob/main/combine_dictionaries.ipynb)). A set of country and demonym information was downloaded from https://gist.github.com/consti/e2c7ddc64f0aa044a8b4fcd28dba0700, and these words were removed. [lemminflect](https://github.com/bjascob/LemmInflect) was then to generate missing inflections for the words, before lastly manual editing was performed to remove or add a few words; the final dictionary was saved as `updated_short_dictionary.txt`.

## How It Works
The hand of letters is represented by a `[usize; 26]` (the number of each letter present in the hand), while the board is represented as a 144x144 grid of `usize`s (flattend to a single `Vec`); words are represented as `Vec<usize>`s as well (with each number being 0-25, or the "empty value" of 30). When play begins, a vector of all possible words playable with the current hand of letters is generated.

During play, we keep track of the boundary of the board (i.e. the square in which all played letters fall). Play then occurs recursively. The play function loops through trying every possible word at every position within the board boundary (and outside ± the length of that word). Every time a word is tried in a location, the validity of that placement is checked (i.e. are all letters connected, are all words valid, and have we not used too many of any letter). If valid, the function is recursively called; if not, the loop continues (and if the loop finishes, we backtrack up one recursive level). Words are tried both horizontally and vertically, althought the order in which they are tried alternates as a heuristic (since it's easier to play a vertical word onto a horizontal word than play two horizontal words side-by-side). At every level, the playable words are filtered down to those that can be played using the letters still in the hand plus a certain number from the board - this setting can be changed, but using a relatively low value (default of 2) can dramatically increase solving speed. With the proper settings, the check is ultimately exhaustive and so can take some time to determine that there are no valid solutions; however, the maximum number of iterations can also be set to alleviate this issue. The first recursive level of the word-checking loop is multithreaded, with each thread responsible for a chunk of all possible words - an `AtomicBool` tracks whether any thread has found a solution for early stopping.

For solving off a previous board, first a heuristic is used when only a single additional letter is added to the previously recorded hand: that letter is tried at every spot on the board. If multiple letters are provided, then words playable with those letters plus a certain number already on the board are checked recursively. After that, single words are removed in a recursive fashion and recursive play is attempted from those. If all that fails, the recursive processing begins anew.

The frontend uses the [PrimeReact](https://primereact.org/) component library, along with [react-zoom-pan-pinch](https://github.com/BetterTyped/react-zoom-pan-pinch).
