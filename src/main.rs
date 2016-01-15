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

fn add_bookmark(url: &str, section: &str, tags: Vec<String>, bms:&mut Vec<Bookmark>) -> SqliteResult<Vec<Bookmark>> {
    let added = Bookmark {
        url: url.to_string(),
        section: section.to_string(),
        tags: tags,
    };
    bms.push(added);
    // let encoded = json::encode(&added).unwrap();
    let filename = "bookmarks.txt";
    //try expects a function to return a result
    let mut file = match File::create(filename) {
        Err(why) => panic!("Oops {}", why),
        Ok(file) => file,
    };

    let mut conn = try!(DatabaseConnection::in_memory());

    try!(conn.exec("CREATE TABLE bookmarks (
                 id              SERIAL PRIMARY KEY,
                 url            VARCHAR NOT NULL
               )"));

    let me = Person {
        id: 0,
        name: format!("Dan"),
        time_created: time::get_time(),
    };
    {
        let mut tx = try!(conn.prepare("INSERT INTO bookmarks (url)
                           VALUES ($1)"));
        let changes = try!(tx.update(&[&added.url]));
        assert_eq!(changes, 1);
    }

    // try!(file.write_fmt(format_args!("{}\n", url)));
    Ok(bms)
}

fn list_bookmarks() {
    let file = match File::open("bookmarks.txt") {
        Err(why) => panic!("Oops {}", why),
        Ok(file) => file,
    };
    let reader = BufReader::new(file);

    println!("Bookshelf:");
    //TODO make error safe
    for line in reader.lines() {
        let l = line.unwrap();
        println!("{}", l);
    }

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
        let result = add_bookmark(&url, &section, tags, &mut bookmarks);
        match result {
            Ok(s) => println!("All good"),
            Err(f) => panic!("Failed to add: {}", f),
        }
        return;
    }
    print_usage(&prog, opts);
}
