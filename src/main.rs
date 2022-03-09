#[macro_use]
extern crate prettytable;

use prettytable::{Cell, Row, Table};
use rand::prelude::SliceRandom;
use reqwest;
use serde::{Deserialize, Serialize};
use std::fs::read_dir;
use std::fs::File;
use std::io::BufRead;
use std::io::BufReader;
use std::path::Path;

#[derive(Debug, Deserialize, Serialize, Eq, PartialEq)]
struct Rhyme {
    word: String,
    score: i32,
}

type RhymeResultOk = Vec<Rhyme>;

struct Phrase {
    content: String,
    source: String,
}

#[derive(Debug)]
struct Pun {
    original: String,
    pun: String,
    phrase_source: String,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    let word: &str = &args[1];
    let num_puns: &str = if args.len() > 2 { &args[2] } else { "10" };

    let (rhymes, phrases) = load_rhymes_and_phrases(word).unwrap();
    let puns = puns(&phrases, &rhymes, word);
    // Get random puns from vec
    let random_puns =
        puns.choose_multiple(&mut rand::thread_rng(), num_puns.parse::<usize>().unwrap());
    print_puns(&random_puns.collect::<Vec<_>>());

    Ok(())
}

fn load_rhymes_and_phrases(
    word: &str,
) -> Result<(Vec<Rhyme>, Vec<Phrase>), Box<dyn std::error::Error>> {
    let rhymeurl: std::string::String = format!("https://api.datamuse.com/words?rel_rhy={}", word);
    let rhymes = reqwest::blocking::get(&rhymeurl)?.json::<RhymeResultOk>()?;
    let best_rhymes = keep_single_words(rhymes);
    let phrases = load_phrases();
    return Ok((best_rhymes, phrases));
}

fn print_puns(puns: &[&Pun]) {
    let mut table = Table::new();
    table.add_row(Row::new(vec![
        Cell::new("Pun")
            .with_style(prettytable::Attr::Bold)
            .with_style(prettytable::Attr::ForegroundColor(
                prettytable::color::GREEN,
            )),
        Cell::new("Original")
            .with_style(prettytable::Attr::Bold)
            .with_style(prettytable::Attr::ForegroundColor(
                prettytable::color::GREEN,
            )),
        Cell::new("Phrase source")
            .with_style(prettytable::Attr::Bold)
            .with_style(prettytable::Attr::ForegroundColor(
                prettytable::color::GREEN,
            )),
    ]));

    for pun in puns {
        table.add_row(row![pun.pun, pun.original, pun.phrase_source]);
    }
    table.printstd();
}

fn replace_word_in_phrase(phrase: &str, word: &str, replacement: &str) -> String {
    let mut new_phrase = String::new();
    let phrase_words = phrase.split_whitespace();
    for phrase_word in phrase_words {
        if phrase_word == word {
            new_phrase.push_str(replacement);
        } else {
            new_phrase.push_str(phrase_word);
        }
        new_phrase.push_str(" ");
    }
    new_phrase.trim().to_string()
}

fn load_phrases() -> Vec<Phrase> {
    // Get all filenames in phrases directory
    let phrase_filenames: Vec<String> = read_dir("phrases")
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .filter(|path| path.is_file())
        .filter(|path| path.extension().unwrap() == "txt")
        .map(|path| path.to_str().unwrap().to_string())
        .collect();

    let (tx, rx) = std::sync::mpsc::channel();
    for filename in phrase_filenames {
        let tx = tx.clone();
        std::thread::spawn(move || {
            for line in lines_from_file(&filename) {
                tx.send(Phrase {
                    content: line,
                    source: filename.clone(),
                })
                .unwrap();
            }
        });
    }

    drop(tx);

    let mut phrases = Vec::new();
    while let Ok(received) = rx.recv() {
        phrases.push(received);
    }

    return phrases;
}

fn puns(phrases: &Vec<Phrase>, rhymes: &Vec<Rhyme>, word: &str) -> Vec<Pun> {
    let mut puns = Vec::new();
    for phrase in phrases {
        let phrase_lower = phrase.content.to_lowercase();
        for rhyme in rhymes {
            let new_phrase = replace_word_in_phrase(&phrase_lower, &rhyme.word, word);
            if new_phrase != phrase_lower {
                puns.push(Pun {
                    original: phrase.content.to_string(),
                    pun: new_phrase,
                    phrase_source: phrase.source.clone(),
                });
            }
        }
    }
    puns
}

fn keep_single_words(rhymes: Vec<Rhyme>) -> Vec<Rhyme> {
    return rhymes
        .into_iter()
        .filter(|r| r.word.split(" ").count() == 1)
        .collect();
}

fn lines_from_file(filename: &impl AsRef<Path>) -> Vec<String> {
    let file = File::open(filename).expect("no such file");
    let buf = BufReader::new(file);
    buf.lines()
        .map(|l| l.expect("Could not parse line"))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_keep_single_words() {
        let phrase_rhyme: Rhyme = Rhyme {
            word: "boo hoo".to_string(),
            score: 300,
        };
        let word_rhyme = Rhyme {
            word: "zoo".to_string(),
            score: 222,
        };
        let input: Vec<&Rhyme> = vec![&phrase_rhyme, &word_rhyme];
        let result: Vec<&Rhyme> = keep_single_words(input);
        assert_eq!(result.len(), 1);
        assert_eq!(*result[0], word_rhyme);
    }
}
