// input / output
use std::io;
use std::io::{Read, Write};

// arguments from the command line
use std::env;

// timer
use std::time::Instant;

// rng
use rand::Rng;

// deserialising JSON
use serde::{Deserialize, Serialize};

// where the language names : codes are kept.
mod langs;

fn main() {
    clear_screen();

    // arguments for if you are building locally
    let args: Vec<_> = env::args().collect();

    // gets the tatoeba language codes from a seperate file
    let lang_codes = langs::propagate();

    // if arguments are passed
    let language_input = if args.len() > 1 {
        // the input from the command line
        (args[1]).to_string()
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

    clear_screen();

    start_game(sentences, len, language);
}

// https requests tatoeba
// language: the language to request from tatoeba
fn sentences_http_request(language: &str) -> Result<Vec<Sentence>, minreq::Error> {
    // the request string we send to tatoeba
    let request = format!("https://tatoeba.org/en/api_v0/search?from=eng&orphans=no&sort=random&to={language}&unapproved=no");

    // the json response string we get back from tatoeba
    let response = minreq::get(request).send()?;

    // format it
    let rep_string = response.as_str()?;

    // extract a Vec of sentences from plaintext
    let sentences = parse(rep_string).unwrap();
    Ok(sentences)
}

// actually requests from tatoeba and formats it (basically just runs other functions)
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
        let translation = &sentence.get_translation().unwrap().text;

        let words: String;

        // checks if the current language is non-spaced
        let is_non_spaced = non_spaced.iter().any(|x| x == &language);

        // gets a new seed for random number generation
        let mut rng = rand::thread_rng();
        let raw_word: String;
        let index: usize;

        if is_non_spaced {
            // get chars and convert them to strings
            let chars = translation.chars().map(|x| x.to_string());
            // convert the map to a string
            words = chars.collect::<String>();
            // index of the random word
            index = rng.gen_range(0..translation.chars().count());
            // get the random word. it's "raw" because punctuation hasn't been removed yet.
            raw_word = translation
                .chars()
                .map(|x| x.to_string())
                .collect::<Vec<_>>()
                .get(index)
                .unwrap()
                .to_string();
        } else {
            // get "words"
            let split_whitespace = translation.split(' ');
            // convert the split to a string
            words = split_whitespace
                .map(|x| x.to_string() + " ")
                .collect::<String>();
            // how many words there are
            let length = translation.split(' ').count();
            // index of the random word
            index = rng.gen_range(0..length);
            // get the random word. it's "raw" because punctuation hasn't been removed yet.
            raw_word = translation
                .split(' ')
                .collect::<Vec<_>>()
                .get(index)
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
            "{}{}{}",
            halved.next().unwrap(),
            underscores,
            halved.last().unwrap()
        );

        // the english translation
        println!("{}", sentence.text);

        // user input
        let mut guess = String::new();

        print!("> ");
        io::stdout().flush().unwrap();

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

    pause();
}
// parse plaintext JSON response string into a vec of sentences results: the JSON
fn parse(results: &str) -> Result<Vec<Sentence>, String> {
    let sentences: Json = serde_json::from_str(results).map_err(convert_error)?;
    Ok(sentences.results)
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

fn convert_error(err: serde_json::Error) -> String {
    format!(
        "{:#?} error thrown by serde at {}:{}.",
        err.classify(),
        err.line(),
        err.column()
    )
}

fn clear_screen() {
    // clear the screen and position cursor at the top left
    print!("{esc}[2J{esc}[1;1H", esc = 27 as char);
}

fn pause() {
    let mut stdin = io::stdin();
    let mut stdout = io::stdout();

    // print without a newline and flush manually.
    write!(stdout, "Press any key to continue...").unwrap();
    stdout.flush().unwrap();

    // read a single byte and discard
    let _ = stdin.read(&mut [0u8]).unwrap();
}
