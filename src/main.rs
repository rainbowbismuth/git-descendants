// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

extern crate git2;
extern crate serde;
extern crate serde_json;

extern crate failure;

use failure::Error;
use git2::{Commit, Oid, Repository};
use serde::ser::{Serialize, SerializeMap, SerializeStruct, Serializer};
use std::collections::{HashMap, HashSet};

fn traverse_refs<'a>(repo: &'a Repository) -> Result<Vec<Commit<'a>>, Error> {
    let mut visited = HashSet::new();
    let mut queue = vec![];
    for reference in repo.references()? {
        if let Ok(commit) = reference?.peel_to_commit() {
            queue.push(commit.id());
        }
    }
    while let Some(commit_id) = queue.pop() {
        if !visited.insert(commit_id) {
            continue;
        }
        let commit = repo.find_commit(commit_id)?;
        for parent_id in commit.parent_ids() {
            queue.push(parent_id);
        }
    }
    let mut commits = vec![];
    for commit_id in visited {
        let commit = repo.find_commit(commit_id)?;
        commits.push(commit);
    }
    Ok(commits)
}

#[derive(Debug)]
struct Node {
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
    fn new() -> Node {
        Node {
            parents: vec![],
            children: vec![],
        }
    }

    fn parent_strings(&self) -> Vec<String> {
        oids_to_strings(&self.parents)
    }

    fn children_strings(&self) -> Vec<String> {
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
struct Graph {
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
    fn new() -> Graph {
        Graph {
            graph: HashMap::new(),
        }
    }

    fn add(&mut self, commit: &Commit) -> Result<(), Error> {
        let parent_ids: Vec<Oid> = commit.parent_ids().collect();

        if let Some(ref mut node) = self.graph.get_mut(&commit.id()) {
            node.parents = parent_ids;
            return Ok(());
        }

        let mut node = Node::new();
        for parent_id in &parent_ids {
            if let Some(ref mut node) = self.graph.get_mut(parent_id) {
                node.children.push(commit.id());
            }
        }
        node.parents = parent_ids;
        self.graph.insert(commit.id(), node);
        Ok(())
    }
}

fn calculate_descendents() -> Result<Graph, Error> {
    let repo = Repository::open(".")?;
    let mut graph = Graph::new();
    for commit in traverse_refs(&repo)? {
        graph.add(&commit)?;
    }
    Ok(graph)
}

fn print_graph() -> Result<(), Error> {
    let graph = calculate_descendents()?;
    println!("{}", serde_json::to_string_pretty(&graph)?);
    Ok(())
}

fn main() {
    match print_graph() {
        Ok(()) => {}
        Err(err) => eprintln!("Error: {}", err),
    }
}
