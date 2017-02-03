use std::fs::File;
use std::io::Read;

extern crate toml;


pub enum ResolverError {
    NoFile,
    BadSyntax,
}

pub struct Metadata {
    pub name    : String,
    pub version : String,
    pub deps    : Vec<Dependency>
}

pub struct Dependency {
    name        : String,
    min_version : String,
    max_version : String
}

pub trait Resolver {
    fn resolve( &self, name : String, version : String ) -> Result<Metadata, ResolverError>;
}



struct DummyResolver {}
impl Resolver for DummyResolver{
    fn resolve( &self, name : String, version : String ) -> Result<Metadata, ResolverError> {
        Ok(Metadata{ name : format!("{}", "vim"), version : format!("{}", "8.0"), deps : vec!() })
    }
}

pub struct FilesystemResolver {}
impl Resolver for FilesystemResolver{
    fn resolve( &self, name : String, version : String ) -> Result<Metadata, ResolverError> {
        let filename = format!("pkg/{}-{}.toml", name, version);

        let mut data = String::new();
        let mut f = File::open(filename).expect("Unable to open file");
        f.read_to_string(&mut data).expect("Unable to read string");

        // Unpack TOML data
        let value = toml::Parser::new(data.as_str()).parse().unwrap();

        // Extract data
        // Header
        let meta = value["metadata"].as_table().unwrap();

        // Rust is hard
        let version = format!("{}", meta["version"]);
        let name = format!("{}", meta["name"]);

        // Read dependencies
        let deps = value["depends"].as_table().unwrap();
        let mut dep_vector : Vec<Dependency> = vec!();
        for (_, val) in deps.iter() {
            let contents = val.as_table().unwrap();
            let d = Dependency{
                name : contents["name"].as_str().unwrap().to_string(),
                min_version : contents["minversion"].as_str().unwrap().to_string(),
                max_version : contents["maxversion"].as_str().unwrap().to_string()
            };
            dep_vector.push(d);
        }

        // Return metadata
        Ok(Metadata{ name : name, version : version, deps : dep_vector })
    }
}
