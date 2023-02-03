use std::io;
use std::io::Write;
use std::env;
use std::time::Instant;
use rand::prelude::*;
use minreq;
use unescaper;

fn main() -> Result<(), minreq::Error> {
    let args: Vec<_>  = env::args().collect();
    let mut language: String;
    if args.len() > 1 {
        language = (&args[1]).to_string();
    }
    else {
        language = String::new();
        print!("What language do you want to study? ");
        io::stdout().flush().unwrap();
        io::stdin().read_line(&mut language).unwrap();
        language = language.trim().to_string();
    }
    print!("Fetching sentences for you...");
    // allows output on the same line
    io::stdout().flush().unwrap();
    let now = Instant::now();
    let request = "https://tatoeba.org/en/api_v0/search?from=eng&orphans=no&sort=random&to=".to_owned()+&language.trim()+"&unapproved=no";
    let response = minreq::get(request).send()?;
    let elapsed = now.elapsed();
    let rep_string = response.as_str()?;
    // finds the start of results by locating the first [ in rep_string
    let results_start = 1+get_char_locations(rep_string, '[')[1];
    let results = &rep_string[results_start..rep_string.len()-2];
    let sentences = parse(results);
    let len = sentences.len();
    print!(" Processing complete in {:.2?}, {} sentences parsed.\n", elapsed, len);
    play(sentences, len, language);
    Ok(())
}

fn play(sentences: Vec<Sentence>, len: usize, language: String) {
    let mut correct = 0;
    let non_spaced = ["cmn", "lzh", "hak", "cjy", "nan", "hsn", "gan", "jpn", "tha", "khm", "lao", "mya"];
    for sentence in sentences {
        let mut cropped = sentence.translation;
        cropped.pop();
        println!();
        let rand = &mut rand::thread_rng();
        let words: Vec<String>;
        let is_non_spaced = non_spaced.iter().any(|x| x == &language);
        if is_non_spaced {
            words = cropped.chars().map(|x| x.to_string()).collect::<Vec<String>>();
        }
        else {
            words = cropped.split(' ').map(|x| x.to_string()).collect::<Vec<String>>();
        }
        let mut shuffled = words.clone();
        shuffled.shuffle(rand);
        let word = &shuffled[0];
        for i in words {
            if &i.to_string() == word {
                for _j in word.chars() {
                        print!("_");
                }
                if !is_non_spaced {
                    print!(" ");
                }
            }
            else if !is_non_spaced {
                print!("{} ", i);
            }
            else {
                print!("{}", i);
            }
        }
        println!("\n{}", sentence.text);
        let mut guess = String::new();
        io::stdin().read_line(&mut guess).unwrap();
        if guess.trim().to_lowercase().contains(&word.to_lowercase()) {
            correct += 1;
            println!("Correct.\n");
        }
        else {
            println!("Wrong, {}.\n", word);
        }
    }
    println!("{}/{} sentences correct", correct, len);
}

fn parse(results: &str) -> Vec<Sentence> {
    let mut sentences = Vec::new();
    let mut raw = results;
    
    // there is a new sentence every second instance of },
    // sentence_and_remainder yields both halves of results split at 2nd },
    for _i in 1..10 {
        let sentence_and_remainder = raw.match_indices("},{").nth(0).map(|(index, _)| raw.split_at(index));
        if let Some(s) = sentence_and_remainder {
            let including_bracket = (s.0).to_owned() + "}";
            let sentence = Sentence::new(&including_bracket);
            if sentence.id != -1 {
                sentences.push(sentence);
            }
            raw = &s.1[2..];
        }
    }
    sentences
}

struct Sentence {
    id: i32,
    text: String,
    translation: String,
}

impl Sentence {
    fn new(string: &String) -> Sentence {
        let text_positions: Vec<usize> = string.match_indices("text").map(|(i, _)|i).collect();

        if text_positions.len() < 2 {
            return blank()
        }

        let id_position = string.find("id").unwrap();

        let text_start = text_positions[0]+7;
        let text_end = &string[text_start..].find(",\"l").unwrap()+text_start;

        let translation_start = text_positions[1]+7;
        let try_translation_end = &string[translation_start..].find(",\"l");
        if let Some(x) = try_translation_end {
            let translation_end = x+translation_start;
            Sentence {
                id: string[id_position+4..(string[id_position-2..].find(',').unwrap())].parse::<i32>().unwrap(),
                text: string[text_start..text_end-1].to_string(),
                translation: parse_unicode(&string[translation_start..translation_end].to_string()),
            }
        }
        else {
            blank()
        }
    }
}

fn parse_unicode(string: &str) -> String {
    let mut i = 0;
    let mut chars: Vec<char> = Vec::new();
    while i < string.len() {
        if string.as_bytes()[i] as char == '\\' {
            let number = &string[i+2..i+6];
            let format = "\\".to_owned() + "u" + "{" + number + "}";
            let result = unescaper::unescape(format.as_str());
            if let Ok(x) = result {
                let character = x.chars().next().unwrap();
                chars.push(character);
            }
            i += 6;
        }
        else {
            chars.push(string.chars().nth(i).unwrap());
            i += 1;
        }
    }
    chars.into_iter().collect()
}

fn blank() -> Sentence {
    Sentence { id: -1, text: "".to_string(), translation: "".to_string() }
}

fn get_char_locations(string: &str, query: char) -> Vec<usize> {
    string.chars().enumerate().filter(|(_, c)| *c == query).map(|(i, _)| i).collect::<Vec<_>>()
}
