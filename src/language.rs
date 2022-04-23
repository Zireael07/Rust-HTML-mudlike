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

//TODO: dialogue (text-reply)

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
    pub map_q: HashMap<String, Vec<String>>,
    pub nouns: Vec<String>,
    pub constraints: HashMap<String, Vec<String>>,
    pub substitutions: HashMap<String, String>,
}

impl Markov {
    pub fn new() -> Markov {
        Markov {
            map: HashMap::new(),
            map_q: HashMap::new(), //for questions
            nouns: Vec::new(),
            constraints: HashMap::new(),
            substitutions: HashMap::new(),
        }
    }

    pub fn parse(&mut self, sentence: &str, num: usize, question: bool) {
        //TODO: maybe pre-tag the words so that it knows whether they're in a NP, VP, AP etc.

        //FIXME: commas should be ignored - no meaning in Toki Pona per https://github.com/ae-dschorsaanjo/lipu-lili-pi-toki-pona/blob/master/grammar.md
        
        //just basic n-gram parsing
        let words = sentence.split(" ").collect::<Vec<&str>>();
        let word_count = words.len();
    
        //FIXME: doesn't grab things of size equal to num (e.g. 2 word phrases for num 2)

        for n in 0..word_count {
            if n + num < word_count {
                //slice to vector
                let key_vec = &words[n..n+num].to_vec();
                let key = key_vec.join(" ");
                //let key = format!("{} {}", words[n], words[n + 1]);
                let value = words[n + num];
                //log!("Parsed to: {} {} ", key, value);
                self.insert(key, value.to_string(), question)
            }
        }

        //TODO: make the Markov chain bi-directional (go from last to first)
    }

    fn insert(&mut self, key: String, value: String, question: bool) {
        //TODO: is it possible to make this prettier?
        if question {
            if self.map_q.contains_key(&key) {
                let current_value = self.map_q.get_mut(&key).unwrap();
                if !current_value.contains(&value) {
                    current_value.push(value);
                }
            } else {
                self.map_q.insert(key, vec!(value));
            }
        }
        else {
            if self.map.contains_key(&key) {
                let current_value = self.map.get_mut(&key).unwrap();
                if !current_value.contains(&value) {
                    current_value.push(value);
                }
            } else {
                self.map.insert(key, vec!(value));
            }
        }
    }

    //graph search algo
    //it can switch between the best-first search and breadth-first search 
    // k is the number of tokens/nodes taken under consideration at each step (also known as beam width)
    // if beam width is very high, it becomes BFS (breadth-first search)
    fn beam_search(&self, k: usize, keys: &Vec<&String>, nouns: &Vec<String>) -> Vec<Path> {
        //data struct (results so far, number of nodes traversed)
        let mut paths_so_far: Vec<Path> = Vec::new(); 
        paths_so_far.push(Path{nodes:Vec::new(), dist:0});

        let mut step = 0;
        let steps_num = 4;
        for mut step in 0..steps_num {
            //get all the options at each step
            let mut path_candidates: Vec<Path> = Vec::new();
            //follow each path
            for i in 0..paths_so_far.len() {
                // options depend on previous steps...
                let mut options: Vec<&String> = Vec::new();
                //match would've been nicer but doesn't want to compile
                //match step {
                if step == 0 { 
                    //step 0: all the nouns
                    options = valid_initial_keys(keys, nouns); 
                } else { //_ => { 
                    //others - get options based on sentence so far
                    //let sentence: Vec<&str> = paths_so_far[i].nodes.iter().map(|s| s as &str).collect(); //collect::<Vec<&str>>();
                    let sentence = format!("{}", paths_so_far[i].nodes.join(" "));
                    log!("sentence: {}", sentence);
                    //wants &str
                    let key = next_key(&sentence);
                    let m = self.map.get(&key);
                    //let m = self.map.get(&paths_so_far[i].nodes[step-1]);
                    match m {
                        Some(values) => {
                            //values are a Vec<String> (see hashmap definitions at the top)
                            //need to convert to Vec<&String>
                            // based on https://stackoverflow.com/questions/33216514/how-do-i-convert-a-vecstring-to-vecstr?noredirect=1&lq=1
                            options = values.iter().map(|s| s).collect();
                        },
                        None => {},
                    }
                }

                // now expand k candidates
                for key in &options {
                    // we assume all distances are 1 (all phrases are equally likely as long as they're valid)
                    let mut p = &mut paths_so_far[i];
                    ///p.nodes.push(key.to_string());
                    //concatenate vecs instead of pushing
                    //p.nodes.clone().append(&mut vec![key.to_string()]);
                    //https://stackoverflow.com/a/69578632
                    let new_nodes = [p.nodes.clone().as_slice(), &[key.to_string()]].concat();
                    //can't find a way to avoid clone here
                    let new_path = Path{nodes:new_nodes, dist:p.dist + 1};
                    path_candidates.push(new_path);
                }
            }
            //sort paths by length
            path_candidates.sort_by(|a,b| a.dist.cmp(&b.dist));
            log!("Path candidates: {:?}", path_candidates);
            //keep best k paths
            path_candidates.truncate(k);
            log!("Kept paths: {:?}", path_candidates);
            paths_so_far = path_candidates;

            //increment counter
            step = step +1;
        }

        return paths_so_far
    }


    // generate text
    pub fn generate_paragraph(&mut self, max:i32) -> String {
        let data = self.generate_sentence_wrapper(false, "".to_string());
        let mut data2 = self.generate_sentence_wrapper(false, data.1);
        let mut text = format!("{} {}", data.0, data2.0);
        for i in 0..max-2 {
            data2 = self.generate_sentence_wrapper(false, data2.1);
            let sn = format!("{}", data2.0);
            text = format!("{} {}", text, sn);
        }

        return text
    }

    pub fn generate_sentence_wrapper(&mut self, question:bool, given_topic: String) -> (String, String) {
        let mut keys = self.map.keys().collect::<Vec<&String>>();
        if question {
            keys = self.map_q.keys().collect::<Vec<&String>>();
        }
    
        if given_topic == "".to_string() {
            //needs to be a function of self because we need self.map
            let beam = self.beam_search(3, &keys, &self.nouns);
            log!("Beam search: {:?}", beam);

            let mut key = initial_key(&keys, &self.nouns);
            //the first word in the initial key gets assigned as topic
            let topic = key.split(" ").collect::<Vec<&str>>()[0].to_string();
            //log!("Topic: {}", topic);
            let mut topics = Vec::new();
            topics.push(topic.clone());
            let mut sentence = key.clone();

            (self.generate_sentence(sentence, key, topics, question), topic)
        }
        else {
            log!("Generating a sentence for a given topic: {} ", given_topic);
            //let mut key = key_with(&keys, &self.nouns, &given_topic);
            let mut key = "".to_string();
            //hack
            if given_topic == "esun" {
                key = key_with(&keys, &vec!["sina".to_string()], &vec![&"sina".to_string(), &"wile".to_string()]);
            }
            else if given_topic == "(sina" {
                key = key_with(&keys, &vec!["(sina".to_string()], &vec![&given_topic]);
            }
            else {
                key = key_with(&keys, &self.nouns, &vec![&given_topic]);
            }


            //let topic = given_topic;
            let mut topics = Vec::new();
            topics.push(given_topic.clone());
            let mut sentence = key.clone();

            (self.generate_sentence(sentence, key, topics, question), given_topic)
        }

    }

    //key function in this module
    pub fn generate_sentence(&mut self, mut sentence: String, mut key: String, mut topics: Vec<String>, question: bool) -> String {
        let mut rng = rand::thread_rng();
        loop {
            //match left-hand side must be a variable!!!
            let mut m = self.map.get(&key);
            if question { m =  self.map_q.get(&key) }

            match m {
                Some(values) => {
                    let mut valid = values.clone();

                    //TODO: can we forbid selections which lead to empty valid list in the next step? 
                    // check beam search algo: https://towardsdatascience.com/the-power-of-constrained-language-models-cf63b65a035d

                    //forbid anu and seme unless we specifically asked for a question
                    if !question {
                        // sina tan (you from) only happens in questions
                        self.constraints.insert("sina".to_string(), vec!["tan".to_string()]);
                        valid.retain(|v| v.ne("anu") && v.ne("seme") && v.ne("seme?"));
                    }
                    //else ensure we only ask questions
                    else {
                        //ensure we have "sina" as an option
                        self.nouns.push("sina".to_string());
                        //this constraint no longer applies
                        self.constraints.remove("sina");
                    }

                    for topic in &topics {
                        //do we have a constraint set by a topic?
                        match self.constraints.get(topic) {
                            Some(constr) => {
                                //log!("We have a constraint, {:?}", constr);
                                //filter values by topic
                                valid.retain(|v| !constr.contains(&v));
                                //log!("Valid: {:?}", valid);
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
                        //log!("Topics: {:?}", topics);
                    }
                    if last_word == "e" {
                        topics.push(value.to_string());
                        //log!("Topics: {:?}", topics);
                    }

                    sentence = format!("{} {}", sentence, value);
    
                    key = next_key(&sentence);
                }
                None => break
            }
        }
    
        sentence
    }

    //prettify the generated text
    pub fn display_paragraph(&mut self, max:i32) -> String {
        let text = self.generate_paragraph(max);

        return string_sub_multi(text, &self.substitutions);
    }

    pub fn display_sentence(&mut self, question:bool, given_topic: String) -> String {
        let text = self.generate_sentence_wrapper(question, given_topic);

        return string_sub_multi(text.0, &self.substitutions);
    }

    //NaNoGenMo special
    pub fn display_block(&mut self, max: i32, max_s: i32) -> String {
        let mut text = "".to_string();
        for i in 0..max {
            let p = self.display_paragraph(max_s);
            text = format!("{} {}", text, p);
        }
        let s = self.display_sentence(true, "".to_string());
        text = format!("{} \n {}", text, s);
        return text;
    }

    pub fn display_novel(&mut self) -> String {
        let mut rng = rand::thread_rng();
        let mut text = "".to_string();
        //50,000 words, 30 per block means we need 1666,6 according to my calculations
        for i in 0..1667 {
            let max = rng.gen_range(2,3);
            let max_s = rng.gen_range(3,4);
            let b = self.display_block(max, max_s);
            text = format!("{} {}", text, b);
        }
        return text;
    }


    // pub fn debug(self) {

    // }

} //end impl

//-------------------------------------------
#[derive(Debug)]
struct Path {
    nodes: Vec<String>, //because in our case the "path" is a sentence
    dist: usize
}

//fn noun_keys(valid_keys, keys, nouns: &Vec<String) {
//     //the initial key HAS to contain a noun
//     //let valid_keys = keys.iter().filter(|&k| for n in self.nouns { k.contains(&n); } )
//     let mut valid_keys: Vec<&String> = Vec::new();
//     for k in keys {
//         let words = k.split(" ").collect::<Vec<&str>>();
//         for n in nouns {
//             if words[0] == n.as_str() && words[1].ne("e") {
//             //if k.contains(n.as_str()) {
//                 valid_keys.push(k);
//             }
//         }
//     }
// }

fn key_with(keys:&Vec<&String>, nouns: &Vec<String>, filter: &Vec<&String>) -> String {
    let mut rng = rand::thread_rng();
    //the initial key HAS to contain a noun and fit our filter
    let mut valid_keys: Vec<&String> = Vec::new();
    for k in keys {
        let words = k.split(" ").collect::<Vec<&str>>();
        //log!("Words: {:?}", words);
        if filter.len() == 1 {
            for n in nouns {
                if words[0] == n.as_str() && words[1].ne("e") && words[0] == filter[0] {
                //if k.contains(n.as_str()) {
                    valid_keys.push(k);
                }
            }
        } else {
            if words[0] == filter[0] && words[1] == filter[1] {
                valid_keys.push(k);
            }
        }

    }

    log!("Keys found for topic: {:?} {:?}", filter, valid_keys);
    //only retain those that match filter
    ///valid_keys.retain(|v| v[0] == filter.as_str());

    //random pick
    return valid_keys.choose(&mut rng).expect("could not get random value").to_string();
}

fn valid_initial_keys<'a>(keys: &Vec<&'a String>, nouns: &'a Vec<String>) -> Vec<&'a String> {
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
    return valid_keys;
}


// this selects a single initial key
fn initial_key(keys: &Vec<&String>, nouns: &Vec<String>) -> String {
    let mut rng = rand::thread_rng();
    //return keys.choose(&mut rng).expect("could not get random value").to_string();

    //the initial key HAS to contain a noun
    let valid_keys = valid_initial_keys(keys, nouns);

    return valid_keys.choose(&mut rng).expect("could not get random value").to_string();
}

fn next_key(sentence: &str) -> String {
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

// based on: https://play.rust-lang.org/?version=stable&mode=debug&edition=2018&gist=2ffced29e8649b5d5e67d1617eeb9af7
// requires Rust 1.52.0+ due to use of str.split_once()

//Find a substring match.
//Push your input into the result up to the start of the match.
//Push the replacement value into the result.
//Update the input to only include what's past the match.
//Repeat until there's no more input or match.

pub fn string_sub_multi(inp: String, subs: &HashMap<String, String>) -> String
{
    //AsRef<str> we express that we want to accept all references that can be converted to &str as an argument.
    let mut current: &str = inp.as_ref();
    let mut result = Vec::with_capacity(32);
    
    while !current.is_empty() {
        let mut found_match = false;
        for (key, value) in subs.iter() {
            let k: &str = key.as_ref();
        //for (sub_find, sub_replace) in subs.iter() {
            if let Some((before, after)) = current.split_once(k) { //key.as_ref()
                found_match = true;
                result.push(before);
                result.push(value.as_ref());
                current = after;
            }
        }
        if !found_match {
            result.push(current);
            break;
        }
    }
    result.join("")
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

    lang.constraints.insert("esun".to_string(), vec!["tawa".to_string()]);

    //debug
    for (key, value) in &lang.constraints {
        log!("{}: {:?}", key, value)
    }

    //markers
    lang.substitutions.insert("<cry>".to_string(), "(pana e telo lukin)".to_string());
}