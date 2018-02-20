// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use failure::Error;
use git2::{Commit, Oid};
use serde::ser::{Serialize, SerializeMap, SerializeStruct, Serializer};
use std::collections::HashMap;

#[derive(Debug)]
pub struct Node {
    parents: Vec<Oid>,
    children: Vec<Oid>,
}

fn oid_to_string(oid: &Oid) -> String {
    format!("{}", oid)
}

fn oids_to_strings(oids: &[Oid]) -> Vec<String> {
    oids.iter().map(oid_to_string).collect()
}

impl Node {
    pub fn new() -> Node {
        Node {
            parents: vec![],
            children: vec![],
        }
    }

    pub fn parent_strings(&self) -> Vec<String> {
        oids_to_strings(&self.parents)
    }

    pub fn children_strings(&self) -> Vec<String> {
        oids_to_strings(&self.children)
    }
}

impl Serialize for Node {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("Node", 2)?;
        state.serialize_field("parents", &self.parent_strings())?;
        state.serialize_field("children", &self.children_strings())?;
        state.end()
    }
}

#[derive(Debug)]
pub struct Graph {
    graph: HashMap<Oid, Node>,
}

impl Serialize for Graph {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut map = serializer.serialize_map(Some(self.graph.len()))?;
        for (k, v) in &self.graph {
            map.serialize_entry(&oid_to_string(&k), &v)?;
        }
        map.end()
    }
}

impl Graph {
    pub fn new() -> Graph {
        Graph {
            graph: HashMap::new(),
        }
    }

    pub fn add(&mut self, commit: &Commit) -> Result<(), Error> {
        let parent_ids = commit.parent_ids().collect();

        if let Some(ref mut node) = self.graph.get_mut(&commit.id()) {
            node.parents = parent_ids;
            return Ok(());
        }

        let mut node = Node::new();
        for parent_id in &parent_ids {
            let ref mut node = self.graph.entry(*parent_id).or_insert(Node::new());
            node.children.push(commit.id());
        }
        node.parents = parent_ids;
        self.graph.insert(commit.id(), node);
        Ok(())
    }

    pub fn children(&self, oid: &Oid) -> Option<&[Oid]> {
        self.graph.get(oid).map(|node| node.children.as_slice())
    }
}
