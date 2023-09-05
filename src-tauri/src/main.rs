// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::{cmp, fmt, mem, thread, usize, collections::HashMap, collections::HashSet};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Instant;
use rand::prelude::*;
use rand::distributions::Uniform;
use serde::Serialize;
use tauri::State;

/// A numeric representation of a word
type Word = Vec<usize>;
/// Represents a hand of letters
type Letters = [usize; 26];
/// Represents the previous sequence of plays
type PlaySequence = Vec<(Word, (usize, usize, Direction))>;

/// The maximum length of any word in the dictionary
const MAX_WORD_LENGTH: usize = 15;
/// Value of an empty cell on the board
const EMPTY_VALUE: usize = 30;
/// Number rows/columns in the board
const BOARD_SIZE: usize = 144;
/// All uppercase letters in the Latin alphabet
const UPPERCASE: &str = "ABCDEFGHIJKLMNOPQRSTUVWXYZ";
/// The number of each letter present in regular Bananagrams
const REGULAR_TILES: [u64; 26] = [13, 3, 3, 6, 18, 3, 4, 3, 12, 2, 2, 5, 3, 8, 11, 3, 2, 9, 6, 9, 6, 3, 3, 2, 3, 2];

/// A thin wrapper for handling the board
#[derive(Clone)]
struct Board {
    /// The underlying array of the board
    arr: [usize; BOARD_SIZE*BOARD_SIZE]
}
impl Board {
    fn new() -> Board {
        return Board { arr: [EMPTY_VALUE; BOARD_SIZE*BOARD_SIZE] }
    }
    /// Unsafely gets a value from the board at the given index
    /// # Arguments
    /// * `row` - Row index of the value to get (must be less than `BOARD_SIZE`)
    /// * `col` - Column index of the value to get (must be less than `BOARD_SIZE`)
    /// # Returns
    /// `usize` - The value in the board at `(row, col)` (if either `row` or `col` are greater than `BOARD_SIZE` this will be undefined behavior)
    fn get_val(&self, row: usize, col: usize) -> usize {
        return unsafe { *self.arr.get_unchecked(row*BOARD_SIZE + col) };
    }
    /// Unsafely sets a value in the board at the given index
    /// # Arguments
    /// * `row` - Row index of the value to get (must be less than `BOARD_SIZE`)
    /// * `col` - Column index of the value to get (must be less than `BOARD_SIZE`)
    /// * `val` - Value to set at `(row, col)` in the board (if either `row` or `col` are greater than `BOARD_SIZE` this will be undefined behavior)
    fn set_val(&mut self, row: usize, col: usize, val: usize) {
        let v = unsafe { self.arr.get_unchecked_mut(row*BOARD_SIZE + col) };
        *v = val;
    }
}

/// Converts a word into a numeric vector representation
/// # Arguments
/// * `word` - String word to convert
/// # Returns
/// `Word` - numeric representation of `word`, with each letter converted from 0 ('A') to 25 ('Z')
/// # See also
/// `convert_array_to_word`
fn convert_word_to_array(word: &str) -> Word {
    word.chars().filter(|c| c.is_ascii_uppercase()).map(|c| (c as usize) - 65).collect()
}

/// Converts a numeric vector representation into a `String`
/// # Arguments
/// * `arr` - Numeric vector of the word
/// # Returns
/// * `String` - `arr` converted into a `String`, with each number converted from 'A' (0) to 'Z' (25)
/// # See also
/// `convert_word_to_array`
fn convert_array_to_word(arr: &Word) -> String {
    arr.iter().map(|c| (*c as u8+65) as char).collect()
}

/// Converts a `board` to a `String`
/// # Arguments
/// * `board` - Board to display
/// * `min_col` - Minimum occupied column index
/// * `max_col` - Maximum occupied column index
/// * `min_row` - Minimum occupied row index
/// * `max_row` - Maximum occupied row index
/// # Returns
/// * `String` - `board` in string form (with all numbers converted to letters)
// fn board_to_string(board: &Board, min_col: usize, max_col: usize, min_row: usize, max_row: usize) -> String {
//     let mut board_string: Vec<char> = Vec::with_capacity((max_row-min_row)*(max_col-min_col));
//     for row in min_row..max_row+1 {
//         for col in min_col..max_col+1 {
//             if board.get_val(row, col) == EMPTY_VALUE {
//                 board_string.push(' ');
//             }
//             else {
//                 board_string.push((board.get_val(row, col) as u8+65) as char);
//             }
//         }
//         board_string.push('\n');
//     }
//     let s: String = board_string.iter().collect();
//     return s.trim_end().to_owned();
// }

/// Converts a `board` to a vector of vectors of chars
/// # Arguments
/// * `board` - Board to display
/// * `min_col` - Minimum occupied column index
/// * `max_col` - Maximum occupied column index
/// * `min_row` - Minimum occupied row index
/// * `max_row` - Maximum occupied row index
/// # Returns
/// * `Vec<Vec<char>>` - `board` in vector form (with all numbers converted to letters)
fn board_to_vec(board: &Board, min_col: usize, max_col: usize, min_row: usize, max_row: usize) -> Vec<Vec<char>> {
    let mut board_vec: Vec<Vec<char>> = Vec::with_capacity(max_row-min_row);
    for row in min_row..max_row+1 {
        let mut row_vec: Vec<char> = Vec::with_capacity(max_col-min_col);
        for col in min_col..max_col+1 {
            if board.get_val(row, col) == EMPTY_VALUE {
                row_vec.push(' ');
            }
            else {
                row_vec.push((board.get_val(row, col) as u8+65) as char);
            }
        }
        board_vec.push(row_vec);
    }
    return board_vec;
}

/// Checks whether a `word` can be made using the given `letters`
/// # Arguments
/// * `word` - The vector form of the word to check
/// * `letters` - Length-26 array of the number of each letter in the hand
/// # Returns
/// * `bool` - Whether `word` can be made using `letters`
fn is_makeable(word: &Word, letters: Letters) -> bool {
    let mut available_letters = letters.clone();
    for letter in word.iter() {
        if unsafe { available_letters.get_unchecked(*letter) } == &0 {
            return false;
        }
        let elem = unsafe { available_letters.get_unchecked_mut(*letter) };
        *elem -= 1;
    }
    return true;
}

/// Checks that a `board` is valid after a word is played horizontally, given the specified list of `valid_word`s
/// Note that this does not check if all words are contiguous; this condition must be enforced elsewhere.
/// # Arguments
/// * `board` - `Board` being checked
/// * `min_col` - Minimum x (column) index of the subsection of the `board` to be checked
/// * `max_col` - Maximum x (column) index of the subsection of the `board` to be checked
/// * `min_row` - Minimum y (row) index of the subsection of the `board` to be checked
/// * `max_row` - Maximum y (row) index of the subsection of the `board` to be checked
/// * `row` - Row of the word played
/// * `start_col` - Starting column of the word played
/// * `end_col` - Ending column of the word played
/// * `valid_words` - HashSet of all valid words as `Vec<usize>`s
/// # Returns
/// `bool` - whether the given `board` is made only of valid words
fn is_board_valid_horizontal(board: &Board, min_col: usize, max_col: usize, min_row: usize, max_row: usize, row: usize, start_col: usize, end_col: usize, valid_words: &HashSet<Word>) -> bool {
    let mut current_letters: Vec<usize> = Vec::with_capacity(MAX_WORD_LENGTH);
    // Check across the row where the word was played
    for col_idx in min_col..max_col+1 {
        // If we're not at an empty square, add it to the current word we're looking at
        if board.get_val(row, col_idx) != EMPTY_VALUE {
            current_letters.push(board.get_val(row, col_idx));
        }
        else {
            if current_letters.len() > 1 && !valid_words.contains(&current_letters) {
                return false;
            }
            current_letters.clear();
            if col_idx > end_col {
                break;
            }
        }
    }
    if current_letters.len() > 1 && !valid_words.contains(&current_letters) {
        return false;
    }
    // Check down each column where a letter was played
    for col_idx in start_col..end_col+1 {
        current_letters.clear();
        for row_idx in min_row..max_row+1 {
            if board.get_val(row_idx, col_idx) != EMPTY_VALUE {
                current_letters.push(board.get_val(row_idx, col_idx));
            }
            else {
                if current_letters.len() > 1 && !valid_words.contains(&current_letters) {
                    return false;
                }
                current_letters.clear();
                if row_idx > row {
                    break;
                }
            }
        }
        if current_letters.len() > 1 && !valid_words.contains(&current_letters) {
            return false;
        }
    }
    return true;
}

/// Checks that a `board` is valid after a word is played vertically, given the specified list of `valid_word`s
/// Note that this does not check if all words are contiguous; this condition must be enforced elsewhere.
/// # Arguments
/// * `board` - `Board` being checked
/// * `min_col` - Minimum x (column) index of the subsection of the `board` to be checked
/// * `max_col` - Maximum x (column) index of the subsection of the `board` to be checked
/// * `min_row` - Minimum y (row) index of the subsection of the `board` to be checked
/// * `max_row` - Maximum y (row) index of the subsection of the `board` to be checked
/// * `start_row` - Starting row of the word played
/// * `end_row` - Ending row of the word played
/// * `col` - Column of the word played
/// * `valid_words` - HashSet of all valid words as `Vec<usize>`s
/// # Returns
/// `bool` - whether the given `board` is made only of valid words
fn is_board_valid_vertical(board: &Board, min_col: usize, max_col: usize, min_row: usize, max_row: usize, start_row: usize, end_row: usize, col: usize, valid_words: &HashSet<Word>) -> bool {
    let mut current_letters: Vec<usize> = Vec::with_capacity(MAX_WORD_LENGTH);
    // Check down the column where the word was played
    for row_idx in min_row..max_row+1 {
        // If it's not an empty value, add it to the current word
        if board.get_val(row_idx, col) != EMPTY_VALUE {
            current_letters.push(board.get_val(row_idx, col));
        }
        else {
            // Otherwise, check if we have more than one letter - if so, check if the word is valid
            if current_letters.len() > 1 && !valid_words.contains(&current_letters) {
                return false;
            }
            current_letters.clear();
            // If we're past the end of the played word, no need to check farther
            if row_idx > end_row {
                break;
            }
        }
    }
    // In case we don't hit the `else` in the previous loop
    if current_letters.len() > 1 {
        if !valid_words.contains(&current_letters) {
            return false;
        }
    }
    // Check across each row where a letter was played
    for row_idx in start_row..end_row+1 {
        current_letters.clear();
        for col_idx in min_col..max_col+1 {
            if board.get_val(row_idx, col_idx) != EMPTY_VALUE {
                current_letters.push(board.get_val(row_idx, col_idx));
            }
            else {
                if current_letters.len() > 1 && !valid_words.contains(&current_letters) {
                    return false;
                }
                current_letters.clear();
                if col_idx > col {
                    break;
                }
            }
        }
        if current_letters.len() > 1 && !valid_words.contains(&current_letters) {
            return false;
        }
    }
    return true;
}

/// Enumeration of how many letters have been used
#[derive(Copy, Clone)]
enum LetterUsage {
    /// There are still unused letters
    Remaining,
    /// More letters have been used than are available
    Overused,
    /// All letters have been used
    Finished
}
impl fmt::Display for LetterUsage {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
       match self {
            LetterUsage::Remaining => write!(f, "Remaining"),
            LetterUsage::Overused => write!(f, "Overused"),
            LetterUsage::Finished => write!(f, "Finished")
       }
    }
}
impl fmt::Debug for LetterUsage {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
             LetterUsage::Remaining => write!(f, "Remaining"),
             LetterUsage::Overused => write!(f, "Overused"),
             LetterUsage::Finished => write!(f, "Finished")
        }
     }
}

/// Enumeration of the direction a word is played
#[derive(Copy, Clone)]
enum Direction {
    /// The word was played horizontally
    Horizontal,
    /// The word was played vertically
    Vertical
}
impl fmt::Display for Direction {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
       match self {
            Direction::Vertical => write!(f, "Horizontal"),
            Direction::Horizontal => write!(f, "Vertical")
       }
    }
}

/// Plays a word on the board
/// # Arguments
/// * `word` - The word to be played
/// * `row_idx` - The starting row at which to play the word
/// * `col_idx` - The starting column at which to play the word
/// * `board` - The current board (is modified in-place)
/// * `direction` - The `Direction` in which to play the word
/// # Returns
/// *`Result` with:*
/// * `bool` - Whether the word could be validly played
/// * `Vec<(usize, usize)>` - Vector of the indices played in `board`
/// 
/// *or empty `Err` if out-of-bounds*
fn play_word(word: &Word, row_idx: usize, col_idx: usize, board: &mut Board, direction: Direction, letters: &Letters) -> Result<(bool, Vec<(usize, usize)>, [usize; 26], LetterUsage), ()> {
    let mut played_indices: Vec<(usize, usize)> = Vec::with_capacity(MAX_WORD_LENGTH);
    match direction {
        Direction::Horizontal => {
            if col_idx + word.len() >= BOARD_SIZE {
                return Err(());
            }
            let mut remaining_letters = letters.clone();
            // Check if the word will start or end at a letter
            let mut valid_loc = (col_idx != 0 && board.get_val(row_idx, col_idx-1) != EMPTY_VALUE) || (BOARD_SIZE-col_idx <= word.len() && board.get_val(row_idx, col_idx+word.len()) != EMPTY_VALUE);
            // Check if the word will border any letters on the top or bottom
            valid_loc |= (col_idx..col_idx+word.len()).any(|c_idx| (row_idx < BOARD_SIZE-1 && board.get_val(row_idx+1, c_idx) != EMPTY_VALUE) || (row_idx > 0 && board.get_val(row_idx-1, c_idx) != EMPTY_VALUE));
            if !valid_loc {
                return Ok((false, played_indices, remaining_letters, LetterUsage::Remaining));
            }
            else{
                let mut entirely_overlaps = true;
                for i in 0..word.len() {
                    if board.get_val(row_idx, col_idx+i) == EMPTY_VALUE {
                        board.set_val(row_idx, col_idx+i, word[i]);
                        played_indices.push((row_idx, col_idx+i));
                        entirely_overlaps = false;
                        let elem = unsafe { remaining_letters.get_unchecked_mut(word[i]) };
                        if *elem == 0 {
                            return Ok((false, played_indices, remaining_letters, LetterUsage::Overused));
                        }
                        *elem -= 1;
                    }
                    else if board.get_val(row_idx, col_idx+i) != word[i] {
                        return Ok((false, played_indices, remaining_letters, LetterUsage::Remaining));
                    }
                }
                if remaining_letters.iter().all(|count| *count == 0) && !entirely_overlaps {
                    return Ok((true, played_indices, remaining_letters, LetterUsage::Finished));
                }
                else {
                    return Ok((!entirely_overlaps, played_indices, remaining_letters, LetterUsage::Remaining));
                }
            }
        },
        Direction::Vertical => {
            if row_idx + word.len() >= BOARD_SIZE {
                return Err(());
            }
            let mut remaining_letters = letters.clone();
            // Check if the word will start or end at a letter
            let mut valid_loc = (row_idx != 0 && board.get_val(row_idx-1, col_idx) != EMPTY_VALUE) || (BOARD_SIZE-row_idx <= word.len() && board.get_val(row_idx+word.len(), col_idx) != EMPTY_VALUE);
            // Check if the word will border any letters on the right or left
            valid_loc |= (row_idx..row_idx+word.len()).any(|r_idx| (col_idx < BOARD_SIZE-1 && board.get_val(r_idx, col_idx+1) != EMPTY_VALUE) || (col_idx > 0 && board.get_val(r_idx, col_idx-1) != EMPTY_VALUE));
            if !valid_loc {
                return Ok((false, played_indices, remaining_letters, LetterUsage::Remaining));
            }
            else{
                let mut entirely_overlaps = true;
                for i in 0..word.len() {
                    if board.get_val(row_idx+i, col_idx) == EMPTY_VALUE {
                        board.set_val(row_idx+i, col_idx, word[i]);
                        played_indices.push((row_idx+i, col_idx));
                        entirely_overlaps = false;
                        let elem = unsafe { remaining_letters.get_unchecked_mut(word[i]) };
                        if *elem == 0 {
                            return Ok((false, played_indices, remaining_letters, LetterUsage::Overused));
                        }
                        *elem -= 1;
                    }
                    else if board.get_val(row_idx+i, col_idx) != word[i] {
                        return Ok((false, played_indices, remaining_letters, LetterUsage::Remaining));
                    }
                }
                if remaining_letters.iter().all(|count| *count == 0) && !entirely_overlaps {
                    return Ok((true, played_indices, remaining_letters, LetterUsage::Finished));
                }
                else {
                    return Ok((!entirely_overlaps, played_indices, remaining_letters, LetterUsage::Remaining));
                }
            }
        }
    }
}

/// Undoes a play on the `board`
/// # Arguments
/// * `board` - `Board` being undone (is modified in-place)
/// * `played_indices` - Vector of the indices in `board` that need to be reset
fn undo_play(board: &mut Board, played_indices: &Vec<(usize, usize)>) {
    for index in played_indices.iter() {
        board.set_val(index.0, index.1, EMPTY_VALUE);
    }
}

/// Checks which words can be played after the first
/// # Arguments
/// * `letters` - Length-26 array of originally available letters
/// * `word_being_checked` - Word that is being checked if playable
/// * `played_on_board` - Set of the letters played on the board
/// # Returns
/// * `bool` - Whether the `word_being_checked` is playable
fn check_filter_after_play(letters: Letters, word_being_checked: &Word, played_on_board: &HashSet<&usize>) -> bool {
    let mut available_letters: [isize; 26] = unsafe { mem::transmute(letters) };//letters.into_iter().map(|l| l as isize).collect();
    let mut already_seen_negative = false;
    for letter in word_being_checked.iter() {
        let elem = unsafe { available_letters.get_unchecked_mut(*letter) };
        if *elem == 0 && !played_on_board.contains(letter) {
            return false;
        }
        else if *elem == 0 && already_seen_negative {
            return false;
        }
        else if *elem == 0 {
            already_seen_negative = true;
        }
        *elem -= 1;
    }
    return true;
}

/// Recursively solves Bananagrams
/// # Arguments
/// * `board` - The `Board` to modify in-place
/// * `min_col` - Minimum occupied column index in `board`
/// * `max_col` - Maximum occupied column index in `board`
/// * `min_row` - Minimum occupied row index in `board`
/// * `max_row` - Maximum occupied row index in `board`
/// * `valid_words_vec` - Vector of vectors, each representing a word (see `convert_word_to_array`)
/// * `valid_words_set` - HashSet of vectors, each representing a word (a HashSet version of `valid_words_vec` for faster membership checking)
/// * `letters` - Length-26 array of the number of each letter in the hand
/// * `depth` - Depth of the current recursive call
/// * `stop_me` - Shared `AtomicBool` used to indicate when early stopping should occur
/// # Returns
/// *`Result` with:*
/// * `bool` - Whether the word could be validly played
/// * `usize` - Minimum occupied column index in `board`
/// * `usize` - Maximum occupied column index in `board`
/// * `usize` - Minimum occupied row index in `board`
/// * `usize` - Maximum occupied row index in `board`
/// 
/// *or empty `Err` on if out-of-bounds*
fn play_further(board: &mut Board, min_col: usize, max_col: usize, min_row: usize, max_row: usize, valid_words_vec: &Vec<Word>, valid_words_set: &HashSet<Word>, letters: Letters, depth: usize, play_sequence: &mut PlaySequence, previous_play_sequence: &PlaySequence, stop_me: &Arc<AtomicBool>) -> Result<(bool, usize, usize, usize, usize), ()> {
    if depth+1 < previous_play_sequence.len() {
        let word = &previous_play_sequence[depth+1].0;
        let row_idx = previous_play_sequence[depth+1].1.0;
        let col_idx = previous_play_sequence[depth+1].1.1;
        let res = play_word(word, row_idx, col_idx, board, previous_play_sequence[depth+1].1.2, &letters)?;
        if res.0 {
            match previous_play_sequence[depth+1].1.2 {
                Direction::Horizontal => {
                    // If the word was played successfully (i.e. it's not a complete overlap and it borders at least one existing tile), then check the validity of the new words it forms
                    let new_min_col = cmp::min(min_col, col_idx);
                    let new_max_col = cmp::max(max_col, col_idx+word.len());
                    let new_min_row = cmp::min(min_row, row_idx);
                    let new_max_row = cmp::max(max_row, row_idx);
                    if is_board_valid_horizontal(board, new_min_col, new_max_col, new_min_row, new_max_row, row_idx, col_idx, col_idx+word.len()-1, valid_words_set) {
                        // If it's valid, go to the next recursive level (where completion will be checked)
                        play_sequence.push((word.clone(), (res.1[0].0, res.1[0].1, Direction::Horizontal)));
                        match res.3 {
                            LetterUsage::Finished => {
                                return Ok((true, new_min_col, new_max_col, new_min_row, new_max_row));
                            },
                            LetterUsage::Remaining => {
                                let res2 = play_further(board, new_min_col, new_max_col, new_min_row, new_max_row, valid_words_vec, valid_words_set, res.2, depth+1, play_sequence, previous_play_sequence, stop_me)?;
                                if res2.0 && !stop_me.load(Ordering::Relaxed) {
                                    // If that recursive stack finishes successfully, we're done! (could have used another Result or Option rather than a bool in the returned tuple, but oh well)
                                    return Ok(res2);
                                }
                                else {
                                    // Otherwise, undo the previous play (cloning the board before each play so we don't have to undo is *way* slower)
                                    play_sequence.pop();
                                    undo_play(board, &res.1);
                                }
                            },
                            LetterUsage::Overused => unreachable!()
                        }
                    }
                    else {
                        // If the play formed some invalid words, undo the previous play
                        undo_play(board, &res.1);
                    }
                },
                Direction::Vertical => {
                    let new_min_col = cmp::min(min_col, col_idx);
                    let new_max_col = cmp::max(max_col, col_idx);
                    let new_min_row = cmp::min(min_row, row_idx);
                    let new_max_row = cmp::max(max_row, row_idx+word.len());
                    if is_board_valid_vertical(board, new_min_col, new_max_col, new_min_row, new_max_row, row_idx, row_idx+word.len()-1, col_idx, valid_words_set) {
                        play_sequence.push((word.clone(), (res.1[0].0, res.1[0].1, Direction::Vertical)));
                        match res.3 {
                            LetterUsage::Finished => {
                                return Ok((true, new_min_col, new_max_col, new_min_row, new_max_row));
                            },
                            LetterUsage::Remaining => {
                                let res2 = play_further(board, new_min_col, new_max_col, new_min_row, new_max_row, valid_words_vec, valid_words_set, res.2, depth+1, play_sequence, previous_play_sequence, stop_me)?;
                                if res2.0 && !stop_me.load(Ordering::Relaxed) {
                                    return Ok(res2);
                                }
                                else {
                                    play_sequence.pop();
                                    undo_play(board, &res.1);
                                }
                            },
                            LetterUsage::Overused => unreachable!()
                        }   
                    }
                    else {
                        undo_play(board, &res.1);
                    }
                }
            }
        }
        else {
            // If trying to play the board was invalid, undo the play
            undo_play(board, &res.1);
        }
        return Ok((false, min_col, max_col, min_row, max_row));
    }
    // If we're at an odd depth, play horizontally first (trying to alternate horizontal-vertical-horizontal as a heuristic to solve faster)
    else if depth % 2 == 1 {
        for word in valid_words_vec.iter() {
            // Stop if we're signalled to
            if stop_me.load(Ordering::Relaxed) {
                return Err(());
            }
            // Try across all rows (starting from one before to one after)
            for row_idx in min_row-1..max_row+2 {
                // For each row, try across all columns (starting from the farthest out the word could be played)
                for col_idx in min_col-word.len()..max_col+2 {
                    // Using the ? because `play_word` can give an `Err` if the index is out of bounds
                    let res = play_word(word, row_idx, col_idx, board, Direction::Horizontal, &letters)?;
                    if res.0 {
                        // If the word was played successfully (i.e. it's not a complete overlap and it borders at least one existing tile), then check the validity of the new words it forms
                        let new_min_col = cmp::min(min_col, col_idx);
                        let new_max_col = cmp::max(max_col, col_idx+word.len());
                        let new_min_row = cmp::min(min_row, row_idx);
                        let new_max_row = cmp::max(max_row, row_idx);
                        if is_board_valid_horizontal(board, new_min_col, new_max_col, new_min_row, new_max_row, row_idx, col_idx, col_idx+word.len()-1, valid_words_set) {
                            // If it's valid, go to the next recursive level (unless we've all the letters, at which point we're done)
                            play_sequence.push((word.clone(), (res.1[0].0, res.1[0].1, Direction::Horizontal)));
                            match res.3 {
                                LetterUsage::Finished => {
                                    return Ok((true, new_min_col, new_max_col, new_min_row, new_max_row));
                                },
                                LetterUsage::Remaining => {
                                    let res2 = play_further(board, new_min_col, new_max_col, new_min_row, new_max_row, valid_words_vec, valid_words_set, res.2, depth+1, play_sequence, previous_play_sequence, stop_me)?;
                                    if res2.0 && !stop_me.load(Ordering::Relaxed) {
                                        // If that recursive stack finishes successfully, we're done! (could have used another Result or Option rather than a bool in the returned tuple, but oh well)
                                        return Ok(res2);
                                    }
                                    else {
                                        // Otherwise, undo the previous play (cloning the board before each play so we don't have to undo is *way* slower)
                                        play_sequence.pop();
                                        undo_play(board, &res.1);
                                    }
                                },
                                LetterUsage::Overused => unreachable!()
                            }
                        }
                        else {
                            // If the play formed some invalid words, undo the previous play
                            undo_play(board, &res.1);
                        }
                    }
                    else {
                        // If trying to play the board was invalid, undo the play
                        undo_play(board, &res.1);
                    }
                }
            }
        }
        // If trying every word horizontally didn't work, try vertically instead
        for word in valid_words_vec.iter() {
            if stop_me.load(Ordering::Relaxed) {
                return Err(());
            }
            // Try down all columns
            for col_idx in min_col-1..max_col+2 {
                // This is analgous to the above
                for row_idx in min_row-word.len()..max_row+2 {
                    let res = play_word(word, row_idx, col_idx, board, Direction::Vertical, &letters)?;
                    if res.0 {
                        let new_min_col = cmp::min(min_col, col_idx);
                        let new_max_col = cmp::max(max_col, col_idx);
                        let new_min_row = cmp::min(min_row, row_idx);
                        let new_max_row = cmp::max(max_row, row_idx+word.len());
                        if is_board_valid_vertical(board, new_min_col, new_max_col, new_min_row, new_max_row, row_idx, row_idx+word.len()-1, col_idx, valid_words_set) {
                            play_sequence.push((word.clone(), (res.1[0].0, res.1[0].1, Direction::Vertical)));
                            match res.3 {
                                LetterUsage::Finished => {
                                    return Ok((true, new_min_col, new_max_col, new_min_row, new_max_row));
                                },
                                LetterUsage::Remaining => {
                                    let res2 = play_further(board, new_min_col, new_max_col, new_min_row, new_max_row, valid_words_vec, valid_words_set, res.2, depth+1, play_sequence, previous_play_sequence, stop_me)?;
                                    if res2.0 && !stop_me.load(Ordering::Relaxed) {
                                        return Ok(res2);
                                    }
                                    else {
                                        play_sequence.pop();
                                        undo_play(board, &res.1);
                                    }
                                },
                                LetterUsage::Overused => unreachable!()
                            }
                        }
                        else {
                            undo_play(board, &res.1);
                        }
                    }
                    else {
                        undo_play(board, &res.1);
                    }
                }
            }
        }
        return Ok((false, min_col, max_col, min_row, max_row));
    }
    // If we're at an even depth, play vertically first. Otherwise this is analgous to the above.
    else {
        for word in valid_words_vec.iter() {
            if stop_me.load(Ordering::Relaxed) {
                return Err(());
            }
            // Try down all columns
            for col_idx in min_col-1..max_col+2 {
                for row_idx in min_row-word.len()..max_row+2 {
                    let res = play_word(word, row_idx, col_idx, board, Direction::Vertical, &letters)?;
                    if res.0 {
                        let new_min_col = cmp::min(min_col, col_idx);
                        let new_max_col = cmp::max(max_col, col_idx);
                        let new_min_row = cmp::min(min_row, row_idx);
                        let new_max_row = cmp::max(max_row, row_idx+word.len());
                        if is_board_valid_vertical(board, new_min_col, new_max_col, new_min_row, new_max_row, row_idx, row_idx+word.len()-1, col_idx, &valid_words_set) {
                            play_sequence.push((word.clone(), (res.1[0].0, res.1[0].1, Direction::Vertical)));
                            match res.3 {
                                LetterUsage::Finished => {
                                    return Ok((true, new_min_col, new_max_col, new_min_row, new_max_row));
                                },
                                LetterUsage::Remaining => {
                                    let res2 = play_further(board, new_min_col, new_max_col, new_min_row, new_max_row, valid_words_vec, valid_words_set, res.2, depth+1, play_sequence, previous_play_sequence, stop_me)?;
                                    if res2.0 && !stop_me.load(Ordering::Relaxed) {
                                        return Ok(res2);
                                    }
                                    else {
                                        play_sequence.pop();
                                        undo_play(board, &res.1);
                                    }
                                },
                                LetterUsage::Overused => unreachable!()
                            }
                        }
                        else {
                            undo_play(board, &res.1);
                        }
                    }
                    else {
                        undo_play(board, &res.1);
                    }
                }
            }
        }
        for word in valid_words_vec.iter() {
            if stop_me.load(Ordering::Relaxed) {
                return Err(());
            }
            // Try across all rows
            for row_idx in min_row-1..max_row+2 {
                for col_idx in min_col-word.len()..max_col+2 {
                    let res = play_word(word, row_idx, col_idx, board, Direction::Horizontal, &letters)?;
                    if res.0 {
                        let new_min_col = cmp::min(min_col, col_idx);
                        let new_max_col = cmp::max(max_col, col_idx+word.len());
                        let new_min_row = cmp::min(min_row, row_idx);
                        let new_max_row = cmp::max(max_row, row_idx);
                        if is_board_valid_horizontal(board, new_min_col, new_max_col, new_min_row, new_max_row, row_idx, col_idx, col_idx+word.len()-1, valid_words_set) {
                            play_sequence.push((word.clone(), (res.1[0].0, res.1[0].1, Direction::Horizontal)));
                            match res.3 {
                                LetterUsage::Finished => {
                                    return Ok((true, new_min_col, new_max_col, new_min_row, new_max_row));
                                },
                                LetterUsage::Remaining => {
                                    let res2 = play_further(board, new_min_col, new_max_col, new_min_row, new_max_row, valid_words_vec, valid_words_set, res.2, depth+1, play_sequence, previous_play_sequence, stop_me)?;
                                    if res2.0 && !stop_me.load(Ordering::Relaxed) {
                                        return Ok(res2);
                                    }
                                    else {
                                        play_sequence.pop();
                                        undo_play(board, &res.1);
                                    }
                                },
                                LetterUsage::Overused => unreachable!()
                            }
                        }
                        else {
                            undo_play(board, &res.1);
                        }
                    }
                    else {
                        undo_play(board, &res.1);
                    }
                }
            }
        }
        return Ok((false, min_col, max_col, min_row, max_row));
    }
}

/// Tries to play a single letter on the board
/// # Arguments
/// * `board` - the `Board` on which to try to play the `letter`
/// * `min_col` - Minimum occupied column index in `board`
/// * `max_col` - Maximum occupied column index in `board`
/// * `min_row` - Minimum occupied row index in `board`
/// * `max_row` - Maximum occupied row index in `board`
/// * `letter` - The numeric representation of the letter to play
/// * `valid_words_set` - HashSet of all valid words
/// # Returns
/// `Option` - either `None` if no solution was found, or a `Some` tuple of `(row, col, new_min_col, new_max_col, new_min_row, new_max_row)` on success
fn play_one_letter(board: &mut Board, min_col: usize, max_col: usize, min_row: usize, max_row: usize, letter: usize, valid_words_set: &HashSet<Word>) -> Option<(usize, usize, usize, usize, usize, usize)> {
    // Loop through all possible locations and check if the letter works there
    for row in min_row-1..max_row+2 {
        for col in min_col-1..max_col+2 {
            if row < BOARD_SIZE && col < BOARD_SIZE && board.get_val(row, col) == EMPTY_VALUE {   // row/col don't need to be checked if they're greater than 0 since they'd underflow
                if (col > 0 && board.get_val(row, col-1) != EMPTY_VALUE) || (col < BOARD_SIZE-1 && board.get_val(row, col+1) != EMPTY_VALUE) || (row > 0 && board.get_val(row-1, col) != EMPTY_VALUE) || (row < BOARD_SIZE-1 && board.get_val(row+1, col) != EMPTY_VALUE) {
                    board.set_val(row, col, letter);
                    let new_min_col = cmp::min(min_col, col);
                    let new_max_col = cmp::max(max_col, col);
                    let new_min_row = cmp::min(min_row, row);
                    let new_max_row = cmp::max(max_row, row);
                    // Could also use `is_board_valid_vertical`
                    if is_board_valid_horizontal(board, new_min_col, new_max_col, new_min_row, new_max_row, row, col, col, valid_words_set) {
                        // If it's valid, return the (potentially) new bounds, along with the location the letter was played
                        return Some((row, col, new_min_col, new_max_col, new_min_row, new_max_row));
                    }
                    else {
                        // If the board wasn't ok, reset this spot
                        board.set_val(row, col, EMPTY_VALUE);
                    }
                }
            }
        }
    }
    // Return `None` if we don't find a solution
    return None;
}

/// Attempts to play off an existing board
/// # Arguments
/// * `previous_play_sequence` - Sequence of previous played moves
/// * `valid_words_vec` - Vector of valid words for the given hand of letters
/// * `valid_words_set` - HashSet of valid words (HashSet of `valid_words_vec` for faster membership checking)
/// * `letters` - Array of the number of each letter in the hand
/// # Returns
/// `Option` with:
/// * `Board` - updated board
/// * `PlaySequence` - updated play sequence
/// * `usize` - Minimum occupied column index in `board`
/// * `usize` - Maximum occupied column index in `board`
/// * `usize` - Minimum occupied row index in `board`
/// * `usize` - Maximum occupied row index in `board`
/// 
/// *or `None` if no valid play can be made on the existing board*
fn play_existing(previous_play_sequence: &PlaySequence, valid_words_vec: &Vec<Word>, valid_words_set: &HashSet<Word>, letters: &Letters) -> Option<(Board, PlaySequence, usize, usize, usize, usize)> {
    let mut board = Board::new();
    let row = previous_play_sequence[0].1.0;
    let col_start = previous_play_sequence[0].1.1;
    let word = &previous_play_sequence[0].0;
    let mut use_letters = letters.clone();
    let mut word_letters = HashSet::with_capacity(word.len());
    for i in 0..word.len() {
        board.set_val(row, col_start+i, word[i]);
        use_letters[word[i]] -= 1;
        word_letters.insert(&word[i]);
    }
    let min_col = col_start;
    let min_row = row;
    let max_col = col_start + (word.len()-1);
    let max_row = row;
    let mut play_sequence: PlaySequence = Vec::with_capacity(50);
    play_sequence.push((word.clone(), (row, col_start, Direction::Horizontal)));
    if use_letters.iter().all(|count| *count == 0) {
        return Some((board, play_sequence, min_col, max_col, min_row, max_row));
    }
    else {
        let new_valid_words_vec: Vec<Word> = valid_words_vec.iter().filter(|word| check_filter_after_play(use_letters.clone(), word, &word_letters)).map(|word| word.clone()).collect();
        let stop = Arc::new(AtomicBool::new(false));    // Just to match the signature
        let res = play_further(&mut board, min_col, max_col, min_row, max_row, &new_valid_words_vec, valid_words_set, use_letters, 0, &mut play_sequence, previous_play_sequence, &stop);
        match res {
            Ok(result) => {
                if result.0 {
                    return Some((board, play_sequence, result.1, result.2, result.3, result.4));
                }
                else {
                    return None;
                }
            },
            _ => None
        }
    }
}

/// For comparing a current hand of letters to a previous hand
enum LetterComparison {
    /// At least one letter has fewer than the previous letter
    SomeLess,
    /// All letters are the same except exactly one is greater by exactly one
    GreaterByOne,
    /// One or more letters are greater by one or more
    GreaterByMoreThanOne,
    /// The hand is the same as before
    Same
}

/// Struct returned when getting playable words
#[derive(Serialize)]
struct PlayableWords {
    /// Playable words using the shorter dictionary
    short: Vec<String>,
    /// Playable words using the whole Scrabble dictionary
    long: Vec<String>
}

/// Struct returned when a board is solved
#[derive(Serialize)]
struct Solution {
    /// The solved board
    board: Vec<Vec<char>>,
    /// How long it took to solve the board
    elapsed: u128
}

/// The previous game state
struct GameState {
    /// The previous board
    board: Board,
    /// The minimum played column in `board`
    min_col: usize,
    /// The maximum played column in `board`
    max_col: usize,
    /// The minimum played row in `board`
    min_row: usize,
    /// The maximum played row in `board`
    max_row: usize,
    /// The hand used to make `board`
    letters: Letters,
    /// The indices played at each level of the recursive chain
    play_sequence: PlaySequence
}

/// Controls the state of the app
struct AppState {
    /// Dictionary of the ~20k most common words in English
    all_words_short: Vec<Word>,
    /// Complete Scrabble dictionary
    all_words_long: Vec<Word>,
    /// The last game state (if `None`, then no previous game has been played)
    last_game: Mutex<Option<GameState>>
}

/// Generates random letters based on user input
/// # Arguments
/// * `what` - Whether to generate characters from an "infinite set" (i.e. all are equal likelihood),
/// or selected from "standard Bananagrams" (144 tiles) or "double Bananagrams" (288 tiles)
/// * `how_many` - How many tiles to randomly generate; must be greater than 0, and less than 144 for regular Bananagrams,
/// or 288 for double
/// # Returns
/// `Result` of mapping of each uppercase Latin character to the number of times it's present
/// 
/// *or String `Err` upon failure*
#[tauri::command]
async fn get_random_letters(what: String, how_many: i64, _state: State<'_, AppState>) -> Result<HashMap<char, u64>, String> {
    if how_many < 1 {
        return Err("The number to choose should be greater than 0".to_owned());
    }
    let mut rng = thread_rng();
    let mut return_chars: HashMap<char, u64> = HashMap::with_capacity(26);
    UPPERCASE.chars().for_each(|c| {return_chars.insert(c, 0);});
    if what == "infinite set" {
        // For "infinite set", randomly generate characters
        let uni = Uniform::new_inclusive(0, 25);
        for _ in 0..how_many {
            let random_num: u8 = uni.sample(&mut rng);
            let random_char = (random_num+65) as char;
            let old_val = return_chars.get(&random_char);
            match old_val {
                Some(v) => {
                    return_chars.insert(random_char, v+1);
                },
                None => {
                    return Err(format!("Missing value in return dictionary: {}", random_char));
                }
            }
        }
    }
    else if what == "standard Bananagrams" {
        if how_many > 144 {
            return Err("The number to choose must be less than 144 for standard Banangrams".to_owned());
        }
        // For regular Bananagrams, first make the vector of characters to choose form
        let mut to_choose_from: Vec<char> = Vec::with_capacity(144);
        for (i, c) in UPPERCASE.chars().enumerate() {
            for _num_letter in 0..REGULAR_TILES[i] {
                to_choose_from.push(c);
            }
        }
        // Then selected `how_many` characters from that vector
        let selected_chars: Vec<char> = to_choose_from.choose_multiple(&mut rng, how_many as usize).cloned().collect();
        for i in 0..selected_chars.len() {
            let old_val = return_chars.get(&selected_chars[i]);
            match old_val {
                Some(v) => {
                    return_chars.insert(selected_chars[i], v+1);
                },
                None => {
                    return Err(format!("Missing value in return dictionary: {}", selected_chars[i]));
                }
            }
        }
    }
    else if what == "double Bananagrams" {
        // "double Bananagrams" is just like regular, except with twice as many pieces
        if how_many > 288 {
            return Err("The number to choose must be less than 288 for double Banangrams".to_owned());
        }
        let mut to_choose_from: Vec<char> = Vec::with_capacity(288);
        for (i, c) in UPPERCASE.chars().enumerate() {
            for _num_letter in 0..REGULAR_TILES[i]*2 {
                to_choose_from.push(c);
            }
        }
        let selected_chars: Vec<char> = to_choose_from.choose_multiple(&mut rng, how_many as usize).cloned().collect();
        for i in 0..selected_chars.len() {
            let old_val = return_chars.get(&selected_chars[i]);
            match old_val {
                Some(v) => {
                    return_chars.insert(selected_chars[i], v+1);
                },
                None => {
                    return Err(format!("Missing value in return dictionary: {}", selected_chars[i]));
                }
            }
        }
    }
    else {
        return Err(format!("`what` must be \"infinite set\", \"standard Bananagrams\", or \"double Bananagrams\", not {}", what))
    }
    return Ok(return_chars);
}

/// Async command executed by the frontend to get the playable words for a given hand of letters
/// # Arguments
/// * `available_letters` - `HashMap` (from JavaScript object) mapping string letters to numeric quanity of each letter
/// * `state` - Current state of the app
/// # Returns
/// `Result` of `PlayableWords` with two keys - "short" (common words playable using `available_letters`) and "long" (Scrabble words playable using `available_letters`)
/// 
/// *or String `Err` upon failure*
#[tauri::command]
async fn get_playable_words(available_letters: HashMap<String, i64>, state: State<'_, AppState>) -> Result<PlayableWords, String> {
    // Check if we have all the letters from the frontend
    let mut letters = [0usize; 26];
    for c in UPPERCASE.chars() {
        let num = available_letters.get(&c.to_string());
        match num {
            Some(number) => {
                if *number < 0 {
                    return Err(format!("Number of letter {} is {}, but must be greater than or equal to 0!", c, number));
                }
                letters[(c as usize) - 65] = *number as usize;
            },
            None => {
                return Err(format!("Missing letter: {}", c));
            }
        }
    }
    let playable_short: Vec<String> = state.all_words_short.iter().filter(|word| is_makeable(word, letters)).map(|word| convert_array_to_word(word)).collect();
    let playable_long: Vec<String> = state.all_words_long.iter().filter(|word| is_makeable(word, letters)).map(|word| convert_array_to_word(word)).collect();
    return Ok(PlayableWords { short: playable_short, long: playable_long })
}

/// Async command executed by the frontend to reset the Banangrams board
/// # Arguments
/// * `state` - Current state of the app
/// # Returns
/// Empty `Result` upon success
/// 
/// *or empty `Err` if out-of-bounds*
#[tauri::command]
async fn reset(state: State<'_, AppState>) -> Result<(), String> {
    let mut last_game_state = state.last_game.lock().or(Err("Failed to get lock on the last game state"))?;
    *last_game_state = None;
    Ok(())
}

/// Async command executed by the frontend to solve a Bananagrams board
/// # Arguments
/// * `available_letters` - `HashMap` (from JavaScript object) mapping string letters to numeric quanity of each letter
/// * `state` - Current state of the app
/// # Returns
/// `Result` as a `Solution` with a vector of vector of chars of the solution and the elapsed time
/// 
/// *or String `Err` upon failure or not finding a tile (with the reason indicated in the String)*
#[tauri::command]
async fn play_bananagrams(available_letters: HashMap<String, i64>, state: State<'_, AppState>) -> Result<Solution, String> {
    let now = Instant::now();
    // Check if we have all the letters from the frontend
    let mut letters = [0usize; 26];
    for c in UPPERCASE.chars() {
        let num = available_letters.get(&c.to_string());
        match num {
            Some(number) => {
                if *number < 0 {
                    return Err(format!("Number of letter {} is {}, but must be greater than or equal to 0!", c, number));
                }
                letters[(c as usize) - 65] = *number as usize;
            },
            None => {
                return Err(format!("Missing letter: {}", c));
            }
        }
    }
    // Check whether a board has been played already
    let mut last_game_state: std::sync::MutexGuard<'_, Option<GameState>>;
    match state.last_game.lock() {
        Ok(locked) => {
            last_game_state = locked;
        },
        _ => {
            return Err("Failed to get lock on last game state".to_owned());
        }
    }
    match &*last_game_state {   // I don't like &*
        Some(prev_state) => {
            // If a board has been played, check whether the letters are the same as before, or if some are more or less
            let mut comparison = LetterComparison::Same;
            let mut seen_greater: usize = EMPTY_VALUE;
            for i in 0..26 {
                if letters[i] < prev_state.letters[i] {
                    // Any less means we re-do the board, so we can break here
                    comparison = LetterComparison::SomeLess;
                    break;
                }
                else if letters[i] > prev_state.letters[i] && (seen_greater != EMPTY_VALUE || letters[i] - prev_state.letters[i] != 1) {
                    comparison = LetterComparison::GreaterByMoreThanOne;
                }
                else if letters[i] > prev_state.letters[i] {
                    comparison = LetterComparison::GreaterByOne;
                    seen_greater = i;
                }
            }
            match comparison {
                LetterComparison::Same => {
                    // If the hand is the same then no need to do anything
                    return Ok(Solution { board: board_to_vec(&prev_state.board, prev_state.min_col, prev_state.max_col, prev_state.min_row, prev_state.max_row), elapsed: now.elapsed().as_millis() });
                },
                LetterComparison::GreaterByOne => {
                    // If only a single letter has increased by one, then first check just that letter
                    let valid_words_vec: Vec<Word> = state.all_words_short.iter().filter(|word| is_makeable(word, letters)).map(|word| word.clone()).collect();
                    let valid_words_set: HashSet<Word> = HashSet::from_iter(valid_words_vec.clone());
                    let mut board = prev_state.board.clone();
                    let res = play_one_letter(&mut board, prev_state.min_col, prev_state.max_col, prev_state.min_row, prev_state.max_row, seen_greater, &valid_words_set);
                    match res {
                        Some(result) => {
                            let mut play_sequence = prev_state.play_sequence.clone();
                            play_sequence.push((vec![seen_greater], (result.0, result.1, Direction::Horizontal)));
                            *last_game_state = Some(GameState { board: board.clone(), min_col: result.2, max_col: result.3, min_row: result.4, max_row: result.5, letters, play_sequence });
                            return Ok(Solution { board: board_to_vec(&board, result.2, result.3, result.4, result.5), elapsed: now.elapsed().as_millis() });
                        },
                        None => {
                            // If we failed when playing one letter, try playing off the existing board
                            let attempt = play_existing(&prev_state.play_sequence, &valid_words_vec, &valid_words_set, &letters);
                            match attempt {
                                Some(result) => {
                                    *last_game_state = Some(GameState { board: result.0.clone(), min_col: result.2, max_col: result.3, min_row: result.4, max_row: result.5, letters, play_sequence: result.1.clone() });
                                    return Ok(Solution { board: board_to_vec(&result.0, result.2, result.3, result.4, result.5), elapsed: now.elapsed().as_millis() });
                                },
                                None => {/* If we failed again, continue with the code that starts from scratch */}
                            }
                        }
                    }
                },
                LetterComparison::GreaterByMoreThanOne => {
                    // If a letter has increased by more than one, or multiple have increased by one or more, then try playing off the existing board
                    let valid_words_vec: Vec<Word> = state.all_words_short.iter().filter(|word| is_makeable(word, letters)).map(|word| word.clone()).collect();
                    let valid_words_set: HashSet<Word> = HashSet::from_iter(valid_words_vec.clone());
                    let attempt = play_existing(&prev_state.play_sequence, &valid_words_vec, &valid_words_set, &letters);
                    match attempt {
                        Some(result) => {
                            *last_game_state = Some(GameState { board: result.0.clone(), min_col: result.2, max_col: result.3, min_row: result.4, max_row: result.5, letters, play_sequence: result.1.clone() });
                            return Ok(Solution { board: board_to_vec(&result.0, result.2, result.3, result.4, result.5), elapsed: now.elapsed().as_millis() });
                        },
                        None => {/* If we failed, continue with the code that starts from scratch */}
                    }
                }
                LetterComparison::SomeLess => {/* We just want to continue to the code that starts from scratch */}
            }
        },
        None => {/* We just want to continue to the code that starts from scratch */}
    }
    // Play from scratch
    // Get a vector of all valid words
    let valid_words_vec: Vec<Word> = state.all_words_short.iter().filter(|word| is_makeable(word, letters)).map(|word| word.clone()).collect();
    if valid_words_vec.len() == 0 {
        return Err("No valid words can be formed from the current letters - dump and try again!".to_owned());
    }
    // Split the words to check up into appropriate chunks based on the available parallelism
    let mut default_parallelism_approx = 1usize;
    match thread::available_parallelism() {
        Ok(available_parallelism) => {
            default_parallelism_approx = available_parallelism.into();
        },
        _ => {}
    }
    let chunk_size = (valid_words_vec.len() as f32)/(default_parallelism_approx as f32);
    let chunks: Vec<Vec<Word>> = valid_words_vec.chunks(chunk_size.ceil() as usize).map(|words| words.to_vec()).collect();
    // Prepare for threading/early termination using `AtomicBool`
    let stop = Arc::new(AtomicBool::new(false));
    let mut handles: Vec<thread::JoinHandle<()>> = Vec::with_capacity(chunks.len());
    let char_vec: Vec<(Vec<Vec<char>>, Board, usize, usize, usize, usize, PlaySequence)> = Vec::new();
    let ret_val = Arc::new(Mutex::new(char_vec));
    // For each thread (i.e. piece of available parallelism), spawn a new thread to check those words
    // These threads check different sets of initial words in the board, and whichever finishes first signals the others to stop
    for chunk in chunks.into_iter() {
        let stop_t = stop.clone();
        let new_letters = letters.clone();
        let copied_new_valid_words_vec = valid_words_vec.clone();
        let conn = ret_val.clone();
        let handle = thread::spawn(move || {
            // Loop through each word and play it on a new board
            for word in chunk.iter() {
                let mut board = Board::new();
                let col_start = BOARD_SIZE/2 - word.len()/2;
                let row = BOARD_SIZE/2;
                let mut use_letters: [usize; 26] = new_letters.clone();
                for i in 0..word.len() {
                    board.set_val(row, col_start+i, word[i]);
                    use_letters[word[i]] -= 1;
                }
                let min_col = col_start;
                let min_row = row;
                let max_col = col_start + (word.len()-1);
                let max_row = row;
                let mut play_sequence: PlaySequence = Vec::with_capacity(50);
                play_sequence.push((word.clone(), (row, col_start, Direction::Horizontal)));
                if use_letters.iter().all(|count| *count == 0) {
                    if !stop_t.load(Ordering::Relaxed) {
                        stop_t.store(true, Ordering::Relaxed);
                        // The expect may panic the thread but I think that's ok
                        let mut ret = conn.lock().expect("Failed to get lock on shared ret_val");
                        ret.push((board_to_vec(&board, min_col, max_col, min_row, max_row), board.clone(), min_col, max_col, min_row, max_row, play_sequence));
                        break;
                    }
                }
                else {
                    // Reduce the set of remaining words to check to those that can be played with the letters not in the first word (plus only one of the tiles played in the first word)
                    let word_letters: HashSet<&usize> = HashSet::from_iter(word.iter());
                    let new_valid_words_vec: Vec<Word> = copied_new_valid_words_vec.iter().filter(|word| check_filter_after_play(use_letters.clone(), word, &word_letters)).map(|word| word.clone()).collect();
                    let valid_words_set: HashSet<Word> = HashSet::from_iter(copied_new_valid_words_vec.clone());
                    // Begin the recursive processing
                    let result = play_further(&mut board, min_col, max_col, min_row, max_row, &new_valid_words_vec, &valid_words_set, use_letters, 0, &mut play_sequence, &Vec::new(), &stop_t);
                    match result {
                        // If the result was good, then store it and signal other threads to finish (so long as another thread isn't doing so)
                        Ok(res) => {
                            if res.0 && !stop_t.load(Ordering::Relaxed) {
                                stop_t.store(true, Ordering::Relaxed);
                                // The expect will panic the thread but I think that's ok
                                let mut ret = conn.lock().expect("Failed to get lock on shared ret_val");
                                ret.push((board_to_vec(&board, res.1, res.2, res.3, res.4), board.clone(), res.1, res.2, res.3, res.4, play_sequence));
                                break;
                            }
                        },
                        // If an error (we're out of bounds or another thread signalled to stop) then we're done
                        Err(()) => {
                            break;
                        }
                    }
                }
            }
        });
        handles.push(handle);
    }
    // Wait for all the threads
    for handle in handles.into_iter() {
        let _res = handle.join();
    }
    // If we're done, store the result in the `State` and return the result to the frontend
    let ret: std::sync::MutexGuard<'_, Vec<(Vec<Vec<char>>, Board, usize, usize, usize, usize, Vec<(Vec<usize>, (usize, usize, Direction))>)>>;
    match ret_val.lock() {
        Ok(locked) => {
            ret = locked;
        },
        _ => {
            return Err("Failed to get lock on shared ret_val when checking return".to_owned());
        }
    }
    if ret.len() > 0 {
        *last_game_state = Some(GameState { board: ret[0].1.clone(), min_col: ret[0].2, max_col: ret[0].3, min_row: ret[0].4, max_row: ret[0].5, letters, play_sequence: ret[0].6.clone() });
        return Ok(Solution { board: ret[0].0.clone(), elapsed: now.elapsed().as_millis() });
    }
    // Try again with all words in the Scrabble dictionary; this could potentially take much longer but is done just to be exhaustive
    let valid_words_vec: Vec<Word> = state.all_words_long.iter().filter(|word| is_makeable(word, letters)).map(|word| word.clone()).collect();
    if valid_words_vec.len() == 0 {
        return Err("No valid words can be formed from the current letters - dump and try again!".to_owned());
    }
    let chunks: Vec<Vec<Word>> = valid_words_vec.chunks(chunk_size.ceil() as usize).map(|words| words.to_vec()).collect();
    let stop = Arc::new(AtomicBool::new(false));
    let mut handles: Vec<thread::JoinHandle<()>> = Vec::with_capacity(chunks.len());
    let char_vec: Vec<(Vec<Vec<char>>, Board, usize, usize, usize, usize, PlaySequence)> = Vec::new();
    let ret_val = Arc::new(Mutex::new(char_vec));
    for chunk in chunks.into_iter() {
        let stop_t = stop.clone();
        let new_letters = letters.clone();
        let copied_new_valid_words_vec = valid_words_vec.clone();
        let conn = ret_val.clone();
        let handle = thread::spawn(move || {
            for word in chunk.iter() {
                let mut board = Board::new();
                let col_start = BOARD_SIZE/2 - word.len()/2;
                let row = BOARD_SIZE/2;
                let mut use_letters = new_letters.clone();
                for i in 0..word.len() {
                    board.set_val(row, col_start+i, word[i]);
                    use_letters[word[i]] -= 1;
                }
                let min_col = col_start;
                let min_row = row;
                let max_col = col_start + (word.len()-1);
                let max_row = row;
                let mut play_sequence: PlaySequence = Vec::with_capacity(50);
                play_sequence.push((word.clone(), (row, col_start, Direction::Horizontal)));
                if use_letters.iter().all(|count| *count == 0) {
                    if !stop_t.load(Ordering::Relaxed) {
                        stop_t.store(true, Ordering::Relaxed);
                        // The expect may panic the thread but I think that's ok
                        let mut ret = conn.lock().expect("Failed to get lock on shared ret_val");
                        ret.push((board_to_vec(&board, min_col, max_col, min_row, max_row), board.clone(), min_col, max_col, min_row, max_row, play_sequence));
                        break;
                    }
                }
                else {
                    let word_letters: HashSet<&usize> = HashSet::from_iter(word.iter());
                    let new_valid_words_vec: Vec<Word> = copied_new_valid_words_vec.iter().filter(|word| check_filter_after_play(use_letters.clone(), word, &word_letters)).map(|word| word.clone()).collect();
                    let valid_words_set: HashSet<Word> = HashSet::from_iter(copied_new_valid_words_vec.clone());
                    let result = play_further(&mut board, min_col, max_col, min_row, max_row, &new_valid_words_vec, &valid_words_set, use_letters, 0, &mut play_sequence, &Vec::new(), &stop_t);
                    match result {
                        Ok(res) => {
                            if res.0 && !stop_t.load(Ordering::Relaxed) {
                                stop_t.store(true, Ordering::Relaxed);
                                // The expect will panic the thread but I think that's ok
                                let mut ret = conn.lock().expect("Failed to get lock on shared ret_val");
                                ret.push((board_to_vec(&board, res.1, res.2, res.3, res.4), board.clone(), res.1, res.2, res.3, res.4, play_sequence));
                                break;
                            }
                        },
                        Err(()) => {
                            break;
                        }
                    }
                }
            }
        });
        handles.push(handle);
    }
    for handle in handles.into_iter() {
        let _res = handle.join();
    }
    let ret_long: std::sync::MutexGuard<'_, Vec<(Vec<Vec<char>>, Board, usize, usize, usize, usize, Vec<(Vec<usize>, (usize, usize, Direction))>)>>;
    match ret_val.lock() {
        Ok(locked) => {
            ret_long = locked;
        },
        _ => {
            return Err("Failed to get lock on shared ret_val when checking return".to_owned());
        }
    }
    if ret_long.len() > 0 {
        *last_game_state = Some(GameState { board: ret_long[0].1.clone(), min_col: ret_long[0].2, max_col: ret_long[0].3, min_row: ret_long[0].4, max_row: ret_long[0].5, letters, play_sequence: ret_long[0].6.clone() });
        return Ok(Solution { board: ret_long[0].0.clone(), elapsed: now.elapsed().as_millis() });
    }
    return Err("No solution found - dump and try again!".to_owned());
}

fn main() {
    let all_words_short: Vec<Word> = include_str!("short_dictionary.txt").lines().map(convert_word_to_array).collect();
    let all_words_long: Vec<Word> = include_str!("dictionary.txt").lines().map(convert_word_to_array).collect();
    tauri::Builder::default()
        .manage(AppState { all_words_short, all_words_long, last_game: None.into() })
        .invoke_handler(tauri::generate_handler![play_bananagrams, reset, get_playable_words, get_random_letters])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
