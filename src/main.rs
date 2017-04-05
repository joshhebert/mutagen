mod solver;
use solver::context::Context;
use solver::package_resolver::FilesystemResolver;


mod archive;
use archive::xz::extract_xz;

mod fs;
use fs::mutagen_fs::MutagenFilesystem;
use fs::mutagen_fs::Tag;
use std::path::Path;

mod collector;
use collector::collector::collect_package;

use std::fs::create_dir_all;

extern crate fuse;

fn main() {

    // We first identify the list of dependencies we need to install for this
    // package
    let mut c = Context::new(FilesystemResolver{});
    c.inject("vim".to_string(), "7.4.1386-1".to_string());

    let dependencies = c.flatten("ROOT".to_string());


    let mut fs = MutagenFilesystem::new();
    // We then collect the packages, extract them, and load them to the vfs
    for (n,v) in dependencies {
        collect_package(n.clone(), v.data.clone());

        let pkg_name = format!("./root/tmp/mutagen/tmp_dl/{}-{}.tar.xz", n, v.data);
        let pkg_dir = format!("/home/josh/devel/mutagen/root/mutagen/pkg/{}/{}/", n, v.data);

        create_dir_all(pkg_dir.clone());

        extract_xz(pkg_name, Path::new(&pkg_dir));

        fs.inject(Path::new(&pkg_dir), Tag{
            owner_name: n.clone(),
            owner_version: v.data.clone(),
        });
    }

    // Launch the vfs
    let mountpoint = "./root/mutagen/vfs";
    fuse::mount(fs, &mountpoint, &[]).expect("Couldn't mount filesystem");
}
