#[macro_use]
extern crate warp;
use core::fmt;
use serde::{Deserialize, Serialize};
use std::{env, fs, rc::Rc, time::Instant};
use trie_rs::{Trie, TrieBuilder};
use warp::Filter;

const SIZE: usize = 5;
const DIRECTIONS: [(i32, i32); 8] = [
    (-1, 0),  // up
    (0, 1),   // right
    (1, 0),   // down
    (0, -1),  // left
    (-1, 1),  // up-right
    (1, 1),   // down-right
    (1, -1),  // down-left
    (-1, -1), // up-left
];
#[derive(Serialize)]
pub struct Letter {
    letter: char,
    row: usize,
    col: usize,
}

impl Letter {
    pub fn get_id(&self) -> (usize, usize) {
        (self.row, self.col)
    }
}

impl fmt::Display for Letter {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Letter('{}', {}, {})", self.letter, self.row, self.col)
    }
}

#[derive(Deserialize, Debug)]
struct Request {
    combo: String,
    iterations: usize,
}

#[tokio::main]
async fn main() {
    // POST /words
    let words = warp::path("words")
        .and(warp::post())
        .and(warp::body::json())
        .map(|req: Request| {
            let grid = generate_grid(&req.combo);
            let words = read_lines("wordlist.txt");
            let mut builder = TrieBuilder::new();
            for word in &words {
                builder.push(word);
            }
            let word_list = builder.build();
            let mut result: Vec<Vec<Vec<&Letter>>> = vec![];

            for i in 2..req.iterations + 1 {
                result.push(get_combos(&grid, &word_list, i))
            }

            warp::reply::json(&result)
        });

    warp::serve(words).run(([127, 0, 0, 1], 3030)).await;
}

fn get_combos<'a>(
    grid: &'a Vec<Vec<Rc<Letter>>>,
    word_list: &'a Trie<u8>,
    n: usize,
) -> Vec<Vec<&'a Letter>> {
    let mut all_combos: Vec<Vec<&Letter>> = vec![];
    let mut combos: Vec<&Letter> = vec![];
    let mut visited: Vec<Vec<bool>> = vec![vec![false; SIZE]; SIZE];

    for row in 0..SIZE {
        for col in 0..SIZE {
            get_all_combinations(
                grid,
                word_list,
                row,
                col,
                &mut visited,
                &mut combos,
                &mut all_combos,
                n,
            )
        }
    }

    // println!("{}", all_combos.len());

    all_combos
}

pub fn get_all_combinations<'a>(
    grid: &'a Vec<Vec<Rc<Letter>>>,
    trie: &Trie<u8>,
    row: usize,
    col: usize,
    visited: &mut Vec<Vec<bool>>,
    combination: &mut Vec<&'a Letter>,
    all_combinations: &mut Vec<Vec<&'a Letter>>,
    desired: usize,
) {
    if row >= SIZE || col >= SIZE || visited[row][col] {
        return;
    }

    visited[row][col] = true;

    combination.push(grid[row][col].as_ref());

    let word: String = combination.iter().map(|&letter| letter.letter).collect();
    match word.len().cmp(&desired) {
        std::cmp::Ordering::Greater => {
            visited[row][col] = false;
            combination.pop();
            return; // terminate early if word is longer than desired
        }
        std::cmp::Ordering::Equal => {
            if trie.exact_match(&word) {
                all_combinations.push(combination.clone());
            }
        }
        std::cmp::Ordering::Less => {
            if trie.predictive_search(&word).is_empty() {
                visited[row][col] = false;
                combination.pop();
                return; // terminate early if no possible words
            }
            for &(dx, dy) in DIRECTIONS.iter() {
                let r = row as i32 + dx;
                let c = col as i32 + dy;

                if r < 0 || r as usize >= SIZE || c < 0 || c as usize >= SIZE {
                    continue;
                }

                get_all_combinations(
                    grid,
                    trie,
                    r as usize,
                    c as usize,
                    visited,
                    combination,
                    all_combinations,
                    desired,
                );
            }
        }
    }

    visited[row][col] = false;
    combination.pop();
}

fn generate_grid(combo: &String) -> Vec<Vec<Rc<Letter>>> {
    if combo.len() != 25 {
        panic!("bruh invalid combo")
    }

    let real_combo = combo.to_lowercase();

    let mut temp_grid: Vec<Vec<Rc<Letter>>> = vec![];

    for row in 0..combo.len() / SIZE {
        let mut temp_row = vec![];
        for col in 0..SIZE {
            let index = row * SIZE + col;
            let letter = match real_combo.chars().nth(index) {
                Some(c) => c,
                None => panic!("Index out of bounds!"),
            };
            // println!("letter = {}", letter);
            let letter_struct = Letter { letter, row, col };
            let rc_letter = Rc::new(letter_struct);

            temp_row.push(rc_letter.clone());
        }
        temp_grid.push(temp_row);
    }

    temp_grid
}

fn read_lines(filename: &str) -> Vec<String> {
    let mut result = Vec::new();

    for line in fs::read_to_string(filename).unwrap().lines() {
        result.push(line.to_string().to_lowercase())
    }

    result
}
