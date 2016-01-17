extern crate sqlite3;
extern crate getopts;
use std::io::prelude::*;
use getopts::Options;
use std::env;

use sqlite3::{
    DatabaseConnection,
    Query,
    ResultRowAccess,
    SqliteResult,
    StatementUpdate,
};

#[derive(Debug)]
pub struct Bookmark  {
    url: String,
    section: String,
    tags: Vec<String>,
}

fn print_usage(prog: &str, opts: Options) {
    let brief = format!("Usage: {} CMD  url [tags]", prog);
    print!("{}", opts.usage(&brief));
}

fn add_bookmark(url: &str, section: &str, tags: Vec<String>) -> SqliteResult<bool> {
    let mut added = Bookmark {
        url: url.to_string(),
        section: section.to_string(),
        tags: tags,
    };

	//TODO: open in file, when not in dev, and pass by reference
    let mut conn = try!(DatabaseConnection::in_memory());

	//TODO: move out to separate connection handler
    try!(conn.exec("CREATE TABLE IF NOT EXISTS bookmarks (
                 id              SERIAL PRIMARY KEY,
                 url            VARCHAR NOT NULL
               )"));

    {
        let mut tx = try!(conn.prepare("INSERT INTO bookmarks (url)
                           VALUES ($1)"));
        let changes = try!(tx.update(&[&added.url]));
        assert_eq!(changes, 1);
    }

    Ok(true)
}

fn list_bookmarks() -> SqliteResult<Vec<Bookmark>> {
    println!("Bookshelf:");
    let mut conn = try!(DatabaseConnection::in_memory());

	//TODO: move out to separate connection handler
    let mut stmt = try!(conn.prepare("SELECT id, url FROM bookmarks"));

    let mut bms = vec!();
    try!(stmt.query(
        &[], &mut |row| {
            bms.push(Bookmark {
                url: row.get(1),
				section: "".to_string(),
				tags: vec!(),
            });
            Ok(())
        }));
    Ok(bms)
}

fn main() {
    let mut bookmarks: Vec<Bookmark> = Vec::new();
    let args: Vec<_> = env::args().collect();

    let prog = &args[0];
    let mut opts = Options::new();
    opts.optopt("a", "add", "add url", "URL");
    //TODO opflagopt for section or tags?
    opts.optflag("l", "list", "list all bookmarks");
    opts.optmulti("t", "tags", "tag bookmarks", "TAGS");
    opts.optopt("s", "section", "Store in section", "SECTION[.SUBSECTION]");
    opts.optflag("h", "help", "print this help menu");
    let matches = match opts.parse(&args[1..]) {
        Ok(m) => { m }
        Err(f) => { panic!(f.to_string()) }
    };
    if matches.opt_present("h") {
        print_usage(&prog, opts);
        return;
    }
    if matches.opt_present("l") {
        list_bookmarks();
        return;
    }
    if matches.opt_present("a") {
        let url = matches.opt_str("a").unwrap();
        println!("Adding {}", url);
        let section = matches.opt_str("s").unwrap_or(String::from("inbox"));
        let tags = matches.opt_strs("t");
        let result = add_bookmark(&url, &section, tags);
        match result {
            Ok(s) => println!("All good"),
            Err(f) => panic!("Failed to add: {}", f),
        }
        return;
    }
    print_usage(&prog, opts);
}
