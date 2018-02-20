use failure::Error;
use git2::{Commit, Oid, Repository};
use std::collections::HashSet;
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
) -> Result<Vec<Commit<'a>>, Error> {
    let mut visited = HashSet::new();
    let mut queue = commits.iter().map(|x| x.id()).collect::<Vec<Oid>>();
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

pub fn graph_from_refs(repo: &Repository) -> Result<Graph, Error> {
    let mut graph = Graph::new();
    let roots = root_commits_by_refs(&repo)?;
    for commit in traverse_from_roots(&repo, &roots)? {
        graph.add(&commit)?;
    }
    Ok(graph)
}
