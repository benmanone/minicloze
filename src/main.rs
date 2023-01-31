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
    }
    print!("Fetching sentences for you...");
    // allows output on the same line
    io::stdout().flush().unwrap();
    let now = Instant::now();
    let request = "https://tatoeba.org/en/api_v0/search?from=eng&orphans=no&sort=random&to=".to_owned()+&language.trim()+"&tran";
    let response = minreq::get(request).send()?;
    let elapsed = now.elapsed();
    let rep_string = response.as_str()?;
    // finds the start of results by locating the first [ in rep_string
    let results_start = 1+get_char_locations(rep_string, '[')[1];
    let results = &rep_string[results_start..rep_string.len()-2];
    let sentences = parse(results);
    print!(" Processing complete in {:.2?}, {} sentences parsed.\n", elapsed, sentences.len());
    play(sentences);
    Ok(())
}

fn play(sentences: Vec<Sentence>) {
    for sentence in sentences {
        println!();
        let rand = &mut rand::thread_rng();
        let mut words = sentence.translation.split(' ').collect::<Vec<_>>();
        words.shuffle(rand);
        let word = words[0];
        for i in sentence.translation.split(' ').collect::<Vec<_>>() {
            if i == word {
                for _j in word.chars() {
                    print!("_");
                }
                print!(" ")
            }
            else {
                print!("{} ", i);
            }
        }
        println!("\n{}", sentence.text);
        let mut guess = String::new();
        io::stdin().read_line(&mut guess).unwrap();
        if guess.trim().to_lowercase() == word.to_lowercase() {
            println!("Correct.\n");
        }
        else {
            println!("Wrong, {}.\n", word);
        }
    }
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
        if text_positions.len() < 2 { return Sentence { id: -1, text: "".to_string(), translation: "".to_string() } }
        let id_position = string.find("id").unwrap();
        let text_start = text_positions[0]+7;
        let text_end = &string[text_start..].find(",\"l").unwrap()+text_start;
        let translation_start = text_positions[1]+7;
        let translation_end = &string[translation_start..].find(",\"l").unwrap()+translation_start;

        Sentence {
            id: string[id_position+4..(string[id_position-2..].find(',').unwrap())].parse::<i32>().unwrap(),
            text: string[text_start..text_end-1].to_string(),
            translation: parse_unicode(&string[translation_start..translation_end-1].to_string()),
        }
    }
}

fn parse_unicode(string: &str) -> String {
    let mut i = 0;
    let mut chars: Vec<char> = Vec::new();
    while i < string.len()-1 {
        if string.as_bytes()[i] as char == '\\' {
            let number = &string[i+2..i+6];
            let format = "\\".to_owned() + "u" + "{" + number + "}";
            let x = unescaper::unescape(&*format).unwrap().chars().nth(0).unwrap();
            chars.push(x);
            i += 6;
        }
        else {
            chars.push(string.chars().nth(i).unwrap());
            i += 1;
        }
    }
    chars.into_iter().collect()
}

fn get_char_locations(string: &str, query: char) -> Vec<usize> {
    string.chars().enumerate().filter(|(_, c)| *c == query).map(|(i, _)| i).collect::<Vec<_>>()
}
