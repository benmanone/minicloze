use minicloze_lib::{
    langs::propagate,
    sentence::Sentence,
    sentence::{generate_sentences, remove_punctuation},
    wiktionary::generate_url,
};

use levenshtein::levenshtein;

use std::env;
use std::io;
use std::io::{Read, Write};
use std::time::Instant;

use inline_colorization::*;

use terminal_link::*;

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

        let underscores_num = if inverse {
            String::from("?")
        } else {
            vec!['_'; prompt.word.chars().count()]
                .into_iter()
                .collect::<String>()
        };

        let print_language = if inverse { "eng" } else { &language };

        let non_english = format!(
            "{style_bold}{}{style_reset}{}{style_bold}{}{style_reset} {}",
            (print_language.to_uppercase() + ": "),
            prompt.first_half,
            underscores_num,
            prompt.second_half
        );

        if inverse {
            println!(
                "{color_black}{bg_bright_white}{}{}{}{color_reset}{bg_reset}",
                &language.to_uppercase(),
                &": ".to_string(),
                &sentence.get_translation().unwrap().text
            );
            println!("{}", &non_english);
        } else {
            print!(
                "{color_black}{bg_bright_white}{style_bold}{}:{style_reset}",
                // {color_black}{bg_bright_white}{}{style_bold}{}{style_reset}{color_black}{bg_bright_white} {}{color_reset}{bg_reset}"
                print_language.to_uppercase()
            );

            for word in prompt.first_half.split(" ") {
                print!(
                    "{color_black}{bg_bright_white} {}{style_reset}",
                    Link::new(
                        &word,
                        &generate_url(
                            &word.trim_matches(|c| char::is_ascii_punctuation(&c)),
                            &language
                        )
                    )
                )
            }

            print!("{color_black}{bg_bright_white}{underscores_num}{style_reset}");

            for word in prompt.second_half.split(" ") {
                print!(
                    "{color_black}{bg_bright_white} {}{style_reset}",
                    Link::new(
                        &word,
                        &generate_url(
                            &word.trim_matches(|c| char::is_ascii_punctuation(&c)),
                            &language
                        )
                    )
                )
            }

            println!("\n{style_bold}{}{style_reset} {}", "ENG:", sentence.text);
        }

        let mut guess = String::new();

        print!("> ");
        read_into(&mut guess);

        let levenshtein_distance = levenshtein(
            &remove_punctuation(&guess.trim().to_lowercase()),
            &prompt.word.to_lowercase().trim(),
        );

        if levenshtein_distance == 0 {
            correct += 1;
            println!(
                "{}, {color_white}{bg_green}{}{color_reset}{bg_reset}",
                "Correct",
                Link::new(
                    &prompt.word.to_lowercase().trim(),
                    &generate_url(&prompt.word.to_lowercase().trim(), &language)
                )
            );
        } else if levenshtein_distance < DISTANCE_FOR_CLOSE as usize {
            println!(
                "Close, {style_bold}{color_bright_white}{bg_yellow}{}{bg_reset}{color_reset}{style_reset}.",
                Link::new(
                    &prompt.word.to_lowercase().trim(),
                    &generate_url(&prompt.word.to_lowercase().trim(), &language)
                )
            );
        } else {
            println!(
                "Wrong, {style_bold}{color_bright_white}{bg_red}{}{bg_reset}{color_reset}{style_reset}.",
                Link::new(
                    &prompt.word.to_lowercase().trim(),
                    &generate_url(&prompt.word.to_lowercase().trim(), &language)
                )
            );
        }
        println!();

        // Old lookup logic

        // loop {
        //     let mut lookup = String::new();
        //     println!("{} {}", "Lookup a word?", "[enter word or ignore]");
        //     print!("> ");
        //     read_into(&mut lookup);

        //     if lookup.trim().is_empty() {
        //         break;
        //     } else {
        //         wiktionary_try_open(lookup, &language);
        //     }
        // }
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
