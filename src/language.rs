//Since we need to store some state, we are going to write our own code
//instead of relying on some preexisting markov implementation

//ref: https://bookowl.github.io/2016/12/08/Rusty-(Markov)-Chains/
//ref: https://blakewilliams.me/posts/generating-arbitrary-text-with-markov-chains-in-rust

use std::collections::HashMap;

//RNG
use rand::Rng;

struct SentenceState {
    pub topic: String,
}

pub struct Markov {
    map: HashMap<String, Vec<String>>
}

impl Markov {
    pub fn new() -> Markov {
        Markov {
            map: HashMap::new()
        }
    }

    pub fn parse(&mut self, sentence: &str, num: int) {
        let words = sentence.split(" ").collect::<Vec<&str>>();
        let word_count = words.len();
    
        for n in 0..word_count {
            if n + num < word_count {
                let key_vec = word_count[n, n+num-1].to_vec();
                let key = key_vec.join(" ");
                //let key = format!("{} {}", words[n], words[n + 1]);
                let value = words[n + num];
    
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

    pub fn generate_sentence(self) -> String {
        let mut rng = thread_rng();
        let keys = self.map.keys().collect::<Vec<&String>>();
    
        let mut key = rng.choose(&keys).expect("could not get random value").to_string();
        let mut sentence = key.clone();
    
        loop {
            match self.map.get(&key) {
                Some(values) => {
                    let value = rng.choose(values).expect("could not get value");
                    sentence = format!("{} {}", sentence, value);
    
                    key = next_key(&key, value);
                }
                None => break
            }
        }
    
        sentence
    }
}

fn next_key(key: &str, value: &str) -> String {
    let last_word = key.split(" ").last().expect("could not get last word");
    format!("{} {}", last_word, value)
}