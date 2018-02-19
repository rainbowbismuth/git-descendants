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
use git2::{Repository, Odb, OdbObject, Oid};

fn list_oids(odb: &Odb) -> Result<Vec<Oid>, Error> {
    let mut oids = vec![];
    odb.foreach(|oid| {
        oids.push(oid.clone());
        true
    })?;
    Ok(oids)
}

fn calculate_descendents() -> Result<(), Error> {
    let repo = Repository::open(".")?;
    let odb = repo.odb()?;
    for oid in list_oids(&odb)? {
        let object = odb.read(oid)?;
        println!("{:>6} {}", object.kind(), oid);
    }

    Ok(())
}


fn main() {
    match calculate_descendents() {
        Ok(()) => {},
        Err(err) => eprintln!("Error: {}", err)
    }
}
