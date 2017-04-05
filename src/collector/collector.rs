extern crate curl;

use std::fs::File;
use std::io::prelude::*;
use std::path::Path;
use std::fs;
use self::curl::easy::Easy;

// Given a name and version, collect the archive to the dir, where it can then
// be unarchived.
pub fn collect_package( name : String, version : String ) -> bool {
    // Hardcoded for now
    let pkg_name : String = format!("{}-{}.tar.xz", name, version);
    let remote_server = "http://127.0.0.1";
    let port : u64 = 8000;

    // If using a nonstandard port, we need to use a slightly different URL
    let url;
    if port != 80 && port != 443 {
        url = format!("{}:{}/{}", remote_server, port, pkg_name);
    }else{
        url = format!("{}/{}", remote_server, pkg_name);
    }


    let mut easy = Easy::new();
    easy.url( url.as_str() ).unwrap();

    let mut dst  = Vec::new();
    // Scoping is necessry to drop the mutable reference to dst in the
    // callback
    {
        let mut transfer = easy.transfer();
        transfer.write_function(|data| {
            dst.extend_from_slice(data);
            Ok(data.len())
        }).unwrap();
        transfer.perform().unwrap();
    }
    // Data is now in dst


    let target_dir : &Path = Path::new("./root/tmp/mutagen/tmp_dl");
    if target_dir.exists() && target_dir.is_dir() {
        let to = target_dir.join(Path::new(&pkg_name));
        let mut file = File::create(to).unwrap();
        file.write_all(dst.as_slice()).unwrap();
    }

    return true;
}
