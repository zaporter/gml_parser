use std::collections::HashMap;
extern crate pest;
#[macro_use]
extern crate pest_derive;

use pest::{iterators::Pairs, Parser};

#[derive(Parser, Debug)]
#[grammar = "grammar.pest"]
pub struct GMLParser;

#[derive(Debug, Clone, PartialEq, Eq)]
struct GMLObject {
    pairs: Vec<(String, GMLValue)>,
}
impl GMLObject {
    fn parse(obj: Pairs<'_, Rule>) -> Self {
        let mut current_key = None;
        let mut pairs = Vec::new();
        for entry in obj {
            dbg!(entry.as_rule());
            match entry.as_rule() {
                Rule::identifier => {
                    current_key = Some(entry.into_inner().as_str().to_owned());
                }
                Rule::value => {
                    let inner_value = entry.into_inner().next().unwrap();
                    match inner_value.as_rule() {
                        Rule::string => {
                            pairs.push((
                                current_key.clone().unwrap(),
                                GMLValue::GMLString(inner_value.into_inner().as_str().to_string()),
                            ));
                        }
                        Rule::number => {
                            todo!()
                        }
                        Rule::object => {
                            pairs.push((
                                current_key.clone().unwrap(),
                                GMLValue::GMLObject(Box::new(GMLObject::parse(
                                    inner_value.into_inner(),
                                ))),
                            ));
                        }
                        _ => {
                            dbg!(inner_value.as_rule());
                            unreachable!()
                        }
                    }
                }
                Rule::EOI => {}
                _ => {
                    // let inner = entry.into_inner();
                    // dbg!(inner.as_str());

                    dbg!(entry.as_rule());
                    unreachable!()
                }
            }
        }
        GMLObject { pairs }
    }
}
impl GMLValue {
    fn parse(obj: Pairs<'_, Rule>) -> Self {
        for entry in obj {
            dbg!(entry.as_rule());
            match entry.as_rule() {
                Rule::identifier => {
                    dbg!(entry.into_inner().as_str());
                }
                Rule::object => {
                    dbg!(entry.into_inner().as_str());
                }
                _ => unreachable!(),
            }
        }
        todo!()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum GMLValue {
    GMLString(String),
    GMLInt(i64),
    GMLObject(Box<GMLObject>),
}

struct Graph {
    directed: bool,
    id: usize,
    label: Option<String>,
}
struct Node {
    id: usize,
    label: Option<String>,
}
struct Edge {
    source: usize,
    target: usize,
    label: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    #[test]
    fn parse_empty() {
        let file = fs::read_to_string("tests/empty.gml").unwrap();
        let file = GMLParser::parse(Rule::text, &file).unwrap().next().unwrap();
        dbg!(file);
    }
    #[test]
    fn parse_single() {
        let file = fs::read_to_string("tests/single.gml").unwrap();
        let file = GMLParser::parse(Rule::text, &file).unwrap().next().unwrap();
        let root = GMLObject::parse(file.into_inner());
        let expected = GMLObject {
            pairs: vec![(
                "graph".into(),
                GMLValue::GMLObject(Box::new(GMLObject {
                    pairs: vec![("k".into(), GMLValue::GMLString("test".into()))],
                })),
            )],
        };
        assert_eq!(root, expected);
    }
    #[test]
    fn parse_simple() {
        let file = fs::read_to_string("tests/simple.gml").unwrap();
        let file = GMLParser::parse(Rule::text, &file).unwrap().next().unwrap();
        // let node = GMLObject::parse(file.into_inner());
        // let _:i32 = file.into_inner();
        // for line in file.into_inner() {
        //     dbg!(line.as_rule());
        //     // match line.as_rule() {
        //     //     _=> {println!("k")}
        //     // }
        // }
    }
    #[test]
    fn parse_wikipedia() {
        let file = fs::read_to_string("tests/wikipedia.gml").unwrap();
        let file = GMLParser::parse(Rule::text, &file).unwrap().next().unwrap();
        // let node = GMLObject::parse(file.into_inner());
        // let _:i32 = file.into_inner();
        // for line in file.into_inner() {
        //     dbg!(line.as_rule());
        //     // match line.as_rule() {
        //     //     _=> {println!("k")}
        //     // }
        // }
    }
}

// /**
//  * TODO:
//  *
//  */
// use regex::Regex;

// impl Graph {
//     fn parse(inner:&str)->Self {
//         todo!()
//     }
// }
// impl Node {
//     fn parse(inner:&str)-> Self {
//         let mut id = None;
//         let mut label = None;
//         let id_reg = Regex::new(r"id (?P<id>\d+)").unwrap();
//         let label_reg = Regex::new(r#"label "(?P<label>.+)""#).unwrap();
//         for line in inner.lines() {
//             let line =line.trim();
//             let id_captures = id_reg.captures(line);
//             if let Some(id_captures) = id_captures {
//                 let id_capture = id_captures.name("id").unwrap();
//                 id = Some(id_capture.as_str().parse().unwrap())
//             }
//             let label_captures = label_reg.captures(line);
//             if let Some(label_captures) = label_captures {
//                 let label_capture = label_captures.name("label").unwrap();
//                 label = Some(label_capture.as_str().to_owned());
//             }
//         }
//         Node {
//             id: id.unwrap(),
//             label
//         }
//     }
// }

// #[cfg(test)]
// mod test {
//     use super::*;
//     #[test]
//     fn node_simple() {
//         let input = "id 3
//             label \"chicken\"";
//         let node = Node::parse(input);
//         assert_eq!(node.id, 3);
//         assert_eq!(node.label, Some("chicken".into()));
//     }
//     #[test]
//     fn simple() {
//         assert_eq!(5,1+4);
//     }
// }
