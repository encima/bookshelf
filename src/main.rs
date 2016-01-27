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
    tags: Vec<String>,
}

fn print_usage(prog: &str, opts: Options) {
    let brief = format!("Usage: {} CMD  url [tags]", prog);
    print!("{}", opts.usage(&brief));
}

fn add_bookmark(mut conn: DatabaseConnection, url: &str, tags: &Vec<String>) -> SqliteResult<bool> {
    // let added = Bookmark {
    //     url: url.to_string(),
    //     tags: *tags,
    // };

	//TODO: move out to separate connection handler and do not run every time!
    try!(conn.exec("CREATE TABLE IF NOT EXISTS bookmarks (
                 id             INTEGER PRIMARY KEY,
                 url            VARCHAR NOT NULL,
                 UNIQUE(url)
               )"));
   	try!(conn.exec("CREATE TABLE IF NOT EXISTS tags (
   				 id             INTEGER PRIMARY KEY,
   				 tag            VARCHAR NOT NULL,
                 UNIQUE(tag)
   			   )"));
   	try!(conn.exec("CREATE TABLE IF NOT EXISTS tags_bookmarks (
   				tag_id              NUMBER NOT NULL,
   				bookmark_id         NUMBER NOT NULL
   			  )"));

      //TODO create tag if not exists.
      for tag in tags {
          let mut tx = try!(conn.prepare("INSERT INTO tags (tag) VALUES ($1)"));
          let changes = try!(tx.update(&[tag]));
          assert_eq!(changes, 1);
      }

    {
        let mut tx = try!(conn.prepare("INSERT INTO bookmarks (url)
                           VALUES ($1)"));
        let changes = try!(tx.update(&[&url.to_string()]));
        assert_eq!(changes, 1);
    }

    Ok(true)
}

fn list_bookmarks(conn: DatabaseConnection, term: String) -> SqliteResult<Vec<String>> {
    println!("Bookshelf:");

	let mut select_string = String::from("SELECT b.url FROM bookmarks b");
	if term.len() > 0 {
		select_string.push_str(&format!(", tags t, `tags_bookmarks` tb
        WHERE t.id = tb.`tag_id` AND b.id = tb.`bm_id` AND t.`tag` LIKE \"%{}%\" OR b.url LIKE \"%{}%\"", term, term));
	}
    let mut stmt = try!(conn.prepare(&select_string));

    let mut bms = vec!();

    try!(stmt.query(
        &[], &mut |row| {
            bms.push(row.get(0));
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
        let result = add_bookmark(conn, &url, &tags);
        match result {
            Ok(s) => println!("All good: {}", s),
            Err(f) => panic!("Failed to add: {}", f),
        }
        return;
    }
    print_usage(&prog, opts);
}
