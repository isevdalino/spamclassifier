extern crate sha2;

use crate::utills::SpamClassifierError;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use sha2::{Sha256, Digest};
use std::io;
use std::fs;
use std::fs::File;
use serde_json::{from_reader, to_writer};
use std::path::Path;

const DEFAULT_CACHE_PATH: &str = "resources/cache.json";

#[derive(Default, Debug, Serialize, Deserialize)]
pub struct Cache {
    cache: HashMap<String, (f64, f64)>,
}

impl Cache {

    pub fn new() -> Result<Self, io::Error> {
        if Path::new(DEFAULT_CACHE_PATH).exists(){
            let file = File::open(DEFAULT_CACHE_PATH)?;
            let result = from_reader(file)?;
            return Ok(result);
        }

        Ok(Default::default())
    }
    
    pub fn add_to_cache(&mut self, string : &String, probs: (f64, f64)) ->  Result<(), SpamClassifierError> {
        let hashed_string = self.hash_string(string);
        self.cache.insert(hashed_string, probs);

        let file = File::create(DEFAULT_CACHE_PATH);
        match file {
            Ok(_) => {},
            Err(error) => return Err(SpamClassifierError::IO(error)),
        }

        match to_writer(file.unwrap(), &self) {
            Ok(_) => return Ok(()),
            Err(error) => return Err(SpamClassifierError::Serde(error)),
        }
    }

    pub fn get_from_cache(&self,string : &String) -> Option<(f64, f64)> {
        let hashed_string = self.hash_string(string);
        let value = self.cache.get(&hashed_string);
        if value.is_none(){
            return None;
        }

       Some(*value.unwrap())
    }

    pub fn clean_cache(&self) ->  Result<(), io::Error> {
        if !Path::new(DEFAULT_CACHE_PATH).exists(){
            return Ok(());
        }

        match fs::remove_file(DEFAULT_CACHE_PATH) {
            Ok(_) => return Ok(()),
            Err(error) => return Err(error),
        }
    }

    fn hash_string(&self,string : &String)-> String{
        let mut hasher = Sha256::new();
        hasher.update(string);
        format!("{:X}", hasher.finalize())
    }
}
