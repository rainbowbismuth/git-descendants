// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

extern crate colored;
extern crate git2;
extern crate serde;
extern crate serde_json;

#[macro_use]
extern crate clap;

#[macro_use]
extern crate failure;

mod graph;
mod calculate;
use colored::*;

use failure::{err_msg, Error};
use clap::{App, AppSettings, Arg, ArgMatches, SubCommand};
use git2::{Commit, Repository};

fn main() {
    let repo_path = Arg::with_name("repo_path")
        .help("Optionally specify the git repository to use")
        .long("repo-path")
        .short("p")
        .takes_value(true);

    let revision = Arg::with_name("REVISION")
        .help("The revision you wish to know the children of")
        .required(true)
        .index(1);

    let matches = App::new("git-descendants")
        .version(crate_version!())
        .author("Emily Amanda Bellows <emily.a.bellows@gmail.com>")
        .about("Calculates an adjacency list of commits")
        .setting(AppSettings::SubcommandRequiredElseHelp)
        .subcommand(
            SubCommand::with_name("graph")
                .about("Calculate and write out the commit graph")
                .arg(&repo_path),
        )
        .subcommand(
            SubCommand::with_name("roots")
                .about("Prints the roots of the graph")
                .arg(&repo_path),
        )
        .subcommand(
            SubCommand::with_name("children")
                .about("Print the child commits of a given revision")
                .arg(&repo_path)
                .arg(&revision),
        )
        .subcommand(
            SubCommand::with_name("lost")
                .about("Find commits that you can't get to normally")
                .arg(&repo_path),
        )
        .get_matches();

    match run_subcommand(matches) {
        Ok(()) => {}
        Err(err) => eprintln!("Error: {}", err),
    }
}

fn run_subcommand(matches: ArgMatches) -> Result<(), Error> {
    if let Some(matches) = matches.subcommand_matches("graph") {
        let path = matches.value_of("repo_path").unwrap_or(".");
        print_graph(path)
    } else if let Some(matches) = matches.subcommand_matches("roots") {
        let path = matches.value_of("repo_path").unwrap_or(".");
        print_roots(path)
    } else if let Some(matches) = matches.subcommand_matches("children") {
        let path = matches.value_of("repo_path").unwrap_or(".");
        let revision = matches
            .value_of("REVISION")
            .ok_or(err_msg("REVISION not specified"))?;
        print_children(path, revision)
    } else if let Some(matches) = matches.subcommand_matches("lost") {
        let path = matches.value_of("repo_path").unwrap_or(".");
        print_lost(path)
    } else {
        Err(format_err!(
            "Unknown subcommand, {}",
            matches.subcommand_name().unwrap()
        ))
    }
}

fn print_graph(path: &str) -> Result<(), Error> {
    let repo = Repository::open(path)?;
    let graph = calculate::graph_from_refs(&repo)?;
    println!("{}", serde_json::to_string_pretty(&graph)?);
    Ok(())
}

fn print_roots(path: &str) -> Result<(), Error> {
    let repo = Repository::open(path)?;
    let roots = calculate::root_commits_by_refs(&repo)?;
    for root in &roots {
        print_commit(root);
    }
    Ok(())
}

fn print_children(path: &str, revision: &str) -> Result<(), Error> {
    let repo = Repository::open(path)?;
    let revision_oid = repo.revparse_single(revision)?.id();
    let graph = calculate::graph_from_refs(&repo)?;
    if let Some(children) = graph.children(&revision_oid) {
        for child in children {
            let commit = repo.find_commit(*child)?;
            print_commit(&commit);
        }
    }
    Ok(())
}

fn print_lost(path: &str) -> Result<(), Error> {
    let repo = Repository::open(path)?;
    let all_commits = calculate::commits_only(&repo)?;
    let root_commits = calculate::root_commits_by_refs(&repo)?;
    let some_commits = calculate::traverse_from_roots(&repo, &root_commits)?;

    let mut commits_to_show = vec![];
    for (oid, commit) in all_commits {
        if some_commits.contains_key(&oid) {
            continue;
        }
        commits_to_show.push(commit);
    }

    commits_to_show.sort_by(|a,b| a.time().cmp(&b.time()));
    for commit in commits_to_show {
        print_commit_extended(&commit);
    }

    Ok(())
}

fn print_commit(commit: &Commit) {
    println!(
        "{} {}",
        commit.id(),
        commit.summary().unwrap_or("<no summary>").trim()
    )
}

fn print_commit_extended(commit: &Commit) {
    let commit_line = format!("commit {}\n", commit.id()).yellow();
    let author_line = format!("Author: {}\n", commit.author());
    let summary_line = format!(
        "     {}\n",
        commit.summary().unwrap_or("<no summary>").trim()
    );
    println!("{}{}{}", commit_line, author_line, summary_line)
}
