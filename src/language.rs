//Since we need to store some state, we are going to write our own code
//instead of relying on some preexisting markov implementation

//ref: https://bookowl.github.io/2016/12/08/Rusty-(Markov)-Chains/
//ref: https://blakewilliams.me/posts/generating-arbitrary-text-with-markov-chains-in-rust

//ref: https://kevingal.com/blog/toki-poetry.html
//ref: https://github.com/gabrielebarbieri/markovconstraints (the idea that we can just exclude some options)

use super::log;

use std::collections::HashMap;

//RNG
use rand::{Rng, thread_rng};
use rand::prelude::SliceRandom;

struct SentenceState {
    pub topic: String,
}

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SentenceType {
    Sentence = 0,
    Question = 1,
}

pub struct Markov {
    pub map: HashMap<String, Vec<String>>,
    pub nouns: Vec<String>,
    pub constraints: HashMap<String, Vec<String>>,
}

impl Markov {
    pub fn new() -> Markov {
        Markov {
            map: HashMap::new(),
            nouns: Vec::new(),
            constraints: HashMap::new(),
        }
    }

    pub fn parse(&mut self, sentence: &str, num: usize) {
        //TODO: treat stuff in brackets in a special way (as a single block)
        // OR some other way to do collocations (markers that are expanded?)

        //TODO: maybe pre-tag the words so that it knows whether they're in a NP, VP, AP etc.

        //FIXME: commas should be ignored - no meaning in Toki Pona per https://github.com/ae-dschorsaanjo/lipu-lili-pi-toki-pona/blob/master/grammar.md
        
        //just basic n-gram parsing
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

        //TODO: make the Markov chain bi-directional (go from last to first)
    }

    fn insert(&mut self, key: String, value: String) {
        if self.map.contains_key(&key) {
            let current_value = self.map.get_mut(&key).unwrap();
            //FIXME: don't add values that are already in
            current_value.push(value);
        } else {
            self.map.insert(key, vec!(value));
        }
    }

    //key function in this module
    pub fn generate_sentence(&self) -> String {
        let mut rng = rand::thread_rng();
        let keys = self.map.keys().collect::<Vec<&String>>();
    
        let mut key = initial_key(keys, &self.nouns);
        //the first word in the initial key gets assigned as topic
        let topic = key.split(" ").collect::<Vec<&str>>()[0].to_string();
        //log!("Topic: {}", topic);
        let mut topics = Vec::new();
        topics.push(topic);
        let mut sentence = key.clone();
    
        loop {
            match self.map.get(&key) {
                Some(values) => {
                    let mut valid = values.clone();

                    //forbid anu and seme unless we specifically asked for a question
                    valid.retain(|v| v.ne("anu") && v.ne("seme") && v.ne("seme?"));

                    for topic in &topics {
                        //do we have a constraint set by a topic?
                        match self.constraints.get(topic) {
                            Some(constr) => {
                                log!("We have a constraint, {:?}", constr);
                                //filter values by topic
                                valid.retain(|v| !constr.contains(&v));
                                log!("Valid: {:?}", valid);
                            }
                            None => {}
                        }
                    }

                    //handle case where the valid list is empty!!!
                    if valid.len() < 1 {
                        log!("No valid continuations detected, breaking...");
                        break
                    }

                    let value = valid.choose(&mut rng).expect("could not get value");
                    //let value = values.choose(&mut rng).expect("could not get value");

                    //if the last word in a sentence was li (pred marker), add the value to topic
                    //get last 2 words in the sentence
                    let words = sentence.split(" ").collect::<Vec<&str>>();
                    let word_count = words.len();
                    let last_word = words[(word_count-1)];
                    if last_word == "li" {
                        topics.push(value.to_string());
                        log!("Topics: {:?}", topics);
                    }
                    if last_word == "e" {
                        topics.push(value.to_string());
                        log!("Topics: {:?}", topics);
                    }

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

fn initial_key(keys: Vec<&String>, nouns: &Vec<String>) -> String {
    let mut rng = rand::thread_rng();
    //return keys.choose(&mut rng).expect("could not get random value").to_string();

    //the initial key HAS to contain a noun
    //let valid_keys = keys.iter().filter(|&k| for n in self.nouns { k.contains(&n); } )
    let mut valid_keys: Vec<&String> = Vec::new();
    for k in keys {
        let words = k.split(" ").collect::<Vec<&str>>();
        for n in nouns {
            if words[0] == n.as_str() && words[1].ne("e") {
            //if k.contains(n.as_str()) {
                valid_keys.push(k);
            }
        }
    }


    return valid_keys.choose(&mut rng).expect("could not get random value").to_string();
}

fn next_key(sentence: &str, value: &str) -> String {
    //get last 2 words in the sentence
    let words = sentence.split(" ").collect::<Vec<&str>>();
    let word_count = words.len();
    let last_words = &words[(word_count-2)..];
    log!("Next key for: {:?} ", last_words);
    let key = last_words.join(" ");

    //let last_word = key.split(" ").last().expect("could not get last word");
    //format!("{} {}", last_word, value)
    return key;
}

pub fn tokenize(sentence: String) -> Vec<String> {
    //split by specified characters
    let words = sentence.split(|c| (c == ' ') || (c == ';') || (c == ',' ) || (c == '.') || (c == ')') || (c == '(') || (c == '?') || (c== '!')).collect::<Vec<&str>>();
    
    //we need a Vec<String> because we'll be using it on JS side...
    let mut v = Vec::new();
    for w in words {
        v.push(w.to_string());
    }
    return v;

    //return format!("{}", words);
}


pub fn setup(lang: &mut Markov){
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

    //constraints
    //semantic constraints
    lang.constraints.insert("kili".to_string(), vec!["telo".to_string(), "e".to_string()]);
    lang.constraints.insert("waso".to_string(), vec!["(pona".to_string(), "sona".to_string(), "(pana".to_string()]);
    lang.constraints.insert("moli".to_string(), vec!["mani".to_string()]);
    //forbid li after e (predicate after object marker)
    lang.constraints.insert("e".to_string(), vec!["li".to_string()]);
    // sina tan (you from) only happens in questions
    lang.constraints.insert("sina".to_string(), vec!["tan".to_string()]);

    //debug
    for (key, value) in &lang.constraints {
        log!("{}: {:?}", key, value)
    }
}