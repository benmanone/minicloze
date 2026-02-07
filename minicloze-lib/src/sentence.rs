// logic which handles parsing a raw JSON from tatoeba into sentences

use rand::{thread_rng, Rng};
use reqwest::Error;
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
#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct Sentence {
    id: i32,
    pub text: String,
    pub translations: Vec<Vec<Translation>>,
}

// represents a translation. id is the tatoeba id of the translation
#[derive(Deserialize, Serialize, Clone, Debug)]
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
        self.translations
            .first()
            .unwrap()
            .first()
            .map_or_else(|| self.translations.get(1).unwrap().first(), Some)
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
                .map(std::string::ToString::to_string)
                .collect::<Vec<String>>()
        };

        words
    }

    // splits a sentence into a prompt consisting of three parts
    pub fn generate_prompt(&self, language: &str, inverse: bool) -> Prompt {
        let words: Vec<String> = self.as_words(language, inverse);

        // Find indices of words that have actual content after punctuation removal
        // (filters out whitespace-only "words" from double spaces, etc.)
        let valid_indices: Vec<usize> = words
            .iter()
            .enumerate()
            .filter(|(_, w)| !remove_punctuation(w).trim().is_empty())
            .map(|(i, _)| i)
            .collect();

        // Pick from valid indices, or fallback to any index if none are valid
        let split_index = if valid_indices.is_empty() {
            thread_rng().gen_range(0..words.len())
        } else {
            valid_indices[thread_rng().gen_range(0..valid_indices.len())]
        };

        let halved = words.split_at(split_index);

        Prompt {
            first_half: halved.0.join(""),
            word: halved.1[0].clone(), // keep raw word with punctuation
            second_half: halved.1[1..].join(""),
        }
    }
}

// language: the language to request from tatoeba
pub async fn generate_sentences(language: &str) -> std::result::Result<Vec<Sentence>, Error> {
    // where the initial request happens
    let mut sentences = sentences_http_request(language).await?;

    let len = sentences.len();

    // makes sure we always get 10 sentences
    if len != 10 {
        let difference = 10 - len;
        // makes more requests if required
        let mut sentences_difference = sentences_http_request(language)
            .await?
            .into_iter()
            .take(difference)
            .collect::<Vec<_>>();

        sentences.append(&mut sentences_difference);
    }
    Ok(sentences)
}

// language: the language to request from tatoeba
pub async fn sentences_http_request(language: &str) -> Result<Vec<Sentence>, Error> {
    let request = format!("https://tatoeba.org/en/api_v0/search?from=eng&orphans=no&sort=random&to={language}&unapproved=no");
    let response = reqwest::get(request).await?.text().await?;

    let resp_str = response.as_str();

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

pub fn remove_punctuation(word: &str) -> String {
    // Includes French whitespace: NBSP (U+00A0), thin space (U+2009), narrow no-break space (U+202F)
    word.replace(
        &[
            '(', ')', ',', '.', ';', ':', '?', '¿', '!', '¡', '"', '«', '»', '。', ' ',
            '\u{00A0}', '\u{2009}', '\u{202F}',
        ][..],
        "",
    )
}
