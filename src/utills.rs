#[derive(Debug)]
pub enum SpamClassifierError {
    InvalidDatasetFormatError(String),
    Serde(serde_json::Error),
    IO(std::io::Error),
}

fn skip_next(input: &str, target: char) -> Option<&str> {
    if input.is_empty(){
      return None
   }

   let first_letter = input.chars().nth(0).unwrap();
   if first_letter == target {
       return Some(&input[1..]);
   } 
   
   return None;
}

fn take_until(input: &str, target: char) -> (&str, &str) {
   for (i, c) in input.chars().enumerate() {
       if c == target {
           return input.split_at(i);
       } 
   }

   return (input, "");
}

pub fn take_and_skip(input: &str, target: char) -> Option<(&str, &str)> {
   let (first, second) = take_until(input, target);
    if second.is_empty() {
       return None
   }
   
   let second_without_target = skip_next(&second, target).unwrap();
   
   return Some((first, second_without_target))
}