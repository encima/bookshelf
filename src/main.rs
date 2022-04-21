extern crate rusqlite;
extern crate getopts;
use getopts::Options;
use std::env;
use std::process::Command;

use rusqlite::{Connection, Result};

#[derive(Debug)]
pub struct Bookmark  {
    url: String,
    tags: Vec<String>,
}

fn print_usage(prog: &str, opts: Options) {
    let brief = format!("Usage: {} CMD  url [tags]", prog);
    print!("{}", opts.usage(&brief));
}

fn add_bookmark(conn: Connection, url: &str, tags: &Vec<String>) -> Result<bool> {
    // let added = Bookmark {
    //     url: url.to_string(),
    //     tags: *tags,
    // };
	//TODO: move out to separate connection handler and do not run every time!
    conn.execute("CREATE TABLE IF NOT EXISTS bookmarks (
                 id             INTEGER PRIMARY KEY,
                 url            VARCHAR NOT NULL,
                 UNIQUE(url)
               )", [])?;
   	conn.execute("CREATE TABLE IF NOT EXISTS tags (
   				 id             INTEGER PRIMARY KEY,
   				 tag            VARCHAR NOT NULL,
                 UNIQUE(tag)
   			   )", [])?;
   	conn.execute("CREATE TABLE IF NOT EXISTS tags_bookmarks (
   				tag_id              NUMBER NOT NULL,
   				bookmark_id         NUMBER NOT NULL
   			  );", [])?;

      let tag_ids: Vec<i32> = Vec::new();

      for tag in tags {
          let mut tx = conn.prepare("INSERT OR IGNORE INTO tags (tag) VALUES ($1)")?;
          tx.execute(&[tag])?;
          //GET INSERTED TAG ID
        //   let mut stmt = conn.prepare(&format!("SELECT id FROM tags WHERE tag=\"{}\"", tag))?;
        //   let rows = stmt.query([]);
      }

    let bm_id = 0;
    {
        let mut tx = conn.prepare("INSERT INTO bookmarks (url) VALUES ($1)")?;
        tx.execute(&[&url.to_string()])?;
        // assert_eq!(changes, 1);
        // let mut stmt = conn.prepare(&format!("SELECT id FROM bookmarks WHERE url=\"{}\"", url.to_string()))?;
        // let rows = stmt.query([]);
    }

    println!("{}", tag_ids.len());
    for id in tag_ids {
        let mut tx = conn.prepare("INSERT INTO tags_bookmarks (tag_id, bookmark_id) VALUES ($1, $2)")?;
        let changes = tx.execute(&[&id, &bm_id])?;
        assert_eq!(changes, 1);
    }

    Ok(true)
}

fn get_bookmarks(conn: &Connection, term: String) -> Result<Vec<String>> {
    let mut select_string = String::from("SELECT DISTINCT b.url FROM bookmarks b");
    if term.len() > 0 {
		select_string.push_str(&format!(", tags t, tags_bookmarks tb
        WHERE t.id = tb.tag_id AND b.id = tb.bookmark_id AND t.tag LIKE \"%{}%\" OR b.url LIKE \"%{}%\"", term, term));
    }
    let mut stmt = conn.prepare(&select_string)?;
    let mut rows = stmt.query([])?;

    let mut bms = Vec::new();
    while let Some(row) = rows.next()? {
        bms.push(row.get(0)?);
    }

    Ok(bms)
}

fn main() {
    let args: Vec<_> = env::args().collect();
	let db_path = "bookshelf.db";

	//TODO: error handle no connection
	let conn = Connection::open(&db_path).unwrap();

    let prog = &args[0];
    let mut opts = Options::new();
    opts.optopt("a", "add", "add url", "URL");
    opts.optopt("o", "open", "open urls matching name or tag", "URL/TAG");
    opts.optflagopt("l", "list", "list all bookmarks", "SEARCH_TERM");
    opts.optmulti("t", "tags", "tag bookmarks", "TAGS");
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
        let bms = get_bookmarks(&conn, term).unwrap();
		for b in bms {
			println!("{:?}", b);
		}
        return;
    }
    if matches.opt_present("o") {
		let term = matches.opt_str("o").unwrap_or(String::from(""));
        let bms = get_bookmarks(&conn, term).unwrap();
		for b in bms {
			println!("{:?}", b);
        //     //TODO: open across all platforms
        //     //TODO: handle http/https
            Command::new("open").arg(b).spawn().ok().expect("Failed to execute.");
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
