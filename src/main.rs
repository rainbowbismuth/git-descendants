// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

extern crate clap;
extern crate git2;
extern crate serde;
extern crate serde_json;

extern crate failure;

mod graph;
mod calculate;

use failure::Error;
use clap::{App, Arg};
use git2::Repository;

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
        println!(
            "{} {}",
            root.id(),
            root.message().unwrap_or("<no message>").trim()
        );
    }
    Ok(())
}

fn print_children(path: &str, revision: &str) -> Result<(), Error> {
    let repo = Repository::open(path)?;
    let revision_oid = repo.revparse_single(revision)?.id();
    let graph = calculate::graph_from_refs(&repo)?;
    if let Some(children) = graph.children(&revision_oid) {
        for child in children {
            println!("{}", child);
        }
        Ok(())
    } else {
        Ok(())
    }
}

fn main() {
    let matches = App::new("git-descendants")
        .version("0.1")
        .author("Emily Amanda Bellows <emily.a.bellows@gmail.com>")
        .about("Calculates an adjacency list of commits")
        .arg(
            Arg::with_name("repo_path")
                .help("Optionally specify the git repository to use")
                .long("repo-path")
                .short("p")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("list_roots")
                .long("list-roots")
                .help("List the root commits of the graph"),
        )
        .arg(
            Arg::with_name("REVISION")
                .help("The revision you wish to know the children of")
                .required(false)
                .index(1),
        )
        .get_matches();

    let path = matches.value_of("repo_path").unwrap_or(".");

    check_err(if let Some(revision) = matches.value_of("REVISION") {
        print_children(path, revision)
    } else if matches.is_present("list_roots") {
        print_roots(path)
    } else {
        print_graph(path)
    });
}

fn check_err(res: Result<(), Error>) {
    match res {
        Ok(()) => {}
        Err(err) => eprintln!("Error: {}", err),
    }
}
