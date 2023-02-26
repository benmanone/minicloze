// input / output
use std::io;
use std::io::Write;

// arguments from the command line
use std::env;

// timer
use std::time::Instant;

// rng
use rand::Rng;

// http(s) requests
use minreq;

// parsing unicode
use unescaper;

// where the language names : codes are kept.
// very long list of Vec insert statements
mod langs;

fn main() {
    // arguments for if you are building locally
    let args: Vec<_> = env::args().collect();

    // gets the tatoeba language codes from a seperate file
    let lang_codes = langs::propagate();

    // if arguments are passed
    let language_input = if args.len() > 1 {
        // the input from the command line
        (&args[1]).to_string()
    }
    // if compiled script is run
    else {
        let mut input = String::new();

        print!("What language do you want to study? ");

        // allows multiple outputs on the same line
        io::stdout().flush().unwrap();
        // user input
        io::stdin().read_line(&mut input).unwrap();

        input = input.trim_start().trim_end().to_string();
        input
    };
    print!("Fetching sentences for you...");
    io::stdout().flush().unwrap();

    // to display how long fetching sentences takes
    let now = Instant::now();

    let language_request = language_input.to_lowercase();

    // get the correct code for the input language
    // TODO: autocorrect obvious mistakes (e.g. frnech vs french) and "mistakes" (e.g. mandarin vs
    // mandarin chinese)
    let language = lang_codes
        .get(&language_request.as_str())
        .expect("Please enter a valid language")
        .to_string();

    // gets sentences for the correct language
    let sentences = generate_sentences(&language).unwrap();
    let len = sentences.len();

    let elapsed = now.elapsed();

    println!(
        " Processing complete in {:.2?}, {} sentences parsed.",
        elapsed, len
    );

    start_game(sentences, len, language);
}

// https requests tatoeba
// language: the language to request from tatoeba
fn sentences_http_request(language: &str) -> Result<Vec<Sentence>, minreq::Error> {
    // the request string we send to tatoeba
    let request = format!(
        "{}{}{}",
        "https://tatoeba.org/en/api_v0/search?from=eng&orphans=no&sort=random&to=",
        language,
        "&unapproved=no"
    );
    // the json response string we get back from tatoeba
    let response = minreq::get(request).send()?;

    // format it
    let rep_string = response.as_str()?;

    // where the results actually begin in the json response
    let results_start = 1 + get_char_locations(rep_string, '[')[1];

    // cut out the irrelevant information from the response
    let results = &rep_string[results_start..rep_string.len() - 2];
    // extract a Vec of sentences from this plaintext
    let sentences = parse(results);
    Ok(sentences)
}

// potential feature. ignore.
fn _definition_http_request() {
    // TODO: would be hard to implement but maybe get definition from wiktionary wikimedia API?
}

// actually requests from tatoeba and formats it (basically just runs other functions)
// language: the language to request from tatoeba
fn generate_sentences(language: &str) -> Result<Vec<Sentence>, minreq::Error> {
    // where the initial request happens
    let mut sentences = sentences_http_request(language).unwrap();
    let len = sentences.len();

    // makes sure we always get 9 sentences
    if len != 9 {
        let difference = 9 - len;
        // makes more requests if required
        let mut sentences_difference = sentences_http_request(language)
            .unwrap()
            .into_iter()
            .take(difference)
            .collect::<Vec<_>>();

        sentences.append(&mut sentences_difference);
    }
    Ok(sentences)
}

// where the actual "game" happens
// sentences: sentences for the game
// len: how many sentences there are. almost always 9
// language: what language the game is in
fn start_game(sentences: Vec<Sentence>, len: usize, language: String) {
    // for a correct count at the end
    let mut correct = 0;
    // languages which don't use spaces. require some different logic
    let non_spaced = [
        "cmn", "lzh", "hak", "cjy", "nan", "hsn", "gan", "jpn", "tha", "khm", "lao", "mya",
    ];
    for sentence in sentences {
        // just the sentence's original text
        let mut cropped = sentence.translation;
        // removes the last useless character (probably a ")
        cropped.pop();

        println!();

        let words: String;

        // checks if the current language is non-spaced
        let is_non_spaced = non_spaced.iter().any(|x| x == &language);

        // gets a new seed for random number generation
        let mut rng = rand::thread_rng();
        let raw_word: String;
        let index: usize;

        if is_non_spaced {
            // get chars and convert them to strings
            let chars = cropped.chars().map(|x| x.to_string());
            // convert the map to a string
            words = chars.collect::<String>();
            // index of the random word
            index = rng.gen_range(0..words.len());
            // doesn't work right now. should do the same as raw_word below
            raw_word = cropped
                .chars()
                .map(|x| x.to_string())
                .collect::<Vec<String>>()
                .iter()
                .nth(index)
                .unwrap()
                .to_string();
        } else {
            // get "words"
            let split_whitespace = cropped.split(" ");
            // convert the split to a string
            words = split_whitespace
                .map(|x| x.to_string() + " ")
                .collect::<String>();
            // how many words there are
            let length = cropped.split(" ").collect::<Vec<_>>().len();
            // index of the random word
            index = rng.gen_range(0..length);
            // get the random word. it's "raw" because punctuation hasn't been removed yet.
            raw_word = cropped
                .split(" ")
                .collect::<Vec<_>>()
                .into_iter()
                .nth(index)
                .unwrap()
                .to_string();
        }

        // remove punctuation
        let word = raw_word.replace(&['(', ')', ',', '.', ';', ':'][..], "");

        // how many underscores to print
        let underscores = vec!['_'; word.chars().count()]
            .into_iter()
            .collect::<String>();

        // split the sentence along the random word
        let mut halved = words.split(&word).collect::<Vec<&str>>().into_iter();

        // print either side and in the middle the underscores
        println!(
            "{} {} {}",
            halved.next().unwrap(),
            underscores,
            halved.last().unwrap()
        );

        // the english translation
        println!("\n{}", sentence.text);

        // user input
        let mut guess = String::new();

        io::stdin().read_line(&mut guess).unwrap();

        // if they're equal
        if guess.trim().to_lowercase().contains(&word.to_lowercase()) {
            correct += 1;
            println!("Correct.\n");
        } else {
            println!("Wrong, {}.\n", word);
        }
    }
    println!("{}/{} sentences correct", correct, len);
}

// parse plaintext response string into a vec of sentences
// results: the plaintext
fn parse(results: &str) -> Vec<Sentence> {
    let mut sentences = Vec::new();
    let mut raw = results;

    // there is a new sentence every second instance of },
    // sentence_and_remainder yields both halves of results split at 2nd },
    for _i in 1..10 {
        let sentence_and_remainder = raw
            .match_indices("},{")
            .nth(0)
            .map(|(index, _)| raw.split_at(index));

        if let Some(s) = sentence_and_remainder {
            // make sure the final bracket is there
            let including_bracket = (s.0).to_owned() + "}";
            // make a new sentence struct
            let sentence = Sentence::new(&including_bracket);

            // make sure it's a valid sentence (not a blank() which was rejected for some reason)
            if sentence.id != -1 {
                sentences.push(sentence);
            } else {
                // TODO: instead get a count of how many sentences were rejected
                print!(" Sentence rejected...");
            }
            // cuts out the irrelevant part
            raw = &s.1[2..];
        }
    }
    sentences
}

// represents a sentence. id is the tatoeba id of the sentence, not used anywhere currently
struct Sentence {
    id: i32,
    text: String,
    translation: String,
}

impl Sentence {
    // make a new sentence
    fn new(string: &String) -> Sentence {
        // where "text" shows up in the response. take a look at a tatoeba api response if you want
        // a better understanding
        let text_positions: Vec<usize> = string.match_indices("text").map(|(i, _)| i).collect();

        // if there's not an english translation text
        if text_positions.len() < 2 {
            return blank();
        }

        // where "id" shows up
        let id_position = string.find("id").unwrap();

        // get the start-end of the text
        let text_start = text_positions[0] + 7;
        let text_end = &string[text_start..].find(",\"l").unwrap() + text_start;

        // get the start-end of the english translation
        let translation_start = text_positions[1] + 7;
        // sometimes it panics. this is an easy way to make it work. i don't remember why.
        let try_translation_end = &string[translation_start..].find(",\"l");

        if let Some(x) = try_translation_end {
            let translation_end = x + translation_start;
            // the sentence to return
            Sentence {
                id: string[id_position + 4..(string[id_position - 2..].find(',').unwrap())]
                    .parse::<i32>()
                    .unwrap(),
                text: parse_unicode(&string[text_start..text_end - 1].to_string()),
                translation: parse_unicode(&string[translation_start..translation_end].to_string()),
            }
        } else {
            // send a blank sentence back, this is a weird way to handle it
            // TODO: just return a Result
            blank()
        }
    }
}

// parse unicode escape characters
fn parse_unicode(string: &str) -> String {
    let mut i = 0;
    let mut chars: Vec<char> = Vec::new();

    while i < string.len() {
        if string.as_bytes()[i] as char == '\\' {
            let number = &string[i + 2..i + 6];
            let format = "\\".to_owned() + "u" + "{" + number + "}";

            let result = unescaper::unescape(format.as_str());

            if let Ok(x) = result {
                let character = x.chars().next().unwrap();

                chars.push(character);
            }
            i += 6;
        } else {
            chars.push(string.chars().nth(i).unwrap());
            i += 1;
        }
    }
    chars.into_iter().collect()
}

fn blank() -> Sentence {
    Sentence {
        id: -1,
        text: "".to_string(),
        translation: "".to_string(),
    }
}

fn get_char_locations(string: &str, query: char) -> Vec<usize> {
    // returns locations a char occurs in a given string. doesn't really need to be a function as i
    // only use it once
    string
        .chars()
        .enumerate()
        .filter(|(_, c)| *c == query)
        .map(|(i, _)| i)
        .collect::<Vec<_>>()
}
