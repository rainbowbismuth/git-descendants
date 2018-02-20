// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use failure::Error;
use git2::{Commit, Oid, Repository};
use std::collections::HashMap;
use graph::Graph;

pub fn root_commits_by_refs<'a>(repo: &'a Repository) -> Result<Vec<Commit<'a>>, Error> {
    let mut commits = vec![];
    for reference in repo.references()? {
        if let Ok(commit) = reference?.peel_to_commit() {
            commits.push(commit);
        }
    }
    Ok(commits)
}

pub fn traverse_from_roots<'a>(
    repo: &'a Repository,
    commits: &[Commit<'a>],
) -> Result<HashMap<Oid, Commit<'a>>, Error> {
    let mut visited = HashMap::new();
    let mut queue = commits.iter().map(|c| c.id()).collect::<Vec<_>>();
    while let Some(commit_id) = queue.pop() {
        if visited.contains_key(&commit_id) {
            continue;
        }
        let commit = repo.find_commit(commit_id)?;
        for parent_id in commit.parent_ids() {
            queue.push(parent_id);
        }
        visited.insert(commit_id, commit);
    }
    Ok(visited)
}

pub fn graph_from_refs(repo: &Repository) -> Result<Graph, Error> {
    let mut graph = Graph::new();
    let roots = root_commits_by_refs(&repo)?;
    for commit in traverse_from_roots(&repo, &roots)?.values() {
        graph.add(&commit);
    }
    Ok(graph)
}
