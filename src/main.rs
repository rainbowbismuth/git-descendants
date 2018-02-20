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

fn main() {
    let matches = App::new("git-descendants")
        .version("0.1")
        .author("Emily Amanda Bellows <emily.a.bellows@gmail.com>")
        .about("Calculates an adjacency list of commits")
        .arg(
            Arg::with_name("PATH_TO_GIT_REPO")
                .help("Optionally specify the git repository to use")
                .required(false)
                .index(1),
        )
        .arg(
            Arg::with_name("list-roots")
                .long("list-roots")
                .help("List the root commits of the graph"),
        )
        .get_matches();

    let path = matches.value_of("PATH_TO_GIT_REPO").unwrap_or(".");

    let runnable = if matches.is_present("list-roots") {
        print_roots
    } else {
        print_graph
    };

    match runnable(path) {
        Ok(()) => {}
        Err(err) => eprintln!("Error: {}", err),
    }
}
