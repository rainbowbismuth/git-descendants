// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

extern crate git2;
extern crate serde;
extern crate serde_json;

#[macro_use] extern crate failure;
#[macro_use] extern crate serde_derive;

use failure::Error;
use git2::{Repository, Odb, OdbObject, ObjectType, Commit, Oid};

fn list_oids(odb: &Odb) -> Result<Vec<Oid>, Error> {
    let mut oids = vec![];
    odb.foreach(|oid| {
        oids.push(oid.clone());
        true
    })?;
    Ok(oids)
}

fn commits_only<'a>(repo: &'a Repository, odb: &Odb) -> Result<Vec<Commit<'a>>, Error> {
    let mut commits = vec![];
    for oid in list_oids(odb)? {
        let object = odb.read(oid)?;
        if object.kind() == ObjectType::Commit {
            let commit = repo.find_commit(oid)?;
            commits.push(commit);
        }
    }
    Ok(commits)
}

fn calculate_descendents() -> Result<(), Error> {
    let repo = Repository::open(".")?;
    let odb = repo.odb()?;
    for commit in commits_only(&repo, &odb)? {
        println!("{} {}", commit.id(), commit.message().unwrap_or("").trim());
    }
    Ok(())
}

fn main() {
    match calculate_descendents() {
        Ok(()) => {},
        Err(err) => eprintln!("Error: {}", err)
    }
}
