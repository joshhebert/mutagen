extern crate tar;
extern crate lzma;
extern crate libc;

use std::io::BufWriter;
use std::io::prelude::*;

use std::ffi::CString;

use std::os::raw::c_char;
use std::os::unix::fs::PermissionsExt;

use std::path::Path;

use std::fs::File;
use std::fs::create_dir;
use std::fs::Permissions;
use std::fs::set_permissions;

use self::libc::chown;

use self::tar::Archive;
use self::tar::EntryType;
use self::tar::Header;

use self::lzma::LzmaReader;

pub fn extract_xz( from_path : String, to_path : &Path ) {
    // Read in the xz compressed file
    let f = File::open( &from_path ).unwrap( );

    // Translate the file into a decompressed byte stream
    let mut xz_bytes : Vec<u8> = vec!();
    let mut f = LzmaReader::new_decompressor( f ).unwrap();

    // Convert byte stream to vector of bytes
    // No real good way to check if the number of decompressed bytes is correct
    // without trusting the lzma library
    let _ = f.read_to_end( &mut xz_bytes );

    // Generate a new tape archive object from the decompressed bytes
    let mut ar = Archive::new( xz_bytes.as_slice() );

    for file in ar.entries().unwrap() {
        // Make sure there wasn't an I/O error
        let mut file = file.unwrap();

        let target_path;
        {
            // Quarantine the immutable reference borrowed
            let file_path  = file.header().path().unwrap().clone();
            target_path = to_path.join( file_path );
        }

        // Check if this is a dir or a file. If it's a dir, we just need to copy
        // it and its attributes
        let et = file.header().entry_type();
        if et == EntryType::Directory {
            // TODO test if the dir exists first


            match create_dir( &target_path ) {
                Err(_) => panic!("Could not create dir during extract"),
                Ok(_) => {},
            }
            set_perms_from_header(file.header(), &target_path);
            continue;
        } else {
            // Otherwise, this is a file and we do a byte copy
            let mut f_bytes : Vec<u8> = vec!();
            let read_bytes = file.read_to_end( &mut f_bytes );
            match read_bytes {
                // Dangerous? Probably not.
                Ok(b) => assert!(b == file.header().size().unwrap() as usize),
                Err(_) => panic!("Couldn't read file during decompression"),
            }

            // Write bytes to the new path
            write_byte_buffer( f_bytes, &target_path );
            set_perms_from_header(file.header(), &target_path);
        }
    }
}


fn write_byte_buffer( bytes : Vec<u8>, path : &Path ) {
    let new_file = File::create( &path ).unwrap( );
    let mut writer = BufWriter::new( new_file );

    writer.write(bytes.as_slice()).unwrap();
    writer.flush().unwrap();
}


fn set_perms_from_header( header : &Header, path : &Path ){
    // Clone attributes of mode, user owner, gid owner,
    let mode : u32 = header.mode().unwrap();
    let fp : Permissions = Permissions::from_mode( mode );
    match set_permissions( path, fp ){
        Err(_) => panic!("Could not set permissions of dir during extract"),
        Ok(_) => {},
    }


    // Daily reminder that rust *still* doesn't have a fucking chown
    // function, so we have to unsafely invoke things from libc
    let u_owner : u32 = header.uid().unwrap();
    let g_owner : u32 = header.gid().unwrap();
    unsafe {
        // Shhhh... just let it happend
        let s1 = path.to_owned();
        let s2 = s1.to_str().unwrap();
        let path_cstring : *const c_char = CString::new(s2).unwrap().as_ptr();
        chown( path_cstring, u_owner, g_owner );
    }
}
