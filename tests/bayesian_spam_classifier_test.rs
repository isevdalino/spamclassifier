use spamclassifier::*;

use std::fs::File;
use std::io;
use std::fs;
use crate::utills::SpamClassifierError;
use spamclassifier::bayesian_spam_classifier::BayesianSpamClassifier;
use std::path::Path;

const DEFAULT_DATASET_PATH: &str = "resources/SMSSpamCollection";
const DEFAULT_MODEL_TEST_PATH: &str = "resources/modelTest.json";
const DEFAULT_MODEL_PATH: &str = "resources/model.json";
const TYPICAL_SPAM_MESSAGE : &str = "Lose up to 19% weight. Special promotion on our new weightloss.";
const TYPICAL_HAM_MESSAGE : &str = "Hi Bob, can you send me your machine learning homework?";

#[test]
fn test_new() {
    let mut classifier = BayesianSpamClassifier::new();

    let spam = "Don't forget our special promotion: -30% on men shoes, only today!";
    classifier.train_spam(spam);

    let ham = "Hi Bob, don't forget our meeting today at 4pm.";
    classifier.train_ham(ham);

    let (spam_prob, ham_prob) = classifier.get_spam_ham_probabilities(TYPICAL_SPAM_MESSAGE);
    assert!(spam_prob > ham_prob);

    let (spam_prob, ham_prob) = classifier.get_spam_ham_probabilities(TYPICAL_HAM_MESSAGE);
    assert!(spam_prob < ham_prob);
}

#[test]
fn test_new_from_pre_trained() -> Result<(), io::Error> {
    let mut file = File::open(DEFAULT_MODEL_PATH)?;
    let classifier = BayesianSpamClassifier::new_from_pre_trained(&mut file)?;

    let (spam_prob, ham_prob) = classifier.get_spam_ham_probabilities(TYPICAL_SPAM_MESSAGE);
    assert!(spam_prob > ham_prob);

    let (spam_prob, ham_prob) = classifier.get_spam_ham_probabilities(TYPICAL_HAM_MESSAGE);
    assert!(spam_prob < ham_prob);

    Ok(())
}

#[test]
fn test_create_model_from_dataset() -> Result<(), SpamClassifierError> {
    let mut classifier = BayesianSpamClassifier::new();
    let dataset_file = File::open(DEFAULT_DATASET_PATH);
    match dataset_file {
        Ok(_) =>  {},
        Err(error) => return Err(SpamClassifierError::IO(error)),
    }

    let model_file = File::create(DEFAULT_MODEL_TEST_PATH);
    match model_file {
        Ok(_) => {},
        Err(error) => return Err(SpamClassifierError::IO(error)),
    }
    
    classifier.create_model_from_dataset(&mut dataset_file.unwrap(),&mut model_file.unwrap())?;

    assert!(Path::new(DEFAULT_MODEL_TEST_PATH).exists());

    let (spam_prob, ham_prob) = classifier.get_spam_ham_probabilities(TYPICAL_SPAM_MESSAGE);
    assert!(spam_prob > ham_prob);

    let (spam_prob, ham_prob) = classifier.get_spam_ham_probabilities(TYPICAL_HAM_MESSAGE);
    assert!(spam_prob < ham_prob);

    match fs::remove_file(DEFAULT_MODEL_TEST_PATH) {
        Ok(_) => return Ok(()),
        Err(error) => return Err(SpamClassifierError::IO(error)),
    }
}
