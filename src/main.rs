use std::io;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use structopt::StructOpt;
use std::collections::HashMap;

mod app;
mod parser;
mod table;

#[derive(Debug, StructOpt)]
struct Opt {
    pub file: String,
}

fn parse_input(templates: Vec<HashMap<String, String>>) -> String {
    let mut input = String::default();
    let _ = io::stdin().read_line(&mut input).ok();
    let i = input.as_str();
    let mut result = String::default();
    let head = 
        app::App::build(vec![templates[0].clone()]);

    match head(i) {
        Ok((_rest, _rows)) =>  {
            result = input.clone();
            loop {
                let mut buf = String::default();
                let _ = io::stdin().read_line(&mut buf);
                let combinator = 
                    app::App::build(vec![templates.last().unwrap().clone()]);
                let r = combinator(&buf);

                if let Ok((_rest, rows)) = r {
                    result = format!("{}{}", result, buf);
                    if !rows.is_empty() {
                        break;
                    }
                } else {
                    result = format!("{}{}", result, buf);
                }
            }
        },
        _ => {}
    }

    dbg!(&result);

    result.clone()
}

fn main() -> anyhow::Result<()> {
    let opt = Opt::from_args();

    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();
    let app = app::App::load_from_file(opt.file.as_str())?; 
    let thandle = thread::spawn(move || {
        while running.load(Ordering::Relaxed) {
            for (k, ap) in app.iter() {
                let templates = ap.templates.clone();
                let input = parse_input(templates.clone());
                let combinate = app::App::build(templates.clone());

                match combinate(&input) {
                    Ok((_rest, rows)) => {
                        dbg!(&_rest);

                        table::printstd(&rows);
                    },
                    _ => {}
                }
            }
        }
    });

    ctrlc::set_handler(move || {
        r.store(false, Ordering::Relaxed);
    })
    .expect("Error setting Ctrl-C handler");

    let _ = thandle.join();

    Ok(())
}