use reqwest;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Eq, PartialEq)]
struct Rhyme {
    word: String,
    score: i32,
}

type RhymeResultOk = Vec<Rhyme>;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let word: &str = "heart";
    let rhymeurl: std::string::String = format!(
        "https://rhymebrain.com/talk?function=getRhymes&maxResults=50&word={}",
        word
    );
    let rhymes = reqwest::blocking::get(&rhymeurl)?.json::<RhymeResultOk>()?;
    let rhyme_references = rhymes.iter().map(|r| r).collect::<Vec<_>>();
    let best_rhymes = keep_best_rhymes(rhyme_references);
    println!("{:?}", best_rhymes);

    Ok(())
}

fn keep_best_rhymes(rhymes: Vec<&Rhyme>) -> Vec<&Rhyme> {
    let max_score = rhymes.iter().map(|r| r.score).max().unwrap();
    return rhymes
        .into_iter()
        .filter(|r| r.score == max_score)
        .collect();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_keep_best_rhymes() {
        let best_rhyme: Rhyme = Rhyme {
            word: "boo".to_string(),
            score: 300,
        };
        let worst_rhyme = Rhyme {
            word: "zoo".to_string(),
            score: 222,
        };
        let input: Vec<&Rhyme> = vec![&best_rhyme, &worst_rhyme, &worst_rhyme];

        let result: Vec<&Rhyme> = keep_best_rhymes(input);
        assert_eq!(result.len(), 1);
        assert_eq!(*result[0], best_rhyme);
    }
}
