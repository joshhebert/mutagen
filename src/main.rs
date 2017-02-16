// Attributes
#![feature(dedup_by)]

mod solver;
use solver::context::Context;
use solver::package_resolver::FilesystemResolver;



fn main() {
    let mut c = Context::new(FilesystemResolver{});
    c.inject("emacs".to_string(), "3.0".to_string());
    c.inject("vim".to_string(), "4.5".to_string());


    let d = c.flatten("ROOT".to_string());


    println!("Resolved tree:");
    let mut i = d.iter();
    loop {
        match i.next() {
            Some(r) => {
                println!("{}", r.0);
            },
            None => break,
        }
    }
}
