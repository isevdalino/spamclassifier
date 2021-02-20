use std::collections::HashMap;
use std::fs::File;
use std::io;
use std::io::{BufRead,BufReader};
use serde::{Deserialize, Serialize};
use serde_json::{from_reader, to_writer};
use unicode_segmentation::UnicodeSegmentation;

const HAM : &str= "ham";
const SPAM : &str= "spam";
const TAB : char= '\t';
const INITIAL_RATING : f64 = 0.5;
const DATASET_FILE_FORMAT_INVALID : &str = "The specified dataset file is in invalid format!";
const ZEROES_REVERTER : f64 = 10000.0;

#[derive(Debug, Default, Serialize, Deserialize)]
struct Counter {
    ham: u32,
    spam: u32,
}

#[derive(Default, Debug, Serialize, Deserialize)]
pub struct BayesianSpamClassifier {
    token_table: HashMap<String, Counter>,
}

impl BayesianSpamClassifier {

    pub fn new() -> Self {
        Default::default()
    }

    pub fn new_from_pre_trained(file: &mut File) -> Result<Self, io::Error> {
        let pre_trained_model = from_reader(file)?;
        Ok(pre_trained_model)
    }

    fn split_string_into_list_of_words(&self,msg: &str) -> Vec<String> {
        msg.unicode_words().map(|word| word.to_string()).collect()
    }

    pub fn train_spam(&mut self, msg: &str) {
        for word in self.split_string_into_list_of_words(msg) {
            let new_counter = Counter{ham:1,spam:2};
            let counter = self.token_table.entry(word).or_insert(new_counter);
            counter.spam += 1;
        }
    }

    pub fn train_ham(&mut self, msg: &str) {
        for word in self.split_string_into_list_of_words(msg) {
            let new_counter = Counter{ham:2,spam:1};
            let counter = self.token_table.entry(word).or_insert(new_counter);
            counter.ham += 1;
        }
    }

    fn spam_total_count(&self) -> u32 {
        self.token_table.values().map(|x| x.spam).sum()
    }

    fn ham_total_count(&self) -> u32 {
        self.token_table.values().map(|x| x.ham).sum()
    }

    fn rate_words(&self, msg: &str) -> Vec<(f64,f64)> {
        let words_list =  self.split_string_into_list_of_words(msg);

        let mut ratings_list = Vec::new();
        for word in words_list.into_iter(){
            let counter = self.token_table.get(&word);
            if !counter.is_none() {
                let counter_unwraped = counter.unwrap();
                let ham_rating = (counter_unwraped.ham as f64) / (self.ham_total_count() as f64);
                let spam_rating = (counter_unwraped.spam as f64) / (self.spam_total_count() as f64); 
                ratings_list.push((ham_rating,spam_rating));
                ratings_list.push((ZEROES_REVERTER,ZEROES_REVERTER));
            } else{
                ratings_list.push((INITIAL_RATING,INITIAL_RATING));
            }
        }

        return ratings_list;
    }

    pub fn create_model_from_dataset(&mut self, dataset_file: &mut File, model_file: &mut File) -> Result<(), crate::utills::SpamClassifierError> {
        let reader = BufReader::new(dataset_file);

        for line in reader.lines() {
            match line {
                Ok(_) => {},
                Err(error) => return Err(crate::utills::SpamClassifierError::IO(error)),
            }

            let line_unwraped = line.unwrap();

            let split_line = crate::utills::take_and_skip(&line_unwraped, TAB);
            if split_line.is_none() {
                return Err(crate::utills::SpamClassifierError::InvalidDatasetFormatError(DATASET_FILE_FORMAT_INVALID.to_string()));
            }

            let split_line_unwraped = split_line.unwrap();
            let (first, second) = split_line_unwraped;
            if first == "" || second == "" {
                return Err(crate::utills::SpamClassifierError::InvalidDatasetFormatError(DATASET_FILE_FORMAT_INVALID.to_string()));
            }

            if first == HAM {
                self.train_ham(second);
            } else if first == SPAM {
                self.train_spam(second);
            }
        }

        match to_writer(model_file, &self) {
            Ok(_) => return Ok(()),
            Err(error) => return Err(crate::utills::SpamClassifierError::Serde(error)),
        }
    }

    pub fn get_spam_ham_probabilities(&self, msg: &str) -> (f64,f64) {
        let prob_list = self.rate_words(msg);

        let spam_initial_prob = (self.spam_total_count() as f64) / (self.ham_total_count() as f64 + self.spam_total_count() as f64);
        let ham_initial_prob = (self.ham_total_count() as f64) / (self.ham_total_count() as f64 + self.spam_total_count() as f64);

        let product_ham: f64 = prob_list.iter().map(|(first ,_)| first).product();
        let product_spam: f64 = prob_list.iter().map(|(_, second)|second).product();

        let final_product_spam = product_spam + spam_initial_prob;
        let final_product_ham = product_ham + ham_initial_prob;

        (final_product_spam,final_product_ham)
    }
}
