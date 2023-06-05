use rand::Rng;
use levenshtein::levenshtein;
use serde::{Deserialize, Serialize};
use std::env;
use std::io;
use std::io::{Read, Write};
use std::time::Instant;
// where the language name codes are kept.
mod langs;

const DISTANCE_FOR_CLOSE: i32 = 3;

fn main() {
    clear_screen();

    let args: Vec<_> = env::args().collect();

    // gets the tatoeba language codes from a seperate file
    let lang_codes = langs::propagate();

    let language_input = if args.len() > 1 {
        // the input from the command line
        (args[1]).to_string()
    }
    // if compiled script is run
    else {
        let mut input = String::new();

        print!("What language do you want to study? ");

        read_into(&mut input);

        input = input.trim_start().trim_end().to_string();
        input
    };
    print!("Fetching sentences for you...");
    io::stdout().flush().unwrap();

    let now = Instant::now();

    print!("debug");
    let language_request = language_input.to_lowercase();

    // TODO: autocorrect obvious mistakes (e.g. frnech vs french) and "mistakes" (e.g. mandarin vs
    // mandarin chinese)
    let language = lang_codes
        .get(&language_request.as_str())
        .expect("Please enter a valid language")
        .to_string();

    let sentences = generate_sentences(&language).unwrap();
    let len = sentences.len();
    let elapsed = now.elapsed();

    println!(
        " Processing complete in {:.2?}, {} sentences parsed.",
        elapsed, len
    );

    start_game(sentences, len, language, 0, 0);
}

// language: the language to request from tatoeba
fn sentences_http_request(language: &str) -> Result<Vec<Sentence>, minreq::Error> {
    let request = format!("https://tatoeba.org/en/api_v0/search?from=eng&orphans=no&sort=random&to={language}&unapproved=no");
    let response = minreq::get(request).send()?;

    let rep_string = response.as_str()?;

    let sentences = parse(rep_string).unwrap();
    Ok(sentences)
}

// language: the language to request from tatoeba
fn generate_sentences(language: &str) -> std::result::Result<Vec<Sentence>, minreq::Error> {
    // where the initial request happens
    let mut sentences = sentences_http_request(language).unwrap();
    let len = sentences.len();

    // makes sure we always get 10 sentences
    if len != 10 {
        let difference = 10 - len;
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

// sentences: sentences for the game
// len: how many sentences there are. almost always 10
// language: what language the game is in
// previous_correct: the total previous correct score
// total: the previous total
fn start_game(sentences: Vec<Sentence>, len: usize, language: String, previous_correct: i32, total: i32) {
    clear_screen();
    let mut correct = 0;
    let non_spaced = [
        "cmn", "lzh", "hak", "cjy", "nan", "hsn", "gan", "jpn", "tha", "khm", "lao", "mya",
    ];
    for sentence in sentences {
        // just the sentence's original text
        let translation = &sentence.get_translation().unwrap().text;

        let words: String;

        let is_non_spaced = non_spaced.iter().any(|x| x == &language);

        let mut rng = rand::thread_rng();
        let raw_word: String;
        let gap_index: usize;

        if is_non_spaced {
            let char_strings = translation.chars().map(|x| x.to_string());
            words = char_strings.collect::<String>();
            gap_index = rng.gen_range(0..translation.chars().count());
            raw_word = translation
                .chars()
                .map(|x| x.to_string())
                .collect::<Vec<_>>()
                .get(gap_index)
                .unwrap()
                .to_string();
        } else {
            words = translation.split_inclusive(' ').collect::<String>();
            let length = translation.split_inclusive(' ').count();
            gap_index = rng.gen_range(0..length);
            raw_word = translation
                .split_inclusive(' ')
                .collect::<Vec<_>>()
                .get(gap_index)
                .unwrap()
                .to_string();
        }

        let word = raw_word.replace(
            &[
                '(', ')', ',', '.', ';', ':', '?', '¿', '!', '¡', '"', '\'', '«', '»',
            ][..],
            "",
        );

        let stripped_word = word.replace(' ', "");

        let underscores_num = vec!['_'; stripped_word.chars().count()]
            .into_iter()
            .collect::<String>();

        let mut halved = words.split(&word).collect::<Vec<&str>>().into_iter();

        println!(
            "{}: {} {} {}",
            language.to_uppercase(),
            halved.next().unwrap(),
            underscores_num,
            halved.last().unwrap_or_default()
        );

        println!("ENG: {}", sentence.text);

        let mut guess = String::new();

        print!("> ");
        read_into(&mut guess);

        let levenshtein_distance = levenshtein(&guess.trim().to_lowercase(), &stripped_word.to_lowercase());

        if levenshtein_distance == 0 {
            correct += 1;
            println!("Correct.");
        } else if levenshtein_distance < DISTANCE_FOR_CLOSE as usize {
            println!("Close, {}.", stripped_word);
        } else {
            println!("Wrong, {}.", stripped_word);
        }
        println!();

        loop {
            let mut lookup = String::new();
            println!("Lookup a word? [enter word or ignore]");
            print!("> ");
            read_into(&mut lookup);

            if lookup.trim().is_empty() {
                break;
            } else {
                let lang_codes = langs::propagate();
                let mut full_language: String = "a".to_string();
                // gets key from value
                for pair in lang_codes {
                    if pair.1 == language {
                        full_language = pair.0.to_string();
                    }
                }
                let titlecase_language = format!(
                    "{}{}",
                    full_language[..1].to_uppercase(),
                    &full_language[1..]
                );

                open::that(
                    [
                        "https://en.wiktionary.org/wiki/",
                        lookup.trim(),
                        "#",
                        titlecase_language.as_str(),
                    ]
                    .join(""),
                )
                .unwrap();
            }
        }

        println!("\n");
    }

    let new_correct = previous_correct + correct;
    let new_total = total + len as i32;

    if (new_total) / len as i32 == 1 {
        println!("{}/{} sentences correct. Play again? [y/n]", correct, len);
    }
    else {
        println!("{}/{} sentences correct locally, {}/{} sentences correct overall. Play again? [y/n]", correct, len, new_correct, new_total);
    }
    print!("> ");

    let mut replay = String::new();

    read_into(&mut replay);

    if replay.trim().to_lowercase().contains('y') {
        let sentences = generate_sentences(language.as_str()).unwrap();
        let len = sentences.len();
        start_game(sentences, len, language, new_correct, new_total);
    } else {
        pause();
    }
}

// parse plaintext JSON response string into a Vec of Sentences results: the JSON
fn parse(results: &str) -> Result<Vec<Sentence>, String> {
    let sentences: Json = serde_json::from_str(results).map_err(convert_error)?;
    Ok(sentences.results)
}

// converts a serde error into a string
fn convert_error(err: serde_json::Error) -> String {
    format!(
        "{:#?} error thrown by serde at {}:{}.",
        err.classify(),
        err.line(),
        err.column()
    )
}

// clear the screen and position cursor at the top left
fn clear_screen() {
    print!("{esc}[2J{esc}[1;1H", esc = 27 as char);
}

// wait for keystroke before quitting
fn pause() {
    let mut stdin = io::stdin();
    let mut stdout = io::stdout();

    // print without a newline and flush manually.
    write!(stdout, "Press any key to exit").unwrap();
    stdout.flush().unwrap();

    // read a single byte and discard
    let _ = stdin.read(&mut [0u8]).unwrap();
}

// user input
fn read_into(buffer: &mut String) {
    io::stdout().flush().unwrap();
    io::stdin().read_line(buffer).unwrap();
}

// represents the entire JSON response. results is the sentences found.
#[derive(Deserialize, Serialize)]
struct Json {
    results: Vec<Sentence>,
}

// represents a sentence. id is the tatoeba id of the sentence, not used anywhere currently
#[derive(Deserialize, Serialize)]
struct Sentence {
    id: i32,
    text: String,
    translations: Vec<Vec<Translation>>,
}

// represents a translation. id is the tatoeba id of the translation
#[derive(Deserialize, Serialize)]
struct Translation {
    id: i32,
    text: String,
}

impl Sentence {
    // get the sentence's translation
    // sometimes translations.0 will be blank
    fn get_translation(&self) -> Option<&Translation> {
        if let Some(t) = self.translations.get(0).unwrap().get(0) {
            Some(t)
        } else {
            self.translations.get(1).unwrap().get(0)
        }
    }
}
