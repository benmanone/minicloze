use serde::{Deserialize, Serialize};

// represents the entire JSON response. results is the sentences found.
#[derive(Deserialize, Serialize)]
pub struct Json {
    pub results: Vec<Sentence>,
}

// represents a sentence. id is the tatoeba id of the sentence, not used anywhere currently
#[derive(Deserialize, Serialize)]
pub struct Sentence {
    id: i32,
    pub text: String,
    pub translations: Vec<Vec<Translation>>,
}

// represents a translation. id is the tatoeba id of the translation
#[derive(Deserialize, Serialize)]
pub struct Translation {
    id: i32,
    pub text: String,
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

    let rep_string = response.as_str()?;

    let sentences = parse(rep_string).unwrap();
    Ok(sentences)
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

// parse plaintext JSON response string into a Vec of Sentences results: the JSON
pub fn parse(results: &str) -> Result<Vec<Sentence>, String> {
    let sentences: Json = serde_json::from_str(results).map_err(convert_error)?;
    Ok(sentences.results)
}
