use regex_lite::Regex;
use serde::Serialize;
use std::{collections::HashMap, fs, str::Lines};

#[derive(Debug, Serialize)]
struct JsonReferenceObject {
    class_name: String,

    #[serde(serialize_with = "sort_alphabetically")]
    properties: HashMap<String, String>,
}

fn sort_alphabetically<T: Serialize, S: serde::Serializer>(
    value: &T,
    serializer: S,
) -> Result<S::Ok, S::Error> {
    let value = serde_json::to_value(value).map_err(serde::ser::Error::custom)?;
    value.serialize(serializer)
}

#[derive(Debug)]
enum TypescriptSchemaParserState {
    ClassParse,
    PropertyParse,
    Complete,
}

#[derive(Debug)]
pub struct TypescriptSchemaParser<'a> {
    doc_lines: Lines<'a>,
    json_reference_object: JsonReferenceObject,
    state: TypescriptSchemaParserState,
}

#[derive(Debug)]
pub struct UnableToParse;
impl<'a> TypescriptSchemaParser<'a> {
    fn new(doc_string: &'a str) -> Self {
        Self {
            doc_lines: doc_string.lines(),
            json_reference_object: JsonReferenceObject {
                class_name: Default::default(),
                properties: Default::default(),
            },
            state: TypescriptSchemaParserState::ClassParse,
        }
    }

    pub fn parse_single(doc_string: impl AsRef<str>) -> Result<String, UnableToParse> {
        let mut parser = TypescriptSchemaParser::new(doc_string.as_ref());
        while parser.process() {}
        return serde_json::to_string(&parser.json_reference_object).map_err(|_| UnableToParse);
    }

    pub fn parse(file_list: Vec<String>) -> Result<String, UnableToParse> {
        let mut json_reference_objects = vec![];
        for file_path in file_list {
            let doc_string = fs::read_to_string(file_path.clone()).unwrap();
            let mut parser = TypescriptSchemaParser::new(doc_string.as_ref());
            while parser.process() {}

            if !parser.json_reference_object.class_name.is_empty() {
                json_reference_objects.push(parser.json_reference_object);
            }
        }

        return serde_json::to_string(&json_reference_objects).map_err(|_| UnableToParse);
    }
}

impl<'a> TypescriptSchemaParser<'a> {
    fn process(&mut self) -> bool {
        match self.state {
            TypescriptSchemaParserState::ClassParse => {
                self.class_parse();
                return true;
            }
            TypescriptSchemaParserState::PropertyParse => {
                self.property_parse();
                return true;
            }
            TypescriptSchemaParserState::Complete => {
                return false;
            }
        }
    }

    fn class_parse(&mut self) {
        let class_processor = Regex::new(r"class (\w+)").unwrap();
        while let Some(line) = self.doc_lines.next() {
            if let Some(captures) = class_processor.captures(line) {
                let class_name = captures.get(1).unwrap().as_str().to_string();
                self.json_reference_object.class_name = class_name;
                self.state = TypescriptSchemaParserState::PropertyParse;
                return;
            }
        }
        self.state = TypescriptSchemaParserState::Complete;
    }

    fn property_parse(&mut self) {
        let property_processor = Regex::new(r".*?(\w+): (.*);").unwrap();
        while let Some(line) = self.doc_lines.next() {
            if let Some(captures) = property_processor.captures(line) {
                let property_name = captures.get(1).unwrap().as_str().to_string();
                let property_type = captures.get(2).unwrap().as_str().to_string();
                self.json_reference_object
                    .properties
                    .insert(property_name, property_type);
            }
        }
        self.state = TypescriptSchemaParserState::Complete;
    }
}

pub fn list_files(dir: impl AsRef<str>) -> Vec<String> {
    let mut file_list = Vec::new();

    if let Ok(dir) = fs::read_dir(dir.as_ref()) {
        for dir_entry_result in dir.into_iter() {
            if let Ok(dir_entry) = dir_entry_result {
                match dir_entry.file_type() {
                    Ok(entry) => {
                        if entry.is_file() {
                            file_list.push(dir_entry.path().to_str().unwrap().to_owned())
                        } else {
                            file_list.append(&mut list_files(
                                dir_entry.path().to_str().unwrap().to_owned(),
                            ))
                        }
                    }
                    _ => {}
                }
            }
        }
    }

    return file_list;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse() {
        let doc_string = fs::read_to_string("assets/Greeter.ts").unwrap();
        let result = TypescriptSchemaParser::parse_single(doc_string);
        println!("{:?}", result);
    }

    #[test]
    fn test_parse_multiple() {
        let file_list = list_files("assets/");
        let result = TypescriptSchemaParser::parse(file_list).unwrap();

        println!("{:?}", result);

        fs::write("out.json", result).unwrap();
    }
}
