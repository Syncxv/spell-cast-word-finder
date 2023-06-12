extern crate warp;
use serde::Deserialize;
use std::{sync::Arc, time::Instant};
use trie_rs::{Trie, TrieBuilder};
use warp::Filter;

use scf::{generate_grid, get_combos, read_lines, Letter};

#[derive(Deserialize, Debug)]
struct Request {
    combo: String,
    iterations: usize,
}

#[tokio::main]
async fn main() {
    let words = read_lines("wordlist.txt");
    let mut builder = TrieBuilder::new();
    for word in &words {
        builder.push(word);
    }
    let word_list = Arc::new(builder.build());
    // POST /words
    let words = warp::path("words")
        .and(warp::post())
        .and(warp::body::json())
        .and(with_word_list(word_list))
        .map(|req: Request, word_list: Arc<Trie<u8>>| {
            let start_time = Instant::now();
            let grid = generate_grid(&req.combo);
            let mut result: Vec<Vec<Vec<&Letter>>> = vec![];

            for i in 2..req.iterations + 1 {
                result.push(get_combos(&grid, &word_list, i))
            }

            let duration = start_time.elapsed();
            println!("Time elapsed is: {:?}", duration);
            warp::reply::json(&result)
        });

    warp::serve(words).run(([0, 0, 0, 0], 3030)).await;
}

fn with_word_list(
    word_list: Arc<Trie<u8>>,
) -> impl Filter<Extract = (Arc<Trie<u8>>,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || word_list.clone())
}
