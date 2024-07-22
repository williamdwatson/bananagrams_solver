// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::{cmp, fmt, thread, usize, collections::HashMap};
use hashbrown::HashSet;
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
type BoardAndIdxs = (Board, usize, usize, usize, usize);

/// The maximum length of any word in the dictionary
const MAX_WORD_LENGTH: usize = 17;
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
    /// The underlying vector of the board (as in optimization level 0 the array overflows the stack)
    arr: Vec<usize>
}
impl Board {
    /// Creates a new board of dimensions `BOARD_SIZE`x`BOARD_SIZE` filled with the `EMPTY_VALUE`
    fn new() -> Board {
        return Board { arr: vec![EMPTY_VALUE; BOARD_SIZE*BOARD_SIZE] }
    }

    /// Gets a value from the board at the given index
    /// # Arguments
    /// * `row` - Row index of the value to get (must be less than `BOARD_SIZE`)
    /// * `col` - Column index of the value to get (must be less than `BOARD_SIZE`)
    /// # Returns
    /// `usize` - The value in the board at `(row, col)`
    fn get_val(&self, row: usize, col: usize) -> usize {
        return *self.arr.get(row*BOARD_SIZE + col).expect("Index not in range!");
    }

    /// Sets a value in the board at the given index
    /// # Arguments
    /// * `row` - Row index of the value to get (must be less than `BOARD_SIZE`)
    /// * `col` - Column index of the value to get (must be less than `BOARD_SIZE`)
    /// * `val` - Value to set at `(row, col)` in the board
    fn set_val(&mut self, row: usize, col: usize, val: usize) {
        let v = self.arr.get_mut(row*BOARD_SIZE + col).expect("Index not in range!");
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
fn _board_to_string(board: &Board, min_col: usize, max_col: usize, min_row: usize, max_row: usize) -> String {
    let mut board_string: Vec<char> = Vec::with_capacity((max_row-min_row)*(max_col-min_col));
    for row in min_row..max_row+1 {
        for col in min_col..max_col+1 {
            if board.get_val(row, col) == EMPTY_VALUE {
                board_string.push(' ');
            }
            else {
                board_string.push((board.get_val(row, col) as u8+65) as char);
            }
        }
        board_string.push('\n');
    }
    let s: String = board_string.iter().collect();
    return s.trim_end().to_owned();
}

/// Converts a `board` to a vector of vectors of chars
/// # Arguments
/// * `board` - Board to display
/// * `min_col` - Minimum occupied column index
/// * `max_col` - Maximum occupied column index
/// * `min_row` - Minimum occupied row index
/// * `max_row` - Maximum occupied row index
/// # Returns
/// * `Vec<Vec<char>>` - `board` in vector form (with all numbers converted to letters)
fn board_to_vec(board: &Board, min_col: usize, max_col: usize, min_row: usize, max_row: usize, previous_idxs: &HashSet<(usize, usize)>) -> Vec<Vec<String>> {
    let mut board_vec: Vec<Vec<String>> = Vec::with_capacity(max_row-min_row);
    for row in min_row..max_row+1 {
        let mut row_vec: Vec<String> = Vec::with_capacity(max_col-min_col);
        for col in min_col..max_col+1 {
            if board.get_val(row, col) == EMPTY_VALUE {
                row_vec.push(' '.to_string());
            }
            else {
                if !previous_idxs.contains(&(row, col)) {
                    row_vec.push(((board.get_val(row, col) as u8+65) as char).to_string());
                }
                else {
                    row_vec.push(((board.get_val(row, col) as u8+65) as char).to_string() + "*");
                }
            }
        }
        board_vec.push(row_vec);
    }
    return board_vec;
}

/// Gets which indices overlap between `previous_board` and `new_board`
/// # Arguments
/// * `previous_board` - The previous board
/// * `new_board` - The new board
/// * `previous_min_col` - The minimum played column in `previous_board`
/// * `previous_max_col` - The maximum played column in `previous_board`
/// * `previous_min_row` - The minimum played row in `previous_board`
/// * `previous_max_row` - The maximum played row in `previous_board`
/// * `new_min_col` - The minimum played column in `new_board`
/// * `new_max_col` - The maximum played column in `new_board`
/// * `new_min_row` - The minimum played row in `new_board`
/// * `new_max_row` - The maximum played row in `new_board`
/// # Returns
/// `HashSet` - Set of the indices where `previous_board` and `new_board` have the same value
fn get_board_overlap(previous_board: &Board, new_board: &Board, previous_min_col: usize, previous_max_col: usize, previous_min_row: usize, previous_max_row: usize, new_min_col: usize, new_max_col: usize, new_min_row: usize, new_max_row: usize) -> HashSet<(usize, usize)> {
    let mut overlapping_idxs: HashSet<(usize, usize)> = HashSet::new();
    for row in cmp::max(previous_min_row, new_min_row)..cmp::min(previous_max_row+1, new_max_row+1) {
        for col in cmp::max(previous_min_col, new_min_col)..cmp::min(previous_max_col+1, new_max_col+1) {
            if previous_board.get_val(row, col) != EMPTY_VALUE && previous_board.get_val(row, col) == new_board.get_val(row, col) {
                overlapping_idxs.insert((row, col));
            }
        }
    }
    return overlapping_idxs;
}

/// Gets the minimum and maximum occupied row and column from a `board` (assuming that tiles have only been removed)
/// # Arguments
/// * `board` - The `Board` to check
/// * `old_min_col` - The previous mimimum occupied column
/// * `old_max_col` - The previous maximum occupied column
/// * `old_min_row` - The previous minimum occupied row
/// * `old_max_row` - The previous maximum occupied row
/// * `except_vec` - Vector of indices to ignore when checking if occupied
/// # Returns
/// * `usize` - New minimum occupied column (never smaller than `old_min_col`)
/// * `usize` - New maximum occupied column (never greater than `old_min_row`)
/// * `usize` - New minimum occupied row (never smaller than `old_min_row`)
/// * `usize` - New maximum occupied row (never greater than `old_max_row`)
fn get_new_min_max(board: &Board, old_min_col: usize, old_max_col: usize, old_min_row: usize, old_max_row: usize, except_vec: &Vec<(usize, usize)>) -> (usize, usize, usize, usize) {
    let mut except_idxs: HashSet<&(usize, usize)> = HashSet::with_capacity(except_vec.len());
    for idxs in except_vec {
        except_idxs.insert(idxs);
    }
    // Start at the old minimum row and check if that row or any subsequent ones have any non-empty values
    let mut min_row = old_min_row;
    for row in old_min_row..old_max_row+1 {
        if (old_min_col..old_max_col).any(|col| !except_idxs.contains(&(row, col)) && board.get_val(row, col) != EMPTY_VALUE) {
            break;
        }
        min_row += 1;
    }
    // Start at the test max_row and work our way down
    let mut max_row = old_max_row;
    while max_row > min_row {
        if (old_min_col..old_max_col).any(|col| !except_idxs.contains(&(max_row, col)) && board.get_val(max_row, col) != EMPTY_VALUE) {
            break;
        }
        max_row -= 1;
    }
    // Now do down columns
    let mut min_col = old_min_col;
    for col in old_min_col..old_max_col+1 {
        if (min_row..max_row).any(|row| !except_idxs.contains(&(row, col)) && board.get_val(row, col) != EMPTY_VALUE) {
            break;
        }
        min_col += 1;
    }
    let mut max_col = old_max_col;
    while max_col > min_col {
        if (min_row..max_row).any(|row| !except_idxs.contains(&(row, max_col)) && board.get_val(row, max_col) != EMPTY_VALUE) {
            break;
        }
        max_col -= 1;
    }
    return (min_col, max_col, min_row, max_row);
}

/// Gets a vector of vectors of each part of a word that can be validly removed from the `board`
/// # Arguments
/// * `board` - The board to check for removable word parts
/// * `min_col` - The minimum played column
/// * `max_col` - The maximum played column
/// * `min_row` - The minimum played row
/// * `max_row` - The maximum played row
/// # Returns
/// `Vec` - Vector of vectors of length-2 index tuples of the indices of `board` that can be validly removed
fn get_removable_indices(board: &Board, min_col: usize, max_col: usize, min_row: usize, max_row: usize) -> Vec<(Vec<(usize, usize)>, usize, usize, usize, usize)> {
    if max_col <= min_col || max_row <= min_row {
        return Vec::new();
    }
    let mut removable: Vec<(Vec<(usize, usize)>, usize, usize, usize, usize)> = Vec::with_capacity((max_col - min_col) + (max_row - min_row));
    let mut board_empty = true;
    // First get horizontal removable word parts
    for row in min_row..max_row+1 {
        let mut current_word_part: Vec<(usize, usize)> = Vec::with_capacity(max_col-min_col);
        for col in min_col..max_col+1 {
            if board.get_val(row, col) != EMPTY_VALUE && !((row != 0 && board.get_val(row-1, col) != EMPTY_VALUE) || (row != BOARD_SIZE-1 && board.get_val(row+1, col) != EMPTY_VALUE)) {
                current_word_part.push((row, col));
            }
            else if board.get_val(row, col) == EMPTY_VALUE && current_word_part.len() > 0 {
                let new_min_max = get_new_min_max(board, min_col, max_col, min_row, max_row, &current_word_part);
                removable.push((current_word_part.clone(), new_min_max.0, new_min_max.1, new_min_max.2, new_min_max.3));
                current_word_part.clear();
            }
            else {
                // There's a non-empty value that didn't get removed (so there's still something on the board)
                board_empty = false;
            }
        }
        if current_word_part.len() > 0 {
            let new_min_max = get_new_min_max(board, min_col, max_col, min_row, max_row, &current_word_part);
            removable.push((current_word_part.clone(), new_min_max.0, new_min_max.1, new_min_max.2, new_min_max.3));
            current_word_part.clear();
        }
    }
    // If the board is empty, that means there was only a single horizontal word that got removed
    if removable.len() == 1 && board_empty {
        return Vec::new();
    }
    // Then get vertical removable word parts
    for col in min_col..max_col+1 {
        let mut current_word_part: Vec<(usize, usize)> = Vec::with_capacity(max_col-min_col);
        for row in min_row..max_row+1 {
            if board.get_val(row, col) != EMPTY_VALUE && !((col != 0 && board.get_val(row, col-1) != EMPTY_VALUE) || (col != BOARD_SIZE-1 && board.get_val(row, col+1) != EMPTY_VALUE)) {
                current_word_part.push((row, col));
            }
            else if board.get_val(row, col) == EMPTY_VALUE {
                let new_min_max = get_new_min_max(board, min_col, max_col, min_row, max_row, &current_word_part);
                removable.push((current_word_part.clone(), new_min_max.0, new_min_max.1, new_min_max.2, new_min_max.3));
                current_word_part.clear();
            }
            else {
                board_empty = false;
            }
        }
        if current_word_part.len() > 0 {
            let new_min_max = get_new_min_max(board, min_col, max_col, min_row, max_row, &current_word_part);
            removable.push((current_word_part.clone(), new_min_max.0, new_min_max.1, new_min_max.2, new_min_max.3));
            current_word_part.clear();
        }
    }
    if board_empty {
        return Vec::new();
    }
    return removable;
}

/// Checks whether a `word` can be made using the given `letters`
/// # Arguments
/// * `word` - The vector form of the word to check
/// * `letters` - Length-26 array of the number of each letter in the hand
/// # Returns
/// * `bool` - Whether `word` can be made using `letters`
fn is_makeable(word: &Word, letters: &Letters) -> bool {
    let mut available_letters = letters.clone();
    for letter in word.iter() {
        if available_letters.get(*letter).unwrap() == &0 {
            return false;
        }
        let elem = available_letters.get_mut(*letter).unwrap();
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
    // Find the furtherest left column that the new play is connected to
    let mut minimum_col = start_col;
    while minimum_col > min_col {
        if board.get_val(row, minimum_col) == EMPTY_VALUE {
            minimum_col += 1;
            break;
        }
        minimum_col -= 1;
    }
    minimum_col = cmp::max(minimum_col, min_col);
    // Check across the row where the word was played
    for col_idx in minimum_col..max_col+1 {
        // If we're not at an empty square, add it to the current word we're looking at
        if board.get_val(row, col_idx) != EMPTY_VALUE {
            current_letters.push(board.get_val(row, col_idx));
        }
        else {
            // Turns out that checking with a set is faster than using a trie, at least for smaller hands
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
        // Find the furtherest up row that the word is connected to
        let mut minimum_row = row;
        while minimum_row > min_row {
            if board.get_val(minimum_row, col_idx) == EMPTY_VALUE {
                minimum_row += 1;
                break;
            }
            minimum_row -= 1;
        }
        minimum_row = cmp::max(minimum_row, min_row);
        for row_idx in minimum_row..max_row+1 {
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
    // Find the furtherest up row that the new play is connected to
    let mut minimum_row = start_row;
    while minimum_row > min_row {
        if board.get_val(minimum_row, col) == EMPTY_VALUE {
            minimum_row += 1;
            break;
        }
        minimum_row -= 1;
    }
    minimum_row = cmp::max(minimum_row, min_row);
    // Check down the column where the word was played
    for row_idx in minimum_row..max_row+1 {
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
        // Find the furtherest left column that the word is connected to
        let mut minimum_col = col;
        while minimum_col > min_col {
            if board.get_val(row_idx, minimum_col) == EMPTY_VALUE {
                minimum_col += 1;
                break;
            }
            minimum_col -= 1;
        }
        minimum_col = cmp::max(minimum_col, min_col);
        for col_idx in minimum_col..max_col+1 {
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
#[derive(Copy, Clone, PartialEq)]
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
/// * `letters` - The number of each letter currently in the hand
/// * `letters_on_board` - The number of each letter on the board (is modified in-place)
/// # Returns
/// *`Result` with:*
/// * `bool` - Whether the word could be validly played
/// * `Vec<(usize, usize)>` - Vector of the indices played in `board`
/// * `[usize; 26]`- The remaining letters
/// * `LetterUsage` - How many letters were used
/// 
/// *or empty `Err` if out-of-bounds*
fn play_word(word: &Word, row_idx: usize, col_idx: usize, board: &mut Board, direction: Direction, letters: &Letters, letters_on_board: &mut Letters) -> Result<(bool, Vec<(usize, usize)>, [usize; 26], LetterUsage), ()> {
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
            else {
                let mut entirely_overlaps = true;
                for i in 0..word.len() {
                    if board.get_val(row_idx, col_idx+i) == EMPTY_VALUE {
                        board.set_val(row_idx, col_idx+i, word[i]);
                        letters_on_board[word[i]] += 1;
                        played_indices.push((row_idx, col_idx+i));
                        entirely_overlaps = false;
                        let elem = remaining_letters.get_mut(word[i]).unwrap();
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
            else {
                let mut entirely_overlaps = true;
                for i in 0..word.len() {
                    if board.get_val(row_idx+i, col_idx) == EMPTY_VALUE {
                        board.set_val(row_idx+i, col_idx, word[i]);
                        letters_on_board[word[i]] += 1;
                        played_indices.push((row_idx+i, col_idx));
                        entirely_overlaps = false;
                        let elem = remaining_letters.get_mut(word[i]).unwrap();
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

/// Removes words that can't be played with `current_letters` plus a set number of `board_letters`
/// # Arguments
/// * `current_letters` - Letters currently available in the hand
/// * `board_letters` - Letters played on the board
/// * `word_being_checked` - Word to check if it contains the appropriate number of letters
/// * `filter_letters_on_board` - Maximum number of letters from `board_letters` that can be used when checking if the word can be played
/// # Returns
/// * `bool` - Whether `word_being_checked` should pass the filter
fn check_filter_after_play_later(mut current_letters: Letters, mut board_letters: Letters, word_being_checked: &Word, filter_letters_on_board: usize) -> bool {
    let mut num_from_board = 0usize;
    for letter in word_being_checked.iter() {
        let num_in_hand = current_letters.get_mut(*letter).unwrap();
        if *num_in_hand == 0 {
            if num_from_board == filter_letters_on_board {
                return false;
            }
            let num_on_board = board_letters.get_mut(*letter).unwrap();
            if *num_on_board == 0 {
                return false;
            }
            *num_on_board -= 1;
            num_from_board += 1;
        }
        else {
            *num_in_hand -= 1;
        }
    }
    return true;
}

/// Undoes a play on the `board`
/// # Arguments
/// * `board` - `Board` being undone (is modified in-place)
/// * `played_indices` - Vector of the indices in `board` that need to be reset
/// * `letters_on_board` - Length-26 array of the number of each letter on the board (is modified in place)
fn undo_play(board: &mut Board, played_indices: &Vec<(usize, usize)>, letters_on_board: &mut Letters) {
    for index in played_indices.iter() {
        letters_on_board[board.get_val(index.0, index.1)] -= 1;
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
fn check_filter_after_play(mut letters: Letters, word_being_checked: &Word, played_on_board: &HashSet<&usize>) -> bool {
    let mut already_seen_negative = false;
    for letter in word_being_checked.iter() {
        let elem = letters.get_mut(*letter).unwrap();
        if *elem == 0 && !played_on_board.contains(letter) {
            return false;
        }
        else if *elem <= 0 && already_seen_negative {
            return false;
        }
        else if *elem == 0 {
            already_seen_negative = true;
        }
        else {
            *elem -= 1;
        }
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
/// * `words_checked` - The number of words checked in total
/// * `letters_on_board` - Length-26 array of the number of each letter currently present on the `board`
/// * `filter_letters_on_board` - Maximum number of letters currently on the board that can be used in a newly played word
/// * `max_words_to_check` - Maximum number of words to check before stopping
/// * `stop_t` - `AtomicBool` that, when set, indicates that processing should stop
/// # Returns
/// *`Result` with:*
/// * `bool` - Whether the word could be validly played
/// * `usize` - Minimum occupied column index in `board`
/// * `usize` - Maximum occupied column index in `board`
/// * `usize` - Minimum occupied row index in `board`
/// * `usize` - Maximum occupied row index in `board`
/// 
/// *or empty `Err` on if out-of-bounds, past the maximum number of words to check, or another thread signalled to stop*
fn play_further(board: &mut Board, min_col: usize, max_col: usize, min_row: usize, max_row: usize, valid_words_vec: Vec<&Word>, valid_words_set: &HashSet<Word>, letters: Letters, depth: usize, words_checked: &mut usize, letters_on_board: &mut Letters, filter_letters_on_board: usize, max_words_to_check: usize, stop_t: &Arc<AtomicBool>) -> Result<(bool, usize, usize, usize, usize), ()> {
    if *words_checked > max_words_to_check || stop_t.load(Ordering::Relaxed) {
        return Err(());
    }
    // If we're at an odd depth, play horizontally first (trying to alternate horizontal-vertical-horizontal as a heuristic to solve faster)
    if depth % 2 == 1 {
        for word in valid_words_vec.iter() {
            *words_checked += 1;
            if stop_t.load(Ordering::Relaxed) {
                return Err(());
            }
            // Try across all rows (starting from one before to one after)
            for row_idx in min_row-1..max_row+2 {
                // For each row, try across all columns (starting from the farthest out the word could be played)
                for col_idx in min_col-word.len()..max_col+2 {
                    // Using the ? because `play_word` can give an `Err` if the index is out of bounds
                    let res = play_word(word, row_idx, col_idx, board, Direction::Horizontal, &letters, letters_on_board)?;
                    if res.0 {
                        // If the word was played successfully (i.e. it's not a complete overlap and it borders at least one existing tile), then check the validity of the new words it forms
                        let new_min_col = cmp::min(min_col, col_idx);
                        let new_max_col = cmp::max(max_col, col_idx+word.len());
                        let new_min_row = cmp::min(min_row, row_idx);
                        let new_max_row = cmp::max(max_row, row_idx);
                        if is_board_valid_horizontal(board, new_min_col, new_max_col, new_min_row, new_max_row, row_idx, col_idx, col_idx+word.len()-1, valid_words_set) {
                            // If it's valid, go to the next recursive level (unless we've all the letters, at which point we're done)
                            match res.3 {
                                LetterUsage::Finished => {
                                    return Ok((true, new_min_col, new_max_col, new_min_row, new_max_row));
                                },
                                LetterUsage::Remaining => {
                                    // Another option: let new_valid_words_vec: Vec<&Word> = valid_words_vec.clone().into_iter().filter(|w| check_filter_after_play_later(letters.clone(), letters_on_board.clone(), w)).collect();
                                    // I think doing it that way might be less efficient however due to the `clone` of `valid_words_vec`
                                    let mut new_valid_words_vec: Vec<&Word> = Vec::with_capacity(valid_words_vec.len()/2);
                                    for i in 0..valid_words_vec.len() {
                                        if check_filter_after_play_later(letters.clone(), letters_on_board.clone(), valid_words_vec[i], filter_letters_on_board) {
                                            new_valid_words_vec.push(valid_words_vec[i]);
                                        }
                                    }
                                    let res2 = play_further(board, new_min_col, new_max_col, new_min_row, new_max_row, new_valid_words_vec, valid_words_set, res.2, depth+1, words_checked, letters_on_board, filter_letters_on_board, max_words_to_check, stop_t)?;
                                    if res2.0 {
                                        // If that recursive stack finishes successfully, we're done! (could have used another Result or Option rather than a bool in the returned tuple, but oh well)
                                        return Ok(res2);
                                    }
                                    else {
                                        // Otherwise, undo the previous play (cloning the board before each play so we don't have to undo is *way* slower)
                                        undo_play(board, &res.1, letters_on_board);
                                    }
                                },
                                LetterUsage::Overused => unreachable!()
                            }
                        }
                        else {
                            // If the play formed some invalid words, undo the previous play
                            undo_play(board, &res.1, letters_on_board);
                        }
                    }
                    else {
                        // If trying to play the board was invalid, undo the play
                        undo_play(board, &res.1, letters_on_board);
                    }
                }
            }
        }
        // If trying every word horizontally didn't work, try vertically instead
        for word in valid_words_vec.iter() {
            *words_checked += 1;
            if stop_t.load(Ordering::Relaxed) {
                return Err(());
            }
            // Try down all columns
            for col_idx in min_col-1..max_col+2 {
                // This is analgous to the above
                for row_idx in min_row-word.len()..max_row+2 {
                    let res = play_word(word, row_idx, col_idx, board, Direction::Vertical, &letters, letters_on_board)?;
                    if res.0 {
                        let new_min_col = cmp::min(min_col, col_idx);
                        let new_max_col = cmp::max(max_col, col_idx);
                        let new_min_row = cmp::min(min_row, row_idx);
                        let new_max_row = cmp::max(max_row, row_idx+word.len());
                        if is_board_valid_vertical(board, new_min_col, new_max_col, new_min_row, new_max_row, row_idx, row_idx+word.len()-1, col_idx, valid_words_set) {
                            match res.3 {
                                LetterUsage::Finished => {
                                    return Ok((true, new_min_col, new_max_col, new_min_row, new_max_row));
                                },
                                LetterUsage::Remaining => {
                                    let mut new_valid_words_vec: Vec<&Word> = Vec::with_capacity(valid_words_vec.len()/2);
                                    for i in 0..valid_words_vec.len() {
                                        if check_filter_after_play_later(letters.clone(), letters_on_board.clone(), valid_words_vec[i], filter_letters_on_board) {
                                            new_valid_words_vec.push(valid_words_vec[i]);
                                        }
                                    }
                                    let res2 = play_further(board, new_min_col, new_max_col, new_min_row, new_max_row, new_valid_words_vec, valid_words_set, res.2, depth+1, words_checked, letters_on_board, filter_letters_on_board, max_words_to_check, stop_t)?;
                                    if res2.0 {
                                        return Ok(res2);
                                    }
                                    else {
                                        undo_play(board, &res.1, letters_on_board);
                                    }
                                },
                                LetterUsage::Overused => unreachable!()
                            }
                        }
                        else {
                            undo_play(board, &res.1, letters_on_board);
                        }
                    }
                    else {
                        undo_play(board, &res.1, letters_on_board);
                    }
                }
            }
        }
        return Ok((false, min_col, max_col, min_row, max_row));
    }
    // If we're at an even depth, play vertically first. Otherwise this is analgous to the above.
    else {
        for word in valid_words_vec.iter() {
            *words_checked += 1;
            if stop_t.load(Ordering::Relaxed) {
                return Err(());
            }
            // Try down all columns
            for col_idx in min_col-1..max_col+2 {
                for row_idx in min_row-word.len()..max_row+2 {
                    let res = play_word(word, row_idx, col_idx, board, Direction::Vertical, &letters, letters_on_board)?;
                    if res.0 {
                        let new_min_col = cmp::min(min_col, col_idx);
                        let new_max_col = cmp::max(max_col, col_idx);
                        let new_min_row = cmp::min(min_row, row_idx);
                        let new_max_row = cmp::max(max_row, row_idx+word.len());
                        if is_board_valid_vertical(board, new_min_col, new_max_col, new_min_row, new_max_row, row_idx, row_idx+word.len()-1, col_idx, valid_words_set) {
                            match res.3 {
                                LetterUsage::Finished => {
                                    return Ok((true, new_min_col, new_max_col, new_min_row, new_max_row));
                                },
                                LetterUsage::Remaining => {
                                    let mut new_valid_words_vec: Vec<&Word> = Vec::with_capacity(valid_words_vec.len()/2);
                                    for i in 0..valid_words_vec.len() {
                                        if check_filter_after_play_later(letters.clone(), letters_on_board.clone(), valid_words_vec[i], filter_letters_on_board) {
                                            new_valid_words_vec.push(valid_words_vec[i]);
                                        }
                                    }
                                    let res2 = play_further(board, new_min_col, new_max_col, new_min_row, new_max_row, new_valid_words_vec, valid_words_set, res.2, depth+1, words_checked, letters_on_board, filter_letters_on_board, max_words_to_check, stop_t)?;
                                    if res2.0 {
                                        return Ok(res2);
                                    }
                                    else {
                                        undo_play(board, &res.1, letters_on_board);
                                    }
                                },
                                LetterUsage::Overused => unreachable!()
                            }
                        }
                        else {
                            undo_play(board, &res.1, letters_on_board);
                        }
                    }
                    else {
                        undo_play(board, &res.1, letters_on_board);
                    }
                }
            }
        }
        // No point in checking horizontally for the first depth, since it would have to form a vertical word that was already checked and failed
        if depth == 0 {
            return Ok((false, min_col, max_col, min_row, max_row));
        }
        for word in valid_words_vec.iter() {
            *words_checked += 1;
            if stop_t.load(Ordering::Relaxed) {
                return Err(());
            }
            // Try across all rows
            for row_idx in min_row-1..max_row+2 {
                for col_idx in min_col-word.len()..max_col+2 {
                    let res = play_word(word, row_idx, col_idx, board, Direction::Horizontal, &letters, letters_on_board)?;
                    if res.0 {
                        let new_min_col = cmp::min(min_col, col_idx);
                        let new_max_col = cmp::max(max_col, col_idx+word.len());
                        let new_min_row = cmp::min(min_row, row_idx);
                        let new_max_row = cmp::max(max_row, row_idx);
                        if is_board_valid_horizontal(board, new_min_col, new_max_col, new_min_row, new_max_row, row_idx, col_idx, col_idx+word.len()-1, valid_words_set) {
                            match res.3 {
                                LetterUsage::Finished => {
                                    return Ok((true, new_min_col, new_max_col, new_min_row, new_max_row));
                                },
                                LetterUsage::Remaining => {
                                    let mut new_valid_words_vec: Vec<&Word> = Vec::with_capacity(valid_words_vec.len()/2);
                                    for i in 0..valid_words_vec.len() {
                                        if check_filter_after_play_later(letters.clone(), letters_on_board.clone(), valid_words_vec[i], filter_letters_on_board) {
                                            new_valid_words_vec.push(valid_words_vec[i]);
                                        }
                                    }
                                    let res2 = play_further(board, new_min_col, new_max_col, new_min_row, new_max_row, new_valid_words_vec, valid_words_set, res.2, depth+1, words_checked, letters_on_board, filter_letters_on_board, max_words_to_check, stop_t)?;
                                    if res2.0 {
                                        return Ok(res2);
                                    }
                                    else {
                                        undo_play(board, &res.1, letters_on_board);
                                    }
                                },
                                LetterUsage::Overused => unreachable!()
                            }
                        }
                        else {
                            undo_play(board, &res.1, letters_on_board);
                        }
                    }
                    else {
                        undo_play(board, &res.1, letters_on_board);
                    }
                }
            }
        }
        return Ok((false, min_col, max_col, min_row, max_row));
    }
}

/// Tries to play a single letter on the board
/// # Arguments
/// * `board` - The `Board` on which to try to play the `letter`
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

/// Plays a new hand of `letters` on an existing `board`
/// # Arguments
/// * `board` - Previous board solution
/// * `min_col` - Minimum occupied column index
/// * `max_col` - Maximum occupied column index
/// * `min_row` - Minimum occupied row index
/// * `max_row` - Maximum occupied row index
/// * `letters` - Letters in the new hand
fn play_existing(board: &Board, min_col: usize, max_col: usize, min_row: usize, max_row: usize, letters: &Letters) {

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
    board: Vec<Vec<String>>,
    /// How long it took to solve the board in milliseconds
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
    letters: Letters
}

/// Controls the state of the app
struct AppState {
    /// Dictionary of the ~20k most common words in English
    all_words_short: Vec<Word>,
    /// Complete Scrabble dictionary
    all_words_long: Vec<Word>,
    /// The last game state (if `None`, then no previous game has been played)
    last_game: Mutex<Option<GameState>>,
    /// Number of letters present on the board that can be used in a word (higher will result in fewer words being filtered out)
    filter_letters_on_board: usize,
    /// Maximum number of words to check before stopping
    maximum_words_to_check: usize
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
        // Then selecte `how_many` characters from that vector
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
/// * `available_letters` - `HashMap` (from JavaScript object) mapping string letters to numeric quantity of each letter
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
    let playable_short: Vec<String> = state.all_words_short.iter().filter(|word| is_makeable(word, &letters)).map(convert_array_to_word).collect();
    let playable_long: Vec<String> = state.all_words_long.iter().filter(|word| is_makeable(word, &letters)).map( convert_array_to_word).collect();
    return Ok(PlayableWords { short: playable_short, long: playable_long });
}

/// Updates the maximum number of letters a word can use from the board; higher values will eliminate fewer words (values above the board size will give an exhausize check)
/// # Arguments
/// * `max_letters` - The maximum number of letters than can be used from the board
/// # Returns
/// Empty `Result` upon success
/// 
/// *or String `Err` upon failure*
#[tauri::command]
async fn set_max_letters_from_board(max_letters: i64, state: State<'_, AppState>) -> Result<(), String> {
    if max_letters < 0 {
        return Err("The maximum letters than can be used from the board must be greater than or equal to 0!".to_owned());
    }
    state.filter_letters_on_board = max_letters as usize;
    Ok(())
}

/// Updates the maximum number of letters a word can use from the board; higher values will eliminate fewer words (values above the board size will give an exhausize check)
/// # Arguments
/// * `max_words` - The maximum number of words than can be used from the board
/// # Returns
/// Empty `Result` upon success
/// 
/// *or String `Err` upon failure*
#[tauri::command]
async fn set_max_words_to_check(max_words: i64, state: State<'_, AppState>) -> Result<(), String> {
    if max_letters < 0 {
        return Err("The maximum letters than can be used from the board must be greater than or equal to 0!".to_owned());
    }
    state.filter_letters_on_board = max_letters as usize;
    Ok(())
}

/// Async command executed by the frontend to reset the Banangrams board
/// # Arguments
/// * `state` - Current state of the app
/// # Returns
/// Empty `Result` upon success
/// 
/// *or String `Err` upon failure*
#[tauri::command]
async fn reset(state: State<'_, AppState>) -> Result<(), String> {
    let mut last_game_state = state.last_game.lock().or(Err("Failed to get lock on the last game state"))?;
    *last_game_state = None;
    Ok(())
}

/// Async command executed by the frontend to solve a Bananagrams board
/// # Arguments
/// * `available_letters` - `HashMap` (from JavaScript object) mapping string letters to numeric quantity of each letter
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
    let mut previous_board: Option<BoardAndIdxs> = None;
    match &*last_game_state {   // I don't like &*
        Some(prev_state) => {
            previous_board = Some((prev_state.board.clone(), prev_state.min_col, prev_state.max_col, prev_state.min_row, prev_state.max_row));
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
                    return Ok(Solution { board: board_to_vec(&prev_state.board, prev_state.min_col, prev_state.max_col, prev_state.min_row, prev_state.max_row, &HashSet::new()), elapsed: now.elapsed().as_millis() });
                },
                LetterComparison::GreaterByOne => {
                    // If only a single letter has increased by one, then first check just that letter
                    let valid_words_vec: Vec<Word> = state.all_words_short.iter().filter(|word| is_makeable(word, &letters)).map(|word| word.clone()).collect();
                    let valid_words_set: HashSet<Word> = HashSet::from_iter(valid_words_vec.clone());
                    let mut board = prev_state.board.clone();
                    let res = play_one_letter(&mut board, prev_state.min_col, prev_state.max_col, prev_state.min_row, prev_state.max_row, seen_greater, &valid_words_set);
                    match res {
                        Some(result) => {
                            let previous_idxs = get_board_overlap(&prev_state.board, &board, prev_state.min_col, prev_state.max_col, prev_state.min_row, prev_state.max_row, result.2, result.3, result.4, result.5);
                            *last_game_state = Some(GameState { board: board.clone(), min_col: result.2, max_col: result.3, min_row: result.4, max_row: result.5, letters });
                            return Ok(Solution { board: board_to_vec(&board, result.2, result.3, result.4, result.5, &previous_idxs), elapsed: now.elapsed().as_millis() });
                        },
                        None => {
                            // If we failed when playing one letter, try playing off the existing board
                            let attempt = play_existing(&prev_state.board, prev_state.min_col, prev_state.max_col, prev_state.min_row, prev_state.max_row, &letters);
                            match attempt {
                                Some(result) => {
                                    let previous_idxs = get_board_overlap(&prev_state.board, &result.0, prev_state.min_col, prev_state.max_col, prev_state.min_row, prev_state.max_row, result.1, result.2, result.3, result.4);
                                    *last_game_state = Some(GameState { board: result.0.clone(), min_col: result.1, max_col: result.2, min_row: result.3, max_row: result.4, letters });
                                    return Ok(Solution { board: board_to_vec(&result.0, result.1, result.2, result.3, result.4, &previous_idxs), elapsed: now.elapsed().as_millis() });
                                },
                                None => { return Err("No solution found - dump and try again!".to_owned()); }
                            }
                        }
                    }
                },
                LetterComparison::GreaterByMoreThanOne => {
                    // If a letter has increased by more than one, or multiple have increased by one or more, then try playing off the existing board
                    let attempt = play_existing(&prev_state.board, prev_state.min_col, prev_state.max_col, prev_state.min_row, prev_state.max_row, &letters);
                    match attempt {
                        Some(result) => {
                            let previous_idxs = get_board_overlap(&prev_state.board, &result.0, prev_state.min_col, prev_state.max_col, prev_state.min_row, prev_state.max_row, result.1, result.2, result.3, result.4);
                            *last_game_state = Some(GameState { board: result.0.clone(), min_col: result.1, max_col: result.2, min_row: result.3, max_row: result.4, letters });
                            return Ok(Solution { board: board_to_vec(&result.0, result.1, result.2, result.3, result.4, &previous_idxs), elapsed: now.elapsed().as_millis() });
                        },
                        None => { return Err("No solution found - dump and try again!".to_owned()); }
                    }
                },
                LetterComparison::SomeLess => {/* We just want to continue to the code that starts from scratch */}
            }
        },
        None => {/* We just want to continue to the code that starts from scratch */}
    }
    // Play from scratch
    // Get a vector of all valid words
    let valid_words_vec: Vec<Word> = state.all_words_short.iter().filter(|word| is_makeable(word, &letters)).map(|word| word.clone()).collect();
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
    let char_vec: Vec<(Vec<Vec<String>>, Board, usize, usize, usize, usize)> = Vec::new();
    let ret_val = Arc::new(Mutex::new(char_vec));
    let max_words_to_check = state.maximum_words_to_check;
    let filter_letters_on_board = state.filter_letters_on_board;
    // For each thread (i.e. piece of available parallelism), spawn a new thread to check those words
    // These threads check different sets of initial words in the board, and whichever finishes first signals the others to stop
    for chunk in chunks.into_iter() {
        let stop_t = stop.clone();
        let new_letters = letters.clone();
        let copied_new_valid_words_vec = valid_words_vec.clone();
        let conn = ret_val.clone();
        let cloned_previous_board = previous_board.clone();
        let handle = thread::spawn(move || {
            let mut tried_words: HashSet<&Word> = HashSet::new();
            // Loop through each word and play it on a new board
            let mut words_checked = 0;
            for word in chunk.iter() {
                let mut board = Board::new();
                let col_start = BOARD_SIZE/2 - word.len()/2;
                let row = BOARD_SIZE/2;
                let mut use_letters: [usize; 26] = new_letters.clone();
                let mut letters_on_board = [0usize; 26];
                for i in 0..word.len() {
                    board.set_val(row, col_start+i, word[i]);
                    letters_on_board[word[i]] += 1;
                    use_letters[word[i]] -= 1;  // Should never underflow because we've verified that every word is playable with these letters
                }
                let min_col = col_start;
                let min_row = row;
                let max_col = col_start + (word.len()-1);
                let max_row = row;
                if use_letters.iter().all(|count| *count == 0) {
                    if !stop_t.load(Ordering::Relaxed) {
                        stop_t.store(true, Ordering::Relaxed);
                        let previous_idxs: HashSet<(usize, usize)>;
                        match cloned_previous_board {
                            Some(prev) => {
                                previous_idxs = get_board_overlap(&prev.0, &board, prev.1, prev.2, prev.3, prev.4, min_col, max_col, min_row, max_row);
                            },
                            None => {previous_idxs = HashSet::new();}
                        }
                        // The expect may panic the thread but I think that's ok
                        let mut ret = conn.lock().expect("Failed to get lock on shared ret_val");
                        ret.push((board_to_vec(&board, min_col, max_col, min_row, max_row, &HashSet::new()), board.clone(), min_col, max_col, min_row, max_row));
                        break;
                    }
                }
                else {
                    // Reduce the set of remaining words to check to those that can be played with the letters not in the first word (plus only one of the tiles played in the first word)
                    let word_letters: HashSet<&usize> = HashSet::from_iter(word.iter());
                    let mut new_valid_words_vec: Vec<&Word> = Vec::with_capacity(copied_new_valid_words_vec.len()-tried_words.len());
                    for w in copied_new_valid_words_vec.iter() {
                        if !tried_words.contains(w) {
                            new_valid_words_vec.push(w);
                        }
                    }
                    let valid_words_set: HashSet<Word> = HashSet::from_iter(new_valid_words_vec.clone().into_iter().cloned());
                    // Begin the recursive processing
                    let result = play_further(&mut board, min_col, max_col, min_row, max_row, new_valid_words_vec, &valid_words_set, use_letters, 0, &mut words_checked, &mut letters_on_board, filter_letters_on_board, max_words_to_check, &stop_t);
                    match result {
                        // If the result was good, then store it and signal other threads to finish (so long as another thread isn't doing so)
                        Ok(res) => {
                            if res.0 && !stop_t.load(Ordering::Relaxed) {
                                stop_t.store(true, Ordering::Relaxed);
                                // The expect will panic the thread but I think that's ok
                                let mut ret = conn.lock().expect("Failed to get lock on shared ret_val");
                                let previous_idxs: HashSet<(usize, usize)>;
                                match cloned_previous_board {
                                    Some(prev) => {
                                        previous_idxs = get_board_overlap(&prev.0, &board, prev.1, prev.2, prev.3, prev.4, res.1, res.2, res.3, res.4);
                                    },
                                    None => {previous_idxs = HashSet::new();}
                                }
                                ret.push((board_to_vec(&board, res.1, res.2, res.3, res.4, &previous_idxs), board.clone(), res.1, res.2, res.3, res.4));
                                break;
                            }
                            else {
                                tried_words.insert(word);
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
    let ret: std::sync::MutexGuard<'_, Vec<(Vec<Vec<String>>, Board, usize, usize, usize, usize)>>;
    match ret_val.lock() {
        Ok(locked) => {
            ret = locked;
        },
        _ => {
            return Err("Failed to get lock on shared ret_val when checking return".to_owned());
        }
    }
    if ret.len() > 0 {
        *last_game_state = Some(GameState { board: ret[0].1.clone(), min_col: ret[0].2, max_col: ret[0].3, min_row: ret[0].4, max_row: ret[0].5, letters });
        return Ok(Solution { board: ret[0].0.clone(), elapsed: now.elapsed().as_millis() });
    }
    // Try again with all words in the Scrabble dictionary; this could potentially take much longer but is done just to be exhaustive
    let valid_words_vec: Vec<Word> = state.all_words_long.iter().filter(|word| is_makeable(word, &letters)).map(|word| word.clone()).collect();
    if valid_words_vec.len() == 0 {
        return Err("No valid words can be formed from the current letters - dump and try again!".to_owned());
    }
    // Split the words to check up into appropriate chunks based on the available parallelism
    let chunks: Vec<Vec<Word>> = valid_words_vec.chunks(chunk_size.ceil() as usize).map(|words| words.to_vec()).collect();
    let mut handles: Vec<thread::JoinHandle<()>> = Vec::with_capacity(chunks.len());
    let char_vec: Vec<(Vec<Vec<String>>, Board, usize, usize, usize, usize)> = Vec::new();
    let ret_val_long = Arc::new(Mutex::new(char_vec));
    // For each thread (i.e. piece of available parallelism), spawn a new thread to check those words
    // These threads check different sets of initial words in the board, and whichever finishes first signals the others to stop
    for chunk in chunks.into_iter() {
        let stop_t = stop.clone();
        let new_letters = letters.clone();
        let copied_new_valid_words_vec = valid_words_vec.clone();
        let conn = ret_val_long.clone();
        let cloned_previous_board = previous_board.clone();
        let handle = thread::spawn(move || {
            let mut tried_words: HashSet<&Word> = HashSet::new();
            // Loop through each word and play it on a new board
            let mut words_checked = 0;
            for word in chunk.iter() {
                let mut board = Board::new();
                let col_start = BOARD_SIZE/2 - word.len()/2;
                let row = BOARD_SIZE/2;
                let mut use_letters: [usize; 26] = new_letters.clone();
                let mut letters_on_board = [0usize; 26];
                for i in 0..word.len() {
                    board.set_val(row, col_start+i, word[i]);
                    letters_on_board[word[i]] += 1;
                    use_letters[word[i]] -= 1;  // Should never underflow because we've verified that every word is playable with these letters
                }
                let min_col = col_start;
                let min_row = row;
                let max_col = col_start + (word.len()-1);
                let max_row = row;
                if use_letters.iter().all(|count| *count == 0) {
                    if !stop_t.load(Ordering::Relaxed) {
                        stop_t.store(true, Ordering::Relaxed);
                        let previous_idxs: HashSet<(usize, usize)>;
                        match cloned_previous_board {
                            Some(prev) => {
                                previous_idxs = get_board_overlap(&prev.0, &board, prev.1, prev.2, prev.3, prev.4, min_col, max_col, min_row, max_row);
                            },
                            None => {previous_idxs = HashSet::new();}
                        }
                        // The expect may panic the thread but I think that's ok
                        let mut ret = conn.lock().expect("Failed to get lock on shared ret_val");
                        ret.push((board_to_vec(&board, min_col, max_col, min_row, max_row, &HashSet::new()), board.clone(), min_col, max_col, min_row, max_row));
                        break;
                    }
                }
                else {
                    // Reduce the set of remaining words to check to those that can be played with the letters not in the first word (plus only one of the tiles played in the first word)
                    let word_letters: HashSet<&usize> = HashSet::from_iter(word.iter());
                    let mut new_valid_words_vec: Vec<&Word> = Vec::with_capacity(copied_new_valid_words_vec.len()-tried_words.len());
                    for w in copied_new_valid_words_vec.iter() {
                        if !tried_words.contains(w) {
                            new_valid_words_vec.push(w);
                        }
                    }
                    let valid_words_set: HashSet<Word> = HashSet::from_iter(new_valid_words_vec.clone().into_iter().cloned());
                    // Begin the recursive processing
                    let result = play_further(&mut board, min_col, max_col, min_row, max_row, new_valid_words_vec, &valid_words_set, use_letters, 0, &mut words_checked, &mut letters_on_board, filter_letters_on_board, max_words_to_check, &stop_t);
                    match result {
                        // If the result was good, then store it and signal other threads to finish (so long as another thread isn't doing so)
                        Ok(res) => {
                            if res.0 && !stop_t.load(Ordering::Relaxed) {
                                stop_t.store(true, Ordering::Relaxed);
                                // The expect will panic the thread but I think that's ok
                                let mut ret = conn.lock().expect("Failed to get lock on shared ret_val");
                                let previous_idxs: HashSet<(usize, usize)>;
                                match cloned_previous_board {
                                    Some(prev) => {
                                        previous_idxs = get_board_overlap(&prev.0, &board, prev.1, prev.2, prev.3, prev.4, res.1, res.2, res.3, res.4);
                                    },
                                    None => {previous_idxs = HashSet::new();}
                                }
                                ret.push((board_to_vec(&board, res.1, res.2, res.3, res.4, &previous_idxs), board.clone(), res.1, res.2, res.3, res.4));
                                break;
                            }
                            else {
                                tried_words.insert(word);
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
    let ret_long: std::sync::MutexGuard<'_, Vec<(Vec<Vec<String>>, Board, usize, usize, usize, usize)>>;
    match ret_val_long.lock() {
        Ok(locked) => {
            ret_long = locked;
        },
        _ => {
            return Err("Failed to get lock on shared ret_val when checking return".to_owned());
        }
    }
    if ret_long.len() > 0 {
        *last_game_state = Some(GameState { board: ret_long[0].1.clone(), min_col: ret_long[0].2, max_col: ret_long[0].3, min_row: ret_long[0].4, max_row: ret_long[0].5, letters });
        return Ok(Solution { board: ret_long[0].0.clone(), elapsed: now.elapsed().as_millis() });
    }
    return Err("No solution found - dump and try again!".to_owned());
}

fn main() {
    let all_words_short: Vec<Word> = include_str!("new_short_dictionary.txt").lines().map(convert_word_to_array).collect();
    let all_words_long: Vec<Word> = include_str!("dictionary.txt").lines().map(convert_word_to_array).collect();
    tauri::Builder::default()
        .manage(AppState { all_words_short, all_words_long, last_game: None.into(), filter_letters_on_board: 2, maximum_words_to_check: 500_000 })
        .invoke_handler(tauri::generate_handler![play_bananagrams, reset, get_playable_words, get_random_letters])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
