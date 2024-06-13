// logic which handles parsing a raw JSON from tatoeba into sentences

use rand::{thread_rng, Rng};
use serde::{Deserialize, Serialize};

const NON_SPACED: [&str; 12] = [
    "cmn", "lzh", "hak", "cjy", "nan", "hsn", "gan", "jpn", "tha", "khm", "lao", "mya",
];

// represents the entire JSON response from Tatoeba. results is the sentences found.
#[derive(Deserialize, Serialize)]
pub struct Json {
    pub results: Vec<Sentence>,
}

// represents a sentence. id is the tatoeba id of the sentence, not used anywhere currently
#[derive(Deserialize, Serialize, Clone)]
pub struct Sentence {
    id: i32,
    pub text: String,
    pub translations: Vec<Vec<Translation>>,
}

// represents a translation. id is the tatoeba id of the translation
#[derive(Deserialize, Serialize, Clone)]
pub struct Translation {
    id: i32,
    pub text: String,
}

#[derive(Clone)]
pub struct Prompt {
    pub first_half: String,
    pub word: String,
    pub second_half: String,
}

impl Sentence {
    // get the sentence's translation
    // sometimes translations.0 will be blank
    pub fn get_translation(&self) -> Option<&Translation> {
        if let Some(t) = self.translations.get(0).unwrap().get(0) {
            Some(t)
        } else {
            self.translations.get(1).unwrap().get(0)
        }
    }

    // split string into vec of words, depends on whether the language uses spaces or not (e.g.
    // japanese is not spaced)
    pub fn as_words(&self, language: &str, inverse: bool) -> Vec<String> {
        let translation = if inverse {
            &self.text
        } else {
            &self.get_translation().unwrap().text
        };

        let words: Vec<String> = if NON_SPACED.contains(&language) {
            let char_strings = translation.trim().chars().map(|x| x.to_string());
            char_strings.collect::<Vec<String>>()
        } else {
            translation
                .trim()
                .split_inclusive(' ')
                .map(|x| x.to_string())
                .collect::<Vec<String>>()
        };

        words
    }

    // splits a sentence into a prompt consisting of three parts
    pub fn generate_prompt(&self, language: &str, inverse: bool) -> Prompt {
        let words: Vec<String> = self.as_words(language, inverse);
        let halved = words.split_at(thread_rng().gen_range(0..words.len()));

        let word = remove_punctuation(&halved.1[0]);

        Prompt {
            first_half: halved.0.join(""),
            word,
            second_half: halved.1[1..].join(""),
        }
    }
}

// language: the language to request from tatoeba
pub fn generate_sentences(language: &str) -> std::result::Result<Vec<Sentence>, minreq::Error> {
    // where the initial request happens
    let mut sentences = sentences_http_request(language)?;

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

// language: the language to request from tatoeba
pub fn sentences_http_request(language: &str) -> Result<Vec<Sentence>, minreq::Error> {
    let request = format!("https://tatoeba.org/en/api_v0/search?from=eng&orphans=no&sort=random&to={language}&unapproved=no");
    let response = minreq::get(request).send()?;

    let resp_str = response.as_str()?;

    let sentences = parse(resp_str).unwrap();
    Ok(sentences)
}

// converts a serde error into a string
pub fn convert_error(err: serde_json::Error) -> String {
    format!(
        "{:#?} error thrown by serde at {}:{}.",
        err.classify(),
        err.line(),
        err.column()
    )
}

// parse plaintext JSON response string into a Vec of Sentences results: the JSON
pub fn parse(results: &str) -> Result<Vec<Sentence>, String> {
    let sentences: Json = serde_json::from_str(results).map_err(convert_error)?;
    Ok(sentences.results)
}

pub fn remove_punctuation(word: &String) -> String {
    word.replace(
        &[
            '(', ')', ',', '.', ';', ':', '?', '¿', '!', '¡', '"', '«', '»', '。', ' ',
        ][..],
        "",
    )
}
