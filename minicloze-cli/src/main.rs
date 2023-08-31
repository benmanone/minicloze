use minicloze_lib::{
    langs::propagate, sentence::{generate_sentences, remove_punctuation}, sentence::Sentence,
    wiktionary::wiktionary_try_open,
};

use levenshtein::levenshtein;

use std::env;
use std::io;
use std::io::{Read, Write};
use std::time::Instant;

use colored::*;

const DISTANCE_FOR_CLOSE: i32 = 3;

fn main() {
    clear_screen();

    let args: Vec<_> = env::args().collect();

    // gets the tatoeba language codes from a separate file
    let lang_codes = propagate();

    let inverse = if args.len() > 2 && args[2] == "inverse" {
        true
    } else {
        false
    };
    
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

    start_game(sentences, len, language, 0, 0, inverse);
}

// sentences: sentences for the game
// len: how many sentences there are. always 10 if the language has enough sentences
// language: what language the game is in
// previous_correct: the total previous correct score
// total: the previous total
fn start_game(
    sentences: Vec<Sentence>,
    len: usize,
    language: String,
    previous_correct: i32,
    total: i32,
    inverse: bool,
) {
    clear_screen();
    let mut correct = 0;

    for sentence in sentences {
        let prompt = sentence.generate_prompt(&language, inverse);

        let underscores_num = vec!['_'; prompt.word.chars().count()]
            .into_iter()
            .collect::<String>();

        let print_language = if inverse {
            "eng"
        } else {
            &language
        };

        let non_english = format!(
            "{}{}{}{}",
            (print_language.to_uppercase() + ": ").bold(),
            prompt.first_half,
            underscores_num + " ",
            prompt.second_half
        );

        if inverse {
            println!("{}{}{}", format_target_language(&language.to_uppercase()), format_target_language(&": ".to_string()), format_target_language(&sentence.get_translation().unwrap().text));
            println!("{}", &non_english);
        } else {
            println!("{}", format_target_language(&non_english));
            println!("{} {}", "ENG:".bold(), sentence.text);
        }

        let mut guess = String::new();

        print!("> ");
        read_into(&mut guess);

        let levenshtein_distance =
            levenshtein(&remove_punctuation(&guess.trim().to_lowercase()), &prompt.word.to_lowercase().trim());

        if levenshtein_distance == 0 {
            correct += 1;
            println!("{}", "Correct.".bold().bright_white().on_green());
        } else if levenshtein_distance < DISTANCE_FOR_CLOSE as usize {
            println!("Close, {}.", prompt.word.to_lowercase().bold().bright_white().on_yellow());
        } else {
            println!("Wrong, {}.", prompt.word.bold().bright_white().on_red());
        }
        println!();

        loop {
            let mut lookup = String::new();
            println!("{} {}", "Lookup a word?", "[enter word or ignore]".italic());
            print!("> ");
            read_into(&mut lookup);

            if lookup.trim().is_empty() {
                break;
            } else {
                wiktionary_try_open(lookup, &language);
            }
        }
        println!();
    }

    let new_correct = previous_correct + correct;
    let new_total = total + len as i32;

    if (new_total) / len as i32 == 1 {
        println!("{}/{} sentences correct. Play again? [y/n]", correct, len);
    } else {
        println!(
            "{}/{} sentences correct locally, {}/{} sentences correct overall. Play again? [y/n]",
            correct, len, new_correct, new_total
        );
    }
    print!("> ");

    let mut replay = String::new();

    read_into(&mut replay);

    if replay.trim().to_lowercase().contains('y') {
        let sentences = generate_sentences(language.as_str()).unwrap();
        let len = sentences.len();
        start_game(sentences, len, language, new_correct, new_total, inverse);
    } else {
        pause();
    }
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

fn format_target_language(str: &String) -> ColoredString {
    str.black().on_bright_white()
}
