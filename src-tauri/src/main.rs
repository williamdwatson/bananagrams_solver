// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::{cmp, fmt, mem, usize, collections::HashMap};
use array2d::Array2D;
use std::collections::HashSet;
use tauri::State;

const MAX_WORD_LENGTH: usize = 15;
const EMPTY_VALUE: usize = 30;
const BOARD_SIZE: usize = 64;
const UPPERCASE: &str = "ABCDEFGHIJKLMNOPQRSTUVWXYZ";

/// Converts a word into a numeric vector representation
/// # Arguments
/// * `word` - String word to convert
/// # Returns
/// `Vec<usize>` - numeric representation of `word`, with each letter converted from 0 ('A') to 25 ('Z')
/// # See also
/// `convert_array_to_word`
fn convert_word_to_array(word: &str) -> Vec<usize> {
    word.chars().filter(|c| c.is_ascii_uppercase()).map(|c| (c as usize) - 65).collect()
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
// fn board_to_string(board: &Array2D<usize>, min_col: usize, max_col: usize, min_row: usize, max_row: usize) -> String {
//     let mut board_string: Vec<char> = Vec::with_capacity((max_row-min_row)*(max_col-min_col));
//     for row in min_row..max_row+1 {
//         for col in min_col..max_col+1 {
//             if board[(row, col)] == EMPTY_VALUE {
//                 board_string.push(' ');
//             }
//             else {
//                 board_string.push((board[(row, col)] as u8+65) as char);
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
fn board_to_vec(board: &Array2D<usize>, min_col: usize, max_col: usize, min_row: usize, max_row: usize) -> Vec<Vec<char>> {
    let mut board_vec: Vec<Vec<char>> = Vec::with_capacity(max_row-min_row);
    for row in min_row..max_row+1 {
        let mut row_vec: Vec<char> = Vec::with_capacity(max_col-min_col);
        for col in min_col..max_col+1 {
            if board[(row, col)] == EMPTY_VALUE {
                row_vec.push(' ');
            }
            else {
                row_vec.push((board[(row, col)] as u8+65) as char);
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
fn is_makeable(word: &Vec<usize>, letters: [usize; 26]) -> bool {
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
/// * `board` - Array2D of the board being checked
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
fn is_board_valid_horizontal(board: &Array2D<usize>, min_col: usize, max_col: usize, min_row: usize, max_row: usize, row: usize, start_col: usize, end_col: usize, valid_words: &HashSet<&Vec<usize>>) -> bool {
    let mut current_letters: Vec<usize> = Vec::with_capacity(MAX_WORD_LENGTH);
    for col_idx in min_col..max_col+1 {
        if board[(row, col_idx)] != EMPTY_VALUE {
            current_letters.push(board[(row, col_idx)]);
        }
        else if current_letters.len() > 1 {
            if valid_words.contains(&current_letters) {
                current_letters.clear();
            }
            else {
                return false;
            }
        }
        else if current_letters.len() == 1 {
            current_letters.clear();
        }
    }
    if current_letters.len() > 1 {
        if valid_words.contains(&current_letters) {
            current_letters.clear();
        }
        else {
            return false;
        }
    }
    for col_idx in start_col..end_col+1 {
        current_letters.clear();
        for row_idx in min_row..max_row+1 {
            if board[(row_idx, col_idx)] != EMPTY_VALUE {
                current_letters.push(board[(row_idx, col_idx)]);
            }
            else if current_letters.len() > 1 {
                if valid_words.contains(&current_letters) {
                    current_letters.clear();
                }
                else {
                    return false;
                }
            }
            else if current_letters.len() == 1 {
                current_letters.clear();
            }
        }
        if current_letters.len() > 1 {
            if !valid_words.contains(&current_letters) {
                return false;
            }
        }
    }
    return true;
}

/// Checks that a `board` is valid after a word is played vertically, given the specified list of `valid_word`s
/// Note that this does not check if all words are contiguous; this condition must be enforced elsewhere.
/// # Arguments
/// * `board` - Array2D of the board being checked
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
fn is_board_valid_vertical(board: &Array2D<usize>, min_col: usize, max_col: usize, min_row: usize, max_row: usize, start_row: usize, end_row: usize, col: usize, valid_words: &HashSet<&Vec<usize>>) -> bool {
    let mut current_letters: Vec<usize> = Vec::with_capacity(MAX_WORD_LENGTH);
    for row_idx in start_row..end_row+1 {
        current_letters.clear();
        for col_idx in min_col..max_col+1 {
            if board[(row_idx, col_idx)] != EMPTY_VALUE {
                current_letters.push(board[(row_idx, col_idx)]);
            }
            else if current_letters.len() > 1 {
                if valid_words.contains(&current_letters) {
                    current_letters.clear();
                }
                else {
                    return false;
                }
            }
            else if current_letters.len() == 1 {
                current_letters.clear();
            }
        }
        if current_letters.len() > 1 {
            if valid_words.contains(&current_letters) {
                current_letters.clear();
            }
            else {
                return false;
            }
        }
    }
    current_letters.clear();
    for row_idx in min_row..max_row+1 {
        if board[(row_idx, col)] != EMPTY_VALUE {
            current_letters.push(board[(row_idx, col)]);
        }
        else if current_letters.len() > 1 {
            if valid_words.contains(&current_letters) {
                current_letters.clear();
            }
            else {
                return false;
            }
        }
        else if current_letters.len() == 1 {
            current_letters.clear();
        }
    }
    if current_letters.len() > 1 {
        if !valid_words.contains(&current_letters) {
            return false;
        }
    }
    return true;
}

/// Enumeration of how many letters have been used. One of:
/// * `Remaining` - there are still unused letters
/// * `Overused` - more letters have been used than are available
/// * `Finished` - all letters have been used
enum LetterUsage {
    Remaining,
    Overused,
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

/// Enumeration of the direction a word is played. One of:
/// * `Horizontal`
/// * `Vertical`
enum Direction {
    Horizontal,
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

/// Checks how many of the available `letters` are used in `board` within the specified bounds
/// # Arguments
/// * `board` - Array2D of the board being checked
/// * `min_col` - Minimum x (column) index of the subsection of the `board` to be checked
/// * `max_col` - Maximum x (column) index of the subsection of the `board` to be checked
/// * `min_row` - Minimum y (row) index of the subsection of the `board` to be checked
/// * `max_row` - Maximum y (row) index of the subsection of the `board` to be checked
/// * `letters` - Length-26 array of the number of letters available
/// # Returns
/// `LetterUsage` enum
fn check_letter_usage(board: &Array2D<usize>, min_col: usize, max_col: usize, min_row: usize, max_row: usize, letters: [usize; 26]) -> LetterUsage {
    let mut remaining_letters = letters.clone();
    for row_idx in min_row..max_row+1 {
        for col_idx in min_col..max_col+1 {
            if board[(row_idx, col_idx)] == EMPTY_VALUE {
                continue;
            }
            let elem = unsafe { remaining_letters.get_unchecked_mut(board[(row_idx, col_idx)]) };
            if elem == &0 {
                return LetterUsage::Overused;
            }
            *elem -= 1;
        }
    }
    if remaining_letters.into_iter().all(|count| count == 0) {
        return LetterUsage::Finished;
    }
    return LetterUsage::Remaining;
}

/// Plays a word on the board
/// # Arguments
/// * `word` - The word to be played
/// * `row_idx` - The starting row at which to play the word
/// * `col_idx` - The starting column at which to play the word
/// * `board` - The current board (will not be modified)
/// * `direction` - The `Direction` in which to play the word
/// # Returns
/// *`Result` with:*
/// * `bool` - Whether the word could be validly played
/// * `Array2D<usize>` - The new board with the `word` played
/// 
/// *or empty `Err` if out-of-bounds*
fn play_word(word: &Vec<usize>, row_idx: usize, col_idx: usize, board: &Array2D<usize>, direction: Direction) -> Result<(bool, Array2D<usize>), ()> {
    let board_size = board.num_columns();   // Should always be square
    match direction {
        Direction::Horizontal => {
            if col_idx + word.len() >= board_size {
                return Err(());
            }
            // Check if the word will start or end at a letter
            let mut valid_loc = (col_idx != 0 && board[(row_idx, col_idx-1)] != EMPTY_VALUE) || (board_size-col_idx <= word.len() && board[(row_idx, col_idx+word.len())] != EMPTY_VALUE);
            // Check if the word will border any letters on the top or bottom
            valid_loc |= (col_idx..col_idx+word.len()).any(|c_idx| (row_idx < board_size-1 && board[(row_idx+1, c_idx)] != EMPTY_VALUE) || (row_idx > 0 && board[(row_idx-1, c_idx)] != EMPTY_VALUE));
            if !valid_loc {
                return Ok((false, board.clone()));
            }
            else{
                let mut new_board = board.clone();
                let mut entirely_overlaps = true;
                for i in 0..word.len() {
                    if new_board[(row_idx, col_idx+i)] == EMPTY_VALUE {
                        new_board[(row_idx, col_idx+i)] = word[i];
                        entirely_overlaps = false;
                    }
                    else if new_board[(row_idx, col_idx+i)] != word[i] {
                        return Ok((false, new_board));
                    }
                }
                return Ok((!entirely_overlaps, new_board));
            }
        },
        Direction::Vertical => {
            if row_idx + word.len() >= board_size {
                return Err(());
            }
            // Check if the word will start or end at a letter
            let mut valid_loc = (row_idx != 0 && board[(row_idx-1, col_idx)] != EMPTY_VALUE) || (board_size-row_idx <= word.len() && board[(row_idx+word.len(), col_idx)] != EMPTY_VALUE);
            // Check if the word will border any letters on the right or left
            valid_loc |= (row_idx..row_idx+word.len()).any(|r_idx| (col_idx < board_size-1 && board[(r_idx, col_idx+1)] != EMPTY_VALUE) || (col_idx > 0 && board[(r_idx, col_idx-1)] != EMPTY_VALUE));
            if !valid_loc {
                return Ok((false, board.clone()));
            }
            else{
                let mut new_board = board.clone();
                let mut entirely_overlaps = true;
                for i in 0..word.len() {
                    if new_board[(row_idx+i, col_idx)] == EMPTY_VALUE {
                        new_board[(row_idx+i, col_idx)] = word[i];
                        entirely_overlaps = false;
                    }
                    else if new_board[(row_idx+i, col_idx)] != word[i] {
                        return Ok((false, new_board));
                    }
                }
                return Ok((!entirely_overlaps, new_board));
            }
        }
    }
}

/// Checks which words can be played after the first
/// # Arguments
/// * `letters` - Length-26 array of originally available letters
/// * `word_being_checked` - Word that is being checked if playable
/// * `previous_word_letters` - Set of the letters of the first word
/// # Returns
/// * `bool` - Whether the `word_being_checked` is playable
fn check_filter_after_play(letters: [usize; 26], word_being_checked: &Vec<usize>, previous_word_letters: &HashSet<&usize>) -> bool {
    let mut available_letters: [isize; 26] = unsafe { mem::transmute(letters) };//letters.into_iter().map(|l| l as isize).collect();
    let mut already_seen_negative = false;
    for letter in word_being_checked.iter() {
        let elem = unsafe { available_letters.get_unchecked_mut(*letter) };
        if elem == &0 && !previous_word_letters.contains(letter) {
            return false;
        }
        else if elem == &-1 && already_seen_negative {
            return false;
        }
        else if elem == &-1 {
            already_seen_negative = true;
        }
        *elem -= 1;
    }
    return true;
}

/// Recursively solves Bananagrams
/// # Arguments
/// * `board` - `Array2D` representation of the board
/// * `min_col` - Minimum occupied column index in `board`
/// * `max_col` - Maximum occupied column index in `board`
/// * `min_row` - Minimum occupied row index in `board`
/// * `max_row` - Maximum occupied row index in `board`
/// * `valid_words_vec` - Vector of vectors, each representing a word (see `convert_word_to_array`)
/// * `valid_words_set` - HashSet of vectors, each representing a word (a HashSet version of `valid_words_vec` for faster membership checking)
/// * `letters` - Length-26 array of the number of each letter in the hand
/// * `depth` - Depth of the current recursive call
/// # Returns
/// *`Result` with:*
/// * `bool` - Whether the word could be validly played
/// * `Array2D<usize>` - The solved board
/// * `usize` - The minimum occupied column index in `board`
/// * `usize` - Maximum occupied column index in `board`
/// * `usize` - Minimum occupied row index in `board`
/// * `usize` - Maximum occupied row index in `board`
/// 
/// *or empty `Err` on if out-of-bounds*
fn play_further(board: Array2D<usize>, min_col: usize, max_col: usize, min_row: usize, max_row: usize, valid_words_vec: &Vec<&Vec<usize>>, valid_words_set: &HashSet<&Vec<usize>>, letters: [usize; 26], depth: usize) -> Result<(bool, Array2D<usize>, usize, usize, usize, usize), ()> {
    match check_letter_usage(&board, min_col, max_col, min_row, max_row, letters) {
        LetterUsage::Overused => Ok((false, board, min_col, max_col, min_row, max_row)),
        LetterUsage::Finished => Ok((true, board, min_col, max_col, min_row, max_row)),
        LetterUsage::Remaining => {
            if depth % 2 == 1 {
                for word in valid_words_vec.iter() {
                    // Try across all rows
                    for row_idx in min_row-1..max_row+2 {
                        for col_idx in min_col-word.len()..max_col+2 {
                            let res = play_word(word, row_idx, col_idx, &board, Direction::Horizontal)?;
                            if res.0 {
                                let new_min_col = cmp::min(min_col, col_idx);
                                let new_max_col = cmp::max(max_col, col_idx+word.len());
                                let new_min_row = cmp::min(min_row, row_idx);
                                let new_max_row = cmp::max(max_row, row_idx);
                                if is_board_valid_horizontal(&res.1, new_min_col, new_max_col, new_min_row, new_max_row, row_idx, col_idx, col_idx+word.len(), valid_words_set) {
                                    // let new_valid_words_vec: Vec<Vec<usize>> = valid_words_vec.iter().filter(|word| check_filter_after_play(letters, word, &res.2)).map(|word| word.clone()).collect();
                                    // let new_valid_words_set: HashSet<Vec<usize>> = HashSet::from_iter(new_valid_words_vec.clone());
                                    let res2 = play_further(res.1, new_min_col, new_max_col, new_min_row, new_max_row, valid_words_vec, valid_words_set, letters, depth+1)?;
                                    if res2.0 {
                                        return Ok(res2);
                                    }
                                }
                            }
                        }
                    }
                }
                for word in valid_words_vec.iter() {
                    // Try down all columns
                    for col_idx in min_col-1..max_col+2 {
                        for row_idx in min_row-word.len()..max_row+2 {
                            let res = play_word(word, row_idx, col_idx, &board, Direction::Vertical)?;
                            if res.0 {
                                let new_min_col = cmp::min(min_col, col_idx);
                                let new_max_col = cmp::max(max_col, col_idx);
                                let new_min_row = cmp::min(min_row, row_idx);
                                let new_max_row = cmp::max(max_row, row_idx+word.len());
                                if is_board_valid_vertical(&res.1, new_min_col, new_max_col, new_min_row, new_max_row, row_idx, row_idx+word.len(), col_idx, &valid_words_set) {
                                    let res2 = play_further(res.1, new_min_col, new_max_col, new_min_row, new_max_row, valid_words_vec, valid_words_set, letters, depth+1)?;
                                    if res2.0 {
                                        return Ok(res2);
                                    }
                                }
                            }
                        }
                    }
                }
                return Ok((false, board, min_col, max_col, min_row, max_row));
            }
            else {
                for word in valid_words_vec.iter() {
                    // Try down all columns
                    for col_idx in min_col-1..max_col+2 {
                        for row_idx in min_row-word.len()..max_row+2 {
                            let res = play_word(word, row_idx, col_idx, &board, Direction::Vertical)?;
                            if res.0 {
                                let new_min_col = cmp::min(min_col, col_idx);
                                let new_max_col = cmp::max(max_col, col_idx);
                                let new_min_row = cmp::min(min_row, row_idx);
                                let new_max_row = cmp::max(max_row, row_idx+word.len());
                                if is_board_valid_vertical(&res.1, new_min_col, new_max_col, new_min_row, new_max_row, row_idx, row_idx+word.len(), col_idx, &valid_words_set) {
                                    // let new_valid_words_vec: Vec<Vec<usize>> = valid_words_vec.iter().filter(|word| check_filter_after_play(letters, word, &res.2)).map(|word| word.clone()).collect();
                                    // let new_valid_words_set: HashSet<Vec<usize>> = HashSet::from_iter(new_valid_words_vec.clone());
                                    let res2 = play_further(res.1, new_min_col, new_max_col, new_min_row, new_max_row, valid_words_vec, valid_words_set, letters, depth+1)?;
                                    if res2.0 {
                                        return Ok(res2);
                                    }
                                }
                            }
                        }
                    }
                }
                for word in valid_words_vec.iter() {
                    // Try across all rows
                    for row_idx in min_row-1..max_row+2 {
                        for col_idx in min_col-word.len()..max_col+2 {
                            let res = play_word(word, row_idx, col_idx, &board, Direction::Horizontal)?;
                            if res.0 {
                                let new_min_col = cmp::min(min_col, col_idx);
                                let new_max_col = cmp::max(max_col, col_idx+word.len());
                                let new_min_row = cmp::min(min_row, row_idx);
                                let new_max_row = cmp::max(max_row, row_idx);
                                if is_board_valid_horizontal(&res.1, new_min_col, new_max_col, new_min_row, new_max_row, row_idx, col_idx, col_idx+word.len(), valid_words_set) {
                                    let res2 = play_further(res.1, new_min_col, new_max_col, new_min_row, new_max_row, valid_words_vec, valid_words_set, letters, depth+1)?;
                                    if res2.0 {
                                        return Ok(res2);
                                    }
                                }
                            }
                        }
                    }
                }
                return Ok((false, board, min_col, max_col, min_row, max_row));
            }
        }
    }
}

struct AllWords {
    all_words_short: Vec<Vec<usize>>,
    all_words_long: Vec<Vec<usize>>
}

// Learn more about Tauri commands at https://tauri.app/v1/guides/features/command
#[tauri::command]
async fn play_bananagrams(available_letters: HashMap<String, usize>, state: State<'_, AllWords>) -> Result<Vec<Vec<char>>, String> {
    let mut letters = [0usize; 26];
    for c in UPPERCASE.chars() {
        let num = available_letters.get(&c.to_string());
        match num {
            Some(number) => {
                letters[(c as usize) - 65] = *number;
            },
            None => {
                return Err(format!("Missing letter: {}", c));
            }
        }
    }
    let mut board_size = BOARD_SIZE;
    let mut restart_because_err: bool;
    let valid_words_vec: Vec<&Vec<usize>> = state.all_words_short.iter().filter(|word| is_makeable(word, letters)).collect();
    loop {
        restart_because_err = false;
        for word in valid_words_vec.iter() {
            let mut board: Array2D<usize> = Array2D::filled_with(EMPTY_VALUE, board_size, board_size);
            let mut new_letters = letters.clone();
            let col_start = board_size/2 - word.len()/2;
            let row = board_size/2;
            for i in 0..word.len() {
                board[(row, col_start+i)] = word[i];
                new_letters[board[(row, col_start+i)]] -= 1;
            }
            let min_col = col_start;
            let min_row = row;
            let max_col = col_start + (word.len()-1);
            let max_row = row;
            let word_letters = HashSet::from_iter(word.iter());
            let new_valid_words_vec: Vec<&Vec<usize>> = valid_words_vec.iter().filter(|word| check_filter_after_play(new_letters, word, &word_letters)).map(|word| word.clone()).collect();
            let new_valid_words_set: HashSet<&Vec<usize>> = HashSet::from_iter(new_valid_words_vec.clone());
            let result = play_further(board, min_col, max_col, min_row, max_row, &new_valid_words_vec, &new_valid_words_set, letters, 0);
            match result {
                Ok(res) => {
                    if res.0 {
                        // println!("Success!");
                        // println!("{}", board_to_string(&res.1, res.2, res.3, res.4, res.5));
                        return Ok(board_to_vec(&res.1, res.2, res.3, res.4, res.5));
                    }
                },
                Err(()) => {
                    // println!("Increasing board size!");
                    board_size *= 2;
                    restart_because_err = true;
                    break;
                }
            }
        }
        if !restart_because_err {
            break;
        }
    }
    // Try again with all words
    let valid_words_vec: Vec<&Vec<usize>> = state.all_words_long.iter().filter(|word| is_makeable(word, letters)).collect();
    loop {
        restart_because_err = false;
        for word in valid_words_vec.iter() {
            let mut board: Array2D<usize> = Array2D::filled_with(EMPTY_VALUE, board_size, board_size);
            let mut new_letters = letters.clone();
            let col_start = board_size/2 - word.len()/2;
            let row = board_size/2;
            for i in 0..word.len() {
                board[(row, col_start+i)] = word[i];
                new_letters[board[(row, col_start+i)]] -= 1;
            }
            let min_col = col_start;
            let min_row = row;
            let max_col = col_start + (word.len()-1);
            let max_row = row;
            let word_letters = HashSet::from_iter(word.iter());
            let new_valid_words_vec: Vec<&Vec<usize>> = valid_words_vec.iter().filter(|word| check_filter_after_play(new_letters, word, &word_letters)).map(|word| word.clone()).collect();
            let new_valid_words_set: HashSet<&Vec<usize>> = HashSet::from_iter(new_valid_words_vec.clone());
            let result = play_further(board, min_col, max_col, min_row, max_row, &new_valid_words_vec, &new_valid_words_set, letters, 0);
            match result {
                Ok(res) => {
                    if res.0 {
                        // println!("Success!");
                        // println!("{}", board_to_string(&res.1, res.2, res.3, res.4, res.5));
                        return Ok(board_to_vec(&res.1, res.2, res.3, res.4, res.5));
                    }
                },
                Err(()) => {
                    // println!("Increasing board size!");
                    board_size *= 2;
                    restart_because_err = true;
                    break;
                }
            }
        }
        if !restart_because_err {
            break;
        }
    }
    Err("No valid words - dumpy and try again!".to_owned())
}

fn main() {
    let all_words_short: Vec<Vec<usize>> = include_str!("C:/Users/willd/Documents/Bananagrams/short_dictionary.txt").lines().map(convert_word_to_array).collect();
    let all_words_long: Vec<Vec<usize>> = include_str!("C:/Users/willd/Documents/Bananagrams/long_dictionary.txt").lines().map(convert_word_to_array).collect();
    tauri::Builder::default()
        .manage(AllWords { all_words_short, all_words_long })
        .invoke_handler(tauri::generate_handler![play_bananagrams])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
