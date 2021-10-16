//Since we need to store some state, we are going to write our own code
//instead of relying on some preexisting markov implementation

//ref: https://bookowl.github.io/2016/12/08/Rusty-(Markov)-Chains/
//ref: https://blakewilliams.me/posts/generating-arbitrary-text-with-markov-chains-in-rust

use super::log;

use std::collections::HashMap;

//RNG
use rand::{Rng, thread_rng};
use rand::prelude::SliceRandom;

struct SentenceState {
    pub topic: String,
}

pub struct Markov {
    pub map: HashMap<String, Vec<String>>,
    pub nouns: Vec<String>
}

impl Markov {
    pub fn new() -> Markov {
        Markov {
            map: HashMap::new(),
            nouns: Vec::new()
        }
    }

    pub fn parse(&mut self, sentence: &str, num: usize) {
        let words = sentence.split(" ").collect::<Vec<&str>>();
        let word_count = words.len();
    
        for n in 0..word_count {
            if n + num < word_count {
                //slice to vector
                let key_vec = &words[n..n+num].to_vec();
                let key = key_vec.join(" ");
                //let key = format!("{} {}", words[n], words[n + 1]);
                let value = words[n + num];
                //log!("Parsed to: {} {} ", key, value);
                self.insert(key, value.to_string())
            }
        }
    }

    fn insert(&mut self, key: String, value: String) {
        if self.map.contains_key(&key) {
            let current_value = self.map.get_mut(&key).unwrap();
            current_value.push(value);
        } else {
            self.map.insert(key, vec!(value));
        }
    }

    //key function in this module
    pub fn generate_sentence(self) -> String {
        let mut rng = rand::thread_rng();
        let keys = self.map.keys().collect::<Vec<&String>>();
    
        let mut key = initial_key(keys, self.nouns);
        let mut sentence = key.clone();
    
        loop {
            match self.map.get(&key) {
                Some(values) => {
                    let value = values.choose(&mut rng).expect("could not get value");
                    sentence = format!("{} {}", sentence, value);
    
                    key = next_key(&sentence, value);
                }
                None => break
            }
        }
    
        sentence
    }

    // pub fn debug(self) {

    // }

}

fn initial_key(keys: Vec<&String>, nouns: Vec<String>) -> String {
    let mut rng = rand::thread_rng();
    //return keys.choose(&mut rng).expect("could not get random value").to_string();

    //the initial key HAS to contain a noun
    //let valid_keys = keys.iter().filter(|&k| for n in self.nouns { k.contains(&n); } )
    let mut valid_keys: Vec<&String> = Vec::new();
    for k in keys {
        let words = k.split(" ").collect::<Vec<&str>>();
        for n in &nouns {
            if words[0] == n.as_str() {
            //if k.contains(n.as_str()) {
                valid_keys.push(k);
            }
        }
    }


    return valid_keys.choose(&mut rng).expect("could not get random value").to_string();
}

fn next_key(sentence: &str, value: &str) -> String {
    //get last 3 words in the sentence
    let words = sentence.split(" ").collect::<Vec<&str>>();
    let word_count = words.len();
    let last_words = &words[(word_count-3)..];
    log!("Next key for: {:?} ", last_words);
    let key = last_words.join(" ");
    //let last_word = key.split(" ").last().expect("could not get last word");
    return key;
    //format!("{} {}", last_word, value)
}

pub fn tokenize(sentence: String) -> Vec<String> {
    //split by specified characters
    let words = sentence.split(|c| (c == ' ') || (c == ';') || (c == ',' ) || (c == '.') || (c == ')') || (c == '(')).collect::<Vec<&str>>();
    
    //we need a Vec<String> because we'll be using it on JS side...
    let mut v = Vec::new();
    for w in words {
        v.push(w.to_string());
    }
    return v;

    //return format!("{}", words);
}


pub fn add_text(lang: &mut Markov){
    //some Toki Pona sentences
    lang.parse("ona li suli.",2);
    lang.parse("kili li pona.",2);
    lang.parse("sina li suli.",2);
    lang.parse("soweli lili li suwi.",2);
    lang.parse("mama mi li pona.",3);
    lang.parse("jan utala li wawa.",3);
    lang.parse("jan lili mi li suwi.",3);
    lang.parse("soweli lili li wawa ala.",3);
    lang.parse("meli mi li pona.",3);
    lang.parse("mije sina li suli.",3);
    lang.parse("soweli ale li pona.",3);
    lang.parse("kili li moku suli.",3);
    lang.parse("jan lili li (pana e telo lukin).",3);
    lang.parse("ona li lukin e lipu.",3);
    lang.parse("soweli ike li utala e meli.",3);
    lang.parse("jan utala li moku e kili suli.",3);
    lang.parse("soweli lili li moku e telo.",3);
    lang.parse("mi telo e ijo suli.",3);
    lang.parse("jan wawa li pali e tomo.",3);
    lang.parse("jan pali li telo e kasi.",3);
    lang.parse("jan wawa li jo e kiwen suli.",3);
    lang.parse("waso lili li moku e pipi.",3);
    lang.parse("meli li toki e soweli, e waso.",3);
    lang.parse("jan pali li pona e ilo, li lukin e lipu.",3);
    lang.parse("jan pali li pana e moku pona.",3);


    // for s in text {
    //     lang.parse(s);
    // }

    //some additional info
    lang.nouns.push("jan".to_string());
    lang.nouns.push("kili".to_string());
    lang.nouns.push("mama".to_string());
    lang.nouns.push("mije".to_string());
    lang.nouns.push("meli".to_string());
    lang.nouns.push("waso".to_string());
    lang.nouns.push("soweli".to_string());
    //not quite but will do for now
    lang.nouns.push("mi".to_string());
    lang.nouns.push("sina".to_string());
}