extern crate sqlite3;
extern crate getopts;
use getopts::Options;
use std::env;

use sqlite3::{
    DatabaseConnection,
    Query,
    ResultRowAccess,
    SqliteResult,
    StatementUpdate,
};
use sqlite3::access;

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

fn add_bookmark(mut conn: DatabaseConnection, url: &str, tags: Vec<String>) -> SqliteResult<bool> {
    let added = Bookmark {
        url: url.to_string(),
        section: section.to_string(),
        tags: tags,
    };

    //TODO create tag if not exists.

	//TODO: move out to separate connection handler and do not run every time!
    try!(conn.exec("CREATE TABLE IF NOT EXISTS bookmarks (
                 id              SERIAL PRIMARY KEY,
                 url            VARCHAR NOT NULL
               )"));
   	try!(conn.exec("CREATE TABLE IF NOT EXISTS tags (
   				 id              SERIAL PRIMARY KEY,
   				 tag            VARCHAR NOT NULL
   			   )"));
   	try!(conn.exec("CREATE TABLE IF NOT EXISTS tags_bookmarks (
   				tag_id              SERIAL NOT NULL,
   				bookmark_id         SERIAL NOT NULL
   			  )"));

    {
        let mut tx = try!(conn.prepare("INSERT INTO bookmarks (url)
                           VALUES ($1)"));
        let changes = try!(tx.update(&[&added.url]));
        assert_eq!(changes, 1);
    }

    Ok(true)
}

fn list_bookmarks(conn: DatabaseConnection, term: String) -> SqliteResult<Vec<Bookmark>> {
    println!("Bookshelf:");

	let mut select_string = String::from("SELECT id, url FROM bookmarks");
	if term.len() > 0 {
		select_string.push_str(&format!(" WHERE url LIKE \"%{}%\"", term));
	}
    let mut stmt = try!(conn.prepare(&select_string));

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
    let args: Vec<_> = env::args().collect();
	let db_path = "bookshelf.db";
	let db_details = access::ByFilename { flags: Default::default(), filename: db_path };

	//TODO: error handle no connection
	let conn = DatabaseConnection::new(db_details).unwrap();

    let prog = &args[0];
    let mut opts = Options::new();
    opts.optopt("a", "add", "add url", "URL");
    //TODO opflagopt for section or tags?
    opts.optflagopt("l", "list", "list all bookmarks", "SEARCH_TERM");
    opts.optmulti("t", "tags", "tag bookmarks", "TAGS");
    //opts.optopt("s", "section", "Store in section", "SECTION[.SUBSECTION]");
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
		let term = matches.opt_str("l").unwrap_or(String::from(""));
        let bms = list_bookmarks(conn, term);
		for b in bms {
			println!("{:?}", b);
		}
        return;
    }
    if matches.opt_present("a") {
        let url = matches.opt_str("a").unwrap();
        println!("Adding {}", url);
        let tags = matches.opt_strs("t");
        let result = add_bookmark(conn, &url, tags);
        match result {
            Ok(s) => println!("All good: {}", s),
            Err(f) => panic!("Failed to add: {}", f),
        }
        return;
    }
    print_usage(&prog, opts);
}
