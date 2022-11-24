//! This crate allows for reading [Graph Modeling Language (GML)](https://en.wikipedia.org/wiki/Graph_Modelling_Language) files.
//!
//!
//! This crate first parses the GML into [GMLObject]s and [GMLValue]s. Then the root GMLObject can
//! be transformed into a [Graph] containing [Node]s and [Edge]s.
//!
//! # Examples
//! ```
//! use gml_parser::{GMLObject, Graph};
//!
//! let data = r#"
//! graph [            
//!    id 4           
//!    node [         
//!        id 0       
//!    ]              
//!    node [         
//!        id 1       
//!    ]              
//!    edge [         
//!        source 1   
//!        target 0   
//!        label "Edge"
//!    ]              
//!]"#;
//! let root = GMLObject::from_str(data).unwrap();
//! let graph = Graph::from_gml(root).unwrap();
//! assert_eq!(graph.id, Some(4));
//! assert_eq!(graph.nodes.len(), 2);
//! assert_eq!(graph.edges.len(), 1);
//! ```
//!
//! # Limitations
//! - This implementation can be fragile and GML is not a very picky standard
//! - We duplicate the data when parsing which can have performance impacts on very large graphs
//!

use std::{error::Error, fmt::Display};
extern crate pest;
#[macro_use]
extern crate pest_derive;

use pest::{iterators::Pairs, Parser};

#[derive(Debug)]
pub struct GMLError(String);

impl Error for GMLError {
    fn description(&self) -> &str {
        &self.0
    }
}

impl Display for GMLError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "GMLError: {}", self.0)
    }
}

#[derive(Parser, Debug)]
#[grammar = "grammar.pest"]
struct GMLParser;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GMLObject {
    pub pairs: Vec<(String, GMLValue)>,
}
impl GMLObject {
    fn parse(obj: Pairs<'_, Rule>) -> Result<Self, Box<dyn Error>> {
        let mut current_key = None;
        let mut pairs = Vec::new();
        for entry in obj {
            match entry.as_rule() {
                Rule::identifier => {
                    current_key = Some(entry.into_inner().as_str().to_owned());
                }
                Rule::value => {
                    let inner_value = entry
                        .into_inner()
                        .next()
                        .ok_or(GMLError("No rule inner value. Please report this.".into()))?;
                    match inner_value.as_rule() {
                        Rule::string => {
                            pairs.push((
                                current_key.clone().ok_or(GMLError(
                                    "String: No rule current key. Please report this.".into(),
                                ))?,
                                GMLValue::GMLString(inner_value.into_inner().as_str().to_string()),
                            ));
                        }
                        Rule::number => {
                            pairs.push((
                                current_key.clone().ok_or(GMLError(
                                    "Number: No rule current key. Please report this".into(),
                                ))?,
                                GMLValue::GMLInt(inner_value.as_str().parse()?),
                            ));
                        }
                        Rule::object => {
                            pairs.push((
                                current_key.clone().ok_or(GMLError(
                                    "Object: No rule current key. Please report this".into(),
                                ))?,
                                GMLValue::GMLObject(Box::new(GMLObject::parse(
                                    inner_value.into_inner(),
                                )?)),
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
                    dbg!(entry.as_rule());
                    unreachable!()
                }
            }
        }
        Ok(GMLObject { pairs })
    }
    pub fn from_str(text: &str) -> Result<GMLObject, GMLError> {
        let file = match GMLParser::parse(Rule::text, text) {
            Ok(k) => Ok(k),
            Err(e) => Err(GMLError(format!(
                "Failed to parse GML! (syntactic): {:?}",
                e
            ))),
        }?
        .next()
        .unwrap();
        match GMLObject::parse(file.into_inner()) {
            Ok(k) => Ok(k),
            Err(e) => Err(GMLError(format!(
                "Failed to parse GML! (semantic): {:?}",
                e
            ))),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GMLValue {
    GMLString(String),
    GMLInt(i64),
    GMLObject(Box<GMLObject>),
}

#[derive(Debug, Clone)]
pub struct Graph {
    pub directed: Option<bool>,
    pub id: Option<i64>,
    pub label: Option<String>,
    pub nodes: Vec<Node>,
    pub edges: Vec<Edge>,
    attrs: Vec<(String, GMLValue)>,
}
#[derive(Debug, Clone)]
pub struct Node {
    pub id: i64,
    pub label: Option<String>,
    attrs: Vec<(String, GMLValue)>,
}
#[derive(Debug, Clone)]
pub struct Edge {
    pub source: i64,
    pub target: i64,
    pub label: Option<String>,
    attrs: Vec<(String, GMLValue)>,
}

impl Graph {
    // This turns the data into the object.
    // The other function is a wrapper to deal with the
    // outer graph[...] nonsense
    fn int_from_gml(mut obj: GMLObject) -> Result<Self, GMLError> {
        let id = int_take_attribute(&mut obj.pairs, "id");
        let id = if let Some(id) = id {
            let GMLValue::GMLInt(id) = id.1 else {
                return Err(GMLError(format!("Failed to parse graph id: {:?}. Expected int but found invalid type.", id.1)));
            };
            Some(id)
        } else {
            None
        };
        let directed = int_take_attribute(&mut obj.pairs, "directed");
        let directed = if let Some(directed) = directed {
            let GMLValue::GMLInt(directed) = directed.1 else {
                return Err(GMLError(format!("Failed to parse graph directed: {:?}. Expected int but found invalid type.", directed.1)));
            };
            Some(directed == 1)
        } else {
            None
        };

        let label = int_take_attribute(&mut obj.pairs, "label");
        let label = if let Some(label) = label {
            let GMLValue::GMLString(label) = label.1 else {
                return Err(GMLError(format!("Failed to parse edge label: {:?}. Expected str but found invalid type.", label.1)));
            };
            Some(label)
        } else {
            None
        };
        let mut nodes = Vec::new();
        let mut edges = Vec::new();
        while let Some((_, node)) = int_take_attribute(&mut obj.pairs, "node") {
            let GMLValue::GMLObject(node) = node else {
                return Err(GMLError(format!("Failed to parse node: {:?}. Expected object but found invalid type.", node)));
            };
            nodes.push(Node::from_gml(*node)?);
        }
        while let Some((_, edge)) = int_take_attribute(&mut obj.pairs, "edge") {
            let GMLValue::GMLObject(edge) = edge else {
                return Err(GMLError(format!("Failed to parse edge: {:?}. Expected object but found invalid type.", edge)));
            };
            edges.push(Edge::from_gml(*edge)?);
        }
        Ok(Graph {
            directed,
            id,
            label,
            nodes,
            edges,
            attrs: obj.pairs,
        })
    }
    /// Transform a [GMLObject] into a graph. This expects the root node
    /// of the graph.
    ///
    /// Note: This does not currently accept multiple graphs in a single file
    pub fn from_gml(mut obj: GMLObject) -> Result<Self, GMLError> {
        let graph = int_take_attribute(&mut obj.pairs, "graph");
        let Some(graph) = graph else {
            return Err(GMLError(format!("Unable to parse graph from GMLObject")));
        };
        let GMLValue::GMLObject(graph) = graph.1 else {
            return Err(GMLError(format!("Failed to parse graph: {:?}. Expected graph but found invalid type.", graph.1)));
        };
        Self::int_from_gml(*graph)
    }
}

impl Node {
    fn from_gml(mut obj: GMLObject) -> Result<Self, GMLError> {
        let id = int_take_attribute(&mut obj.pairs, "id");
        let Some(id) = id else {
            return Err(GMLError(format!("Unable to parse id from node")));
        };
        let GMLValue::GMLInt(id) = id.1 else {
            return Err(GMLError(format!("Failed to parse node id: {:?}. Expected int but found invalid type.", id.1)));
        };
        let label = int_take_attribute(&mut obj.pairs, "label");
        let label = if let Some(label) = label {
            let GMLValue::GMLString(label) = label.1 else {
                return Err(GMLError(format!("Failed to parse edge label: {:?}. Expected str but found invalid type.", label.1)));
            };
            Some(label)
        } else {
            None
        };
        Ok(Self {
            id,
            label,
            attrs: obj.pairs,
        })
    }
}
impl Edge {
    fn from_gml(mut obj: GMLObject) -> Result<Self, GMLError> {
        let source = int_take_attribute(&mut obj.pairs, "source");
        let Some(source) = source else {
            return Err(GMLError(format!("Unable to parse source from edge")));
        };
        let GMLValue::GMLInt(source) = source.1 else {
            return Err(GMLError(format!("Failed to parse edge source id: {:?}. Expected int but found invalid type.", source.1)));
        };
        let target = int_take_attribute(&mut obj.pairs, "target");
        let Some(target) = target else {
            return Err(GMLError(format!("Unable to parse target from edge")));
        };
        let GMLValue::GMLInt(target) = target.1 else {
            return Err(GMLError(format!("Failed to parse edge source id: {:?}. Expected int but found invalid type.", target.1)));
        };
        let label = int_take_attribute(&mut obj.pairs, "label");
        let label = if let Some(label) = label {
            let GMLValue::GMLString(label) = label.1 else {
                return Err(GMLError(format!("Failed to parse edge label: {:?}. Expected str but found invalid type.", label.1)));
            };
            Some(label)
        } else {
            None
        };

        Ok(Self {
            source,
            target,
            label,
            attrs: obj.pairs,
        })
    }
}
pub trait HasGMLAttributes {
    fn attributes(&self) -> &Vec<(String, GMLValue)>;
    fn attributes_mut(&mut self) -> &mut Vec<(String, GMLValue)>;
}

pub trait ReadableGMLAttributes<'a> {
    /// Take the attribute from the object if the key == name
    fn take_attribute(&mut self, name: &str) -> Option<(String, GMLValue)>;
    /// Return a reference to the object if the key == name
    fn get_attribute(&'a self, name: &str) -> Option<&'a (String, GMLValue)>;
}
fn int_take_attribute(
    attrs: &mut Vec<(String, GMLValue)>,
    name: &str,
) -> Option<(String, GMLValue)> {
    let mut index = None;
    for (i, attr) in attrs.iter().enumerate() {
        if attr.0 == name {
            index = Some(i);
            break;
        }
    }
    if let Some(index) = index {
        // remove is O(n) which would make
        // building the graph O(n^2)
        Some(attrs.swap_remove(index))
    } else {
        None
    }
}
fn int_get_attribute<'a>(
    attrs: &'a Vec<(String, GMLValue)>,
    name: &str,
) -> Option<&'a (String, GMLValue)> {
    for attr in attrs {
        if attr.0 == name {
            return Some(&attr);
        }
    }
    None
}
// Blanket impl is far better but it doesn't show up in the docs.
// impl<'a, T> ReadableGMLAttributes<'a> for T
// where
//     T: HasGMLAttributes,
// {
//     fn take_attribute(&mut self, name: &str) -> Option<(String, GMLValue)> {
//         let attrs = self.attributes_mut();
//         int_take_attribute(attrs, name)
//     }
//     fn get_attribute(&'a self, name: &str) -> Option<&'a (String, GMLValue)> {
//         let attrs = self.attributes();
//         int_get_attribute(attrs, name)
//     }
// }
impl<'a> ReadableGMLAttributes<'a> for Node {
    fn take_attribute(&mut self, name: &str) -> Option<(String, GMLValue)> {
        let attrs = self.attributes_mut();
        int_take_attribute(attrs, name)
    }
    fn get_attribute(&'a self, name: &str) -> Option<&'a (String, GMLValue)> {
        let attrs = self.attributes();
        int_get_attribute(attrs, name)
    }
}
impl<'a> ReadableGMLAttributes<'a> for Edge {
    fn take_attribute(&mut self, name: &str) -> Option<(String, GMLValue)> {
        let attrs = self.attributes_mut();
        int_take_attribute(attrs, name)
    }
    fn get_attribute(&'a self, name: &str) -> Option<&'a (String, GMLValue)> {
        let attrs = self.attributes();
        int_get_attribute(attrs, name)
    }
}
impl<'a> ReadableGMLAttributes<'a> for Graph {
    fn take_attribute(&mut self, name: &str) -> Option<(String, GMLValue)> {
        let attrs = self.attributes_mut();
        int_take_attribute(attrs, name)
    }
    fn get_attribute(&'a self, name: &str) -> Option<&'a (String, GMLValue)> {
        let attrs = self.attributes();
        int_get_attribute(attrs, name)
    }
}

impl HasGMLAttributes for Node {
    fn attributes(&self) -> &Vec<(String, GMLValue)> {
        return &self.attrs;
    }
    fn attributes_mut(&mut self) -> &mut Vec<(String, GMLValue)> {
        return &mut self.attrs;
    }
}
impl HasGMLAttributes for Edge {
    fn attributes(&self) -> &Vec<(String, GMLValue)> {
        return &self.attrs;
    }
    fn attributes_mut(&mut self) -> &mut Vec<(String, GMLValue)> {
        return &mut self.attrs;
    }
}
impl HasGMLAttributes for Graph {
    fn attributes(&self) -> &Vec<(String, GMLValue)> {
        return &self.attrs;
    }
    fn attributes_mut(&mut self) -> &mut Vec<(String, GMLValue)> {
        return &mut self.attrs;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    #[test]
    fn parse_empty() {
        let file = fs::read_to_string("tests/empty.gml").unwrap();
        let file = GMLParser::parse(Rule::text, &file).unwrap().next().unwrap();
        let root = GMLObject::parse(file.into_inner()).unwrap();
        assert!(Graph::from_gml(root).is_ok());
    }
    #[test]
    fn parse_single() {
        let file = fs::read_to_string("tests/single.gml").unwrap();
        let file = GMLParser::parse(Rule::text, &file).unwrap().next().unwrap();
        let root = GMLObject::parse(file.into_inner()).unwrap();
        let expected = GMLObject {
            pairs: vec![(
                "graph".into(),
                GMLValue::GMLObject(Box::new(GMLObject {
                    pairs: vec![("k".into(), GMLValue::GMLString("test".into()))],
                })),
            )],
        };
        assert_eq!(root, expected);
        assert!(Graph::from_gml(root).is_ok());
    }
    #[test]
    fn parse_simple() {
        let file = fs::read_to_string("tests/simple.gml").unwrap();
        let file = GMLParser::parse(Rule::text, &file).unwrap().next().unwrap();
        let root = GMLObject::parse(file.into_inner()).unwrap();
        assert!(Graph::from_gml(root).is_ok());
    }
    #[test]
    fn parse_wikipedia() {
        let file = fs::read_to_string("tests/wikipedia.gml").unwrap();
        let file = GMLParser::parse(Rule::text, &file).unwrap().next().unwrap();
        let root = GMLObject::parse(file.into_inner()).unwrap();
        let graph = Graph::from_gml(root).unwrap();
        assert_eq!(graph.id, Some(42));
        assert_eq!(graph.directed, Some(true));
        assert_eq!(graph.label, Some("Hello, I am a graph".into()));
        assert_eq!(graph.nodes.len(), 3);
        assert_eq!(graph.edges.len(), 3);
    }

    #[test]
    fn parse_synoptic() {
        let file = fs::read_to_string("tests/synoptic.gml").unwrap();
        let file = GMLParser::parse(Rule::text, &file).unwrap().next().unwrap();
        let root = GMLObject::parse(file.into_inner()).unwrap();
        let graph = Graph::from_gml(root).unwrap();
        assert_eq!(graph.nodes.len(), 7);
        assert_eq!(graph.nodes[0].id, 0);
        assert_eq!(graph.nodes[0].label, Some("a".into()));
        assert_eq!(graph.nodes[6].id, 6);
        assert_eq!(graph.nodes[6].label, Some("INITIAL".into()));
        dbg!(&graph);
        assert_eq!(graph.edges.len(), 8);
        assert_eq!(graph.edges[0].label, Some("P: 1.00".into()));
        assert_eq!(graph.edges[0].source, 6);
        assert_eq!(graph.edges[0].target, 0);
    }
}
