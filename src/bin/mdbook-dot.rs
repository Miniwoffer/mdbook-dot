
extern crate clap;
extern crate mdbook;
extern crate serde_json;

use clap::{App, Arg, ArgMatches, SubCommand};

use std::io;
use std::process::{Command,Stdio};
use std::io::Write;
use std::fs;
use std::process;

use mdbook::book::{Book,BookItem,Chapter};
use mdbook::errors::{Error,Result};
use mdbook::preprocess::{Preprocessor, PreprocessorContext, CmdPreprocessor};
pub fn make_app() -> App<'static, 'static> {
    App::new("dot-preprocessor")
        .about("A mdbook preprocessor so genereate dot graphs.")
        .subcommand(
            SubCommand::with_name("supports")
                .arg(Arg::with_name("renderer").required(true))
                .about("Check whether a renderer is supported by this preprocessor"),
        )
}

fn main() {
    let matches = make_app().get_matches();

    let mdd = MdbookDot::new();



    if let Some(sub_args) = matches.subcommand_matches("supports") {
        process::exit(0);
    }

    let (ctx, book) = CmdPreprocessor::parse_input(io::stdin()).unwrap();
    let processed_book = mdd.run(&ctx,book).unwrap();
    serde_json::to_writer(io::stdout(), &processed_book);
}

const NAME: &str = "mdbook-dot";


struct MdbookDot;

impl MdbookDot {
    pub fn new() -> MdbookDot {
        MdbookDot
    }
}

impl Preprocessor for MdbookDot {
    fn name(&self) -> &str {
       NAME 
    }
    fn run(&self, _ctx: &PreprocessorContext, mut book: Book) -> Result<Book> {
        eprintln!("Running '{}' preprocessor",self.name());
        let mut num_replaced_items = 0;
        process(&mut book.sections, &mut num_replaced_items);
        Ok(book)
    }
    fn supports_renderer(&self, renderer: &str) -> bool {
        true
    }
}
fn process <'a,I>(items: I, num_replaced_items: &mut usize) -> Result<()>
where 
    I : IntoIterator<Item = &'a mut BookItem> + 'a,
{
    for item in items {
        if let BookItem::Chapter(ref mut chapter) = *item {
            eprintln!("{}: processing chapter '{}'",NAME,chapter.name);
            let mut i : usize = 0;
            loop {
                match chapter.content.find("\n```dot\n") {
                    Some(start) => {

                        let end = { 
                            let (front,back) = chapter.content.split_at(start);
                            match back.find("\n```\n") { Some(e) => {e}
                                None => {
                                    eprintln!("err");
                                    Error::from(format!("Failed to generate Dot: dot not closed"));
                                    break;
                                }
                            }
                        };
                        let st : String = chapter.content.drain(start+8..start+end).collect(); 
                        let ret = match dot_to_image(st,format!("figure{}",i)) {
                            Ok(s) => {
                                s
                            },
                            Err(_) => {
                                format!("failed to parse")
                            }
                        };
                        i = i + 1;
                        chapter.content.replace_range(start..start+12,&ret);
                    }
                    None => {
                        break;
                    }
                } 
            }

        }
    }
    Ok(())
}
fn dot_to_image(input : String,path : String) -> Result<String> {
    let mut dot = Command::new("/usr/bin/dot")
        .arg("-Tsvg")
        .stdin(Stdio::piped()).stdout(Stdio::piped())
        .spawn().expect("Cant find \"dot\"");
    {
        let mut stdin = dot.stdin.as_mut().expect("Failed to open stdin");
        stdin.write_all(input.as_bytes()).expect("Failed to write to stdin");
    }
    let ret = dot.wait_with_output().expect("failed to wait for dot");
    let mut ret = String::from_utf8(ret.stdout).expect("failed to string");
    let start = ret.find("width=").expect("");
    let mut num = 0;
    loop {
        let s : String = ret.drain(start..start+1).collect();
        if s == "\"" {
            num = num + 1;
        }
        if num == 4 {
            break;
        }
    }
    Ok(ret)
}
