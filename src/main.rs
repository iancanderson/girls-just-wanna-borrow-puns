#[macro_use]
extern crate prettytable;

use prettytable::{Cell, Row, Table};
use reqwest;
use serde::{Deserialize, Serialize};
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

#[derive(Debug)]
struct Pun {
    original: String,
    pun: String,
    rhyme_word: String,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    let word: &str = args[1].as_str();

    let rhymeurl: std::string::String = format!("https://api.datamuse.com/words?rel_rhy={}", word);
    let rhymes = reqwest::blocking::get(&rhymeurl)?.json::<RhymeResultOk>()?;
    let rhyme_references = rhymes.iter().map(|r| r).collect::<Vec<_>>();
    let best_rhymes = keep_single_words(rhyme_references);
    // println!("{:?}", best_rhymes);

    // Load array of strings from text file
    let beatles_songs = lines_from_file("phrases/beatles-songs.txt");
    // println!("{:?}", beatles_songs);

    let puns = puns(&beatles_songs, &best_rhymes, word);
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
        Cell::new("Rhyme word")
            .with_style(prettytable::Attr::Bold)
            .with_style(prettytable::Attr::ForegroundColor(
                prettytable::color::GREEN,
            )),
    ]));

    for pun in puns {
        table.add_row(row![pun.pun, pun.original, pun.rhyme_word]);
    }
    table.printstd();

    Ok(())
}

fn replace_word_in_phrase(phrase: &str, word: &str, replacement: &str) -> String {
    // println!("Replacing {} with {} in {}", word, replacement, phrase);
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

fn puns(phrases: &Vec<String>, rhymes: &Vec<&Rhyme>, word: &str) -> Vec<Pun> {
    let mut puns = Vec::new();
    for phrase in phrases {
        let phrase_lower = phrase.to_lowercase();
        for rhyme in rhymes {
            let new_phrase = replace_word_in_phrase(&phrase_lower, &rhyme.word, word);
            if new_phrase != phrase_lower {
                puns.push(Pun {
                    original: phrase.to_string(),
                    pun: new_phrase,
                    rhyme_word: rhyme.word.to_string(),
                });
            }
        }
    }
    puns
}

fn keep_single_words(rhymes: Vec<&Rhyme>) -> Vec<&Rhyme> {
    return rhymes
        .into_iter()
        .filter(|r| r.word.split(" ").count() == 1)
        .collect();
}

fn lines_from_file(filename: impl AsRef<Path>) -> Vec<String> {
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
