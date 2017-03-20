mod solver;
use solver::context::Context;
use solver::package_resolver::FilesystemResolver;


mod archive;
use archive::xz::extract_xz;

mod fs;

use std::path::Path;

fn example_solver_use(){
    let mut c = Context::new(FilesystemResolver{});
    c.inject("emacs".to_string(), "3.0".to_string());
    c.inject("vim".to_string(), "4.5".to_string());

    let d = c.flatten("ROOT".to_string());
    let mut i = d.iter();
    loop {
        match i.next() {
            Some(_) => {
                // println!("{}", r.0);
            },
            None => break,
        }
    }
}


fn main() {
    example_solver_use();
    extract_xz("./vim.tar.xz".to_string(), Path::new("/tmp/"));
}
