extern crate clap;

use spamclassifier::bayesian_spam_classifier::BayesianSpamClassifier;
use spamclassifier::cache::Cache;
use clap::{Arg, App, SubCommand,AppSettings,ArgMatches};
use std::io::prelude::*;
use std::fs::File;
use std::path::Path;
use rayon::prelude::*;

const NEUTRAL_MULTIPLIER : f64 = 1.0;
const APPLICATION_AUTHOR: &str = "Ivan Ivanov";
const APPLICATION_NAME: &str = "Spam classifier";
const DEFAULT_MODEL_PATH: &str = "resources/model.json";
const CLEAN_CACHE_SUBCOMMAND: &str = "clean-cache";
const CREATE_MODEL_FROM_DATASET_SUBCOMMAND: &str = "create-model-from-dataset";
const FROM_MODEL_PARAMETER: &str = "from-model";
const MESSAGE_PARAMETER: &str = "message";
const MESSAGE_FROM_FILE_PARAMETER: &str = "message-from-file";
const DATASET_PATH_PARAMETER: &str = "dataset-path";
const MODEL_PATH_PARAMETER: &str = "model-path";
const APP_VERSION: &str = "1.0";
const SUBCOMMAND_VERSION: &str = "1.3";
const APP_ABOUT: &str = "Classifies whether a message is a spam or not";
const CREATE_MODEL_FROM_DATASET_SUBCOMMAND_ABOUT: &str = "Creates a new model from the specified dataset and writes it to the specified file";
const DATASET_PATH_PARAMETER_HELP: &str = "The path to the dataset file";
const MODEL_PATH_PARAMETER_HELP: &str = "The path to the model file to be created";
const CLEAN_CACHE_SUBCOMMAND_ABOUT: &str = "Cleans the spam classification cache";
const FROM_MODEL_PARAMETER_HELP: &str = "Loads a model from file to use for the classifications";
const MESSAGE_PARAMETER_HELP: &str = "The message to classify";
const MESSAGE_FROM_FILE_PARAMETER_HELP: &str = "Path to the file from which to extract the message to classify";

fn main() {
    let matches = App::new(APPLICATION_NAME)
                        .version(APP_VERSION)
                        .author(APPLICATION_AUTHOR)
                        .about(APP_ABOUT)
                        .setting(AppSettings::SubcommandsNegateReqs)
                        .subcommand(SubCommand::with_name(CREATE_MODEL_FROM_DATASET_SUBCOMMAND)
                            .about(CREATE_MODEL_FROM_DATASET_SUBCOMMAND_ABOUT)
                            .version(SUBCOMMAND_VERSION)
                            .author(APPLICATION_AUTHOR)
                            .arg(Arg::with_name(DATASET_PATH_PARAMETER)
                                .long(DATASET_PATH_PARAMETER)
                                .required(true)
                                .takes_value(true)
                                .help(DATASET_PATH_PARAMETER_HELP))
                            .arg(Arg::with_name(MODEL_PATH_PARAMETER)
                                .long(MODEL_PATH_PARAMETER)
                                .required(true)
                                .takes_value(true)
                                .help(MODEL_PATH_PARAMETER_HELP)))
                        .subcommand(SubCommand::with_name(CLEAN_CACHE_SUBCOMMAND)
                            .about(CLEAN_CACHE_SUBCOMMAND_ABOUT)
                            .version(SUBCOMMAND_VERSION)
                            .author(APPLICATION_AUTHOR))
                        .arg(Arg::with_name(FROM_MODEL_PARAMETER)
                            .long(FROM_MODEL_PARAMETER)
                            .help(FROM_MODEL_PARAMETER_HELP)
                            .takes_value(true))
                        .arg(Arg::with_name(MESSAGE_PARAMETER)
                            .long(MESSAGE_PARAMETER)
                            .required(true)
                            .conflicts_with(MESSAGE_FROM_FILE_PARAMETER)
                            .help(MESSAGE_PARAMETER_HELP)
                            .takes_value(true))
                        .arg(Arg::with_name(MESSAGE_FROM_FILE_PARAMETER)
                            .long(MESSAGE_FROM_FILE_PARAMETER)
                            .required(true)
                            .conflicts_with(MESSAGE_PARAMETER)
                            .help(MESSAGE_FROM_FILE_PARAMETER_HELP)
                            .takes_value(true))
                        .get_matches();

    let cache = Cache::new();
    match cache {
        Ok(_) => {}
        Err(error) => panic!("An error ocurred while initializing cache logic - {:?}", error),
    }

    let mut cache_unwraped = cache.unwrap();

    execute_clean_cache_if_specified(&cache_unwraped, &matches);

    execute_create_model_from_dataset_if_specified(&matches);
    
    let model_filename = get_model_filename(&matches);
    
    execute_message_if_specified(&model_filename, &mut cache_unwraped, &matches);

    execute_message_from_file_if_specified(&model_filename, &cache_unwraped, &matches);
}

pub fn execute_clean_cache_if_specified(cache : &Cache, matches: &ArgMatches){
    if let Some(_) = matches.subcommand_matches(CLEAN_CACHE_SUBCOMMAND) {
        match cache.clean_cache() {
            Ok(_) => {}
            Err(error) => panic!("An error ocurred while trying to clean the cache - {:?}", error),
        }
    }
}

pub fn execute_create_model_from_dataset_if_specified(matches: &ArgMatches){
    if let Some(matches) = matches.subcommand_matches(CREATE_MODEL_FROM_DATASET_SUBCOMMAND) {
        let dataset_path = matches.value_of(DATASET_PATH_PARAMETER).unwrap();
        let model_path = matches.value_of(MODEL_PATH_PARAMETER).unwrap();

        if !Path::new(dataset_path).exists(){
            panic!("The specified dataset file path - {:?}, does not exist!", dataset_path);
        }

        if Path::new(model_path).exists(){
            panic!("The specified model file - {:?}, already exist!", model_path);
        }

        let dataset_file = File::open(dataset_path);
        match dataset_file {
            Ok(_) => {}
            Err(error) => panic!("Failed to open file with name {} - {:?}",dataset_path, error),
        }

        let model_file = File::create(model_path);
        match model_file {
            Ok(_) => {}
            Err(error) => panic!("Failed to create file with name {} - {:?}",model_path, error),
        }

        let mut classifier = BayesianSpamClassifier::new();
        let result = classifier.create_model_from_dataset(&mut dataset_file.unwrap(),&mut model_file.unwrap());
        match result {
            Ok(_) => {}
            Err(error) => panic!("An error ocurred while creating the model and publishing it to the file - {:?}", error),
        }
    }
}

pub fn get_model_filename(matches: &ArgMatches) -> String{
    if let Some(from_model) = matches.value_of(FROM_MODEL_PARAMETER) {
        if !Path::new(from_model).exists(){
            panic!("The specified model file path - {:?}, does not exist!", from_model);
        }

        return from_model.to_string();
    }

    DEFAULT_MODEL_PATH.to_string()
}

pub fn execute_message_if_specified(model_filename: &str, cache: &mut Cache,matches: &ArgMatches){
    if let Some(message) = matches.value_of(MESSAGE_PARAMETER) {        
        let file = File::open(model_filename);
        match file {
            Ok(_) => {}
            Err(error) => panic!("Failed to open file with name {} - {:?}",model_filename, error),
        }

        let cached_classification = cache.get_from_cache(&message.to_string());
        if !cached_classification.is_none(){
            let (spam_prob, ham_prob) = cached_classification.unwrap();
            print_spam_or_ham(spam_prob,ham_prob,message);
        } else {
            let classifier = BayesianSpamClassifier::new_from_pre_trained(&mut file.unwrap());
            match classifier {
                Ok(_) => {}
                Err(error) => panic!("An error ocurred while creating classifier from the pre-trained model {} - {:?}",model_filename, error),
            }

            let classifier_unwraped = classifier.unwrap();
            let probs = classifier_unwraped.get_spam_ham_probabilities(message);
            match cache.add_to_cache(&message.to_string(), probs) {
                Ok(_) => {}
                Err(error) => panic!("An error ocurred while trying to add value {} in cache - {:?}", message, error),
            }

            let (spam_prob, ham_prob) = probs;
            print_spam_or_ham(spam_prob,ham_prob,message);
        }
    }
}

fn print_spam_or_ham(spam_prob: f64, ham_prob: f64,message: &str){
    if spam_prob > ham_prob {
        println!("The message - {:?}, is indeed spam!", message);
    } else {
        println!("The message - {:?}, is indeed ham!", message);
    }
}

pub fn execute_message_from_file_if_specified(model_filename:&str , cache : &Cache,matches: &ArgMatches){
    if let Some(file_containing_message) = matches.value_of(MESSAGE_FROM_FILE_PARAMETER) {
        let file = File::open(file_containing_message);
        match file {
            Ok(_) => {}
            Err(error) => panic!("An error ocurred while trying to open provided file with name {} - {:?}", file_containing_message, error),
        }

        let model_file = File::open(model_filename);
        match model_file {
            Ok(_) => {}
            Err(error) => panic!("Failed to open file with name {} - {:?}", model_filename, error),
        }

        let classifier = BayesianSpamClassifier::new_from_pre_trained(&mut model_file.unwrap());
        match classifier {
            Ok(_) => {}
            Err(error) => panic!("An error ocurred while creating classifier from the pre-trained model {} - {:?}", model_filename, error),
        }

        let classifier_unwraped = classifier.unwrap();

        let file_buff_reader = std::io::BufReader::new(file.unwrap());
        let spam_ham_probs = file_buff_reader.lines()
                    .filter_map(|line: Result<String, _>| line.ok())
                    .par_bridge()
                    .map(|line: String| {
                                let cached_classification = cache.get_from_cache(&line);
                                if !cached_classification.is_none(){
                                    return cached_classification.unwrap();
                                }
                                
                                classifier_unwraped.get_spam_ham_probabilities(&line)
                                })
                    .reduce( || (NEUTRAL_MULTIPLIER, NEUTRAL_MULTIPLIER),|acc:(f64,f64), curr:(f64,f64)|{ 
                        let (first, second) = curr;
                        let (first_acc,second_acc) = acc;
                        let first_new :f64= first_acc * first;
                        let second_new :f64= second_acc * second;
                        (first_new,second_new)
                    });

        let (first , second) = spam_ham_probs;
        if first > second {
            println!("The message from file - {:?}, is indeed spam!", file_containing_message);
        } else {
            println!("The message from file - {:?}, is indeed ham!", file_containing_message);
        }
    }
}
