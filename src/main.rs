use std::collections::HashMap;

mod version;
use version::Version;

mod package_resolver;
use package_resolver::Resolver;
use package_resolver::FilesystemResolver;


struct Ecosystem<T>{
    map : HashMap<String, Node>,
    resolver : T,
}

impl<T : Resolver> Ecosystem<T>{
    fn inject(&self){
        match self.resolver.resolve( "vim".to_string(), "4.5".to_string() ){
            Ok(s) => println!("{}", s.name),
            Err(_) => println!("Badthing")
        }
    }
}


struct Rule{
    min_version : Version,
    max_version : Version,
    owner       : String,
}

struct Node {
    name : String,
    rules : Vec<Rule>,
    deps : Vec<String>,
}
impl Node{
    fn collapse_rules(&self) -> Rule {
        // Iterate over all the max versions of the rules to find
        // the lowest

        //TODO test that we have rules

        // Duplicate our rules so that we don't f*ck up the iterator
        let mut lowest_max : &Version = &self.rules[0].max_version;
        let mut iter = self.rules.iter();
        loop {
            match iter.next() {
                Some(r) => {
                    if r.max_version.cmp(&lowest_max) == -1 {
                        lowest_max = &r.max_version;
                    }
                },
                None => break,
            }
        }

        let mut highest_min : &Version = &self.rules[0].min_version;
        iter = self.rules.iter();
        loop {
            match iter.next() {
                Some(r) => {
                    if r.min_version.cmp(&highest_min) == 1 {
                        highest_min = &r.min_version;
                    }
                },
                None => break,
            }
        }

        assert!( highest_min.cmp( lowest_max ) == -1 ||
                 highest_min.cmp( lowest_max ) == 0
               );


        let max = Version{ data : lowest_max.data.clone() };
        let min = Version{ data : highest_min.data.clone() };
        return Rule{ max_version: max,
                     min_version: min,
                     owner: "nobody".to_string()
                   };
    }
}

///
/// This function bothers me a lot, as it doesn't elegantly check
/// for key existance and *then* update, so we end up basically
/// testing twice. Furthermore, I'd prefer that owner of Rule be a
/// direct reference to the node, rather than a string that has to be
/// looked up every time.
///
fn add_rule<'a>( map  : &mut HashMap<String, Node>,
             from : &'a str,
             to   : &'a str,
             min  : &'a str,
             max  : &'a str ) {


    // Ensure that both from and to exist
    if !map.contains_key(from) || !map.contains_key(to){
        // Abort
        return;
    }

    // Create new rule
    // Abusing clone(), nbd
    let new_rule : Rule = Rule{ min_version : Version::new(min.to_string()),
                                max_version : Version::new(max.to_string()),
                                owner       : from.to_string().clone()
                              };

    // The panic!'s should never occur, but if they do, we should
    // abort, as it means there could be an inconsistency between
    // the from node and the to node
    match map.get_mut(to) {
        Some(n) => n.rules.push(new_rule),
        None => panic!("Bad thing")
    };

    match map.get_mut(from) {
        Some(n) => n.deps.push(to.to_string().clone()),
        None => panic!("Bad thing")
    };

}

///
/// Remove all rules owned by owner from target, and remove target as
/// a dependency of owner
///
fn remove_rule<'a>( map : &mut HashMap<String, Node>, owner : &'a str, target : &'a str ){
    // Ensure that both from and to exist
    if !map.contains_key(owner) || !map.contains_key(target){
        // Abort
        return;
    }

    // The panic!'s should never occur, but if they do, we should
    // abort, as it means there could be an inconsistency between
    // the from node and the to node
    match map.get_mut(owner) {
        Some(n) => n.deps.retain(|i| *i != target),
        None => panic!("Bad thing")
    };

    match map.get_mut(target) {
        Some(n) => n.rules.retain(|i| i.owner != owner),
        None => panic!("Bad thing")
    };
}

///
/// add_node(HashMap<&str,Node>, &str) -> void
/// Given a name node_name, insert a new node into the hashmap
///
fn add_node<'a>( map  : &mut HashMap<String, Node>,
             node_name : &'a str ) {
    let n = Node{ name : node_name.to_string().clone(), rules : vec!(), deps : vec!() };
    map.insert(node_name.to_string().clone(), n);

}





fn main() {
    let mut map : HashMap<String,Node> = HashMap::new();
    add_node( &mut map, "ROOT" );
    add_node( &mut map, "vim" );
    add_rule( &mut map, "ROOT", "vim", "1.0", "2.0" );
    add_rule( &mut map, "ROOT", "vim", "1.4", "2.5" );
    add_rule( &mut map, "ROOT", "vim", "0.5", "1.6" );

    match map.get_mut("vim") {
        Some(n) => {
            let r = n.collapse_rules();
            println!( "{}, {}", r.min_version.data, r.max_version.data );
        },
        None => {}
    };
    // remove_rule( &mut map, "ROOT", "vim" );

    // println!("Testing");
    // match map.get_mut("vim") {
    //     Some(n) => println!("{}", n.rules.len()),
    //     None => return
    // }


    // let map : HashMap<String, Node> = HashMap::new();
    // let r = FilesystemResolver{};
    // let e = Ecosystem{ map : map, resolver : r };

    // e.inject();
    // let v = Version::new("1.0.2.k-1".to_string());
    // let v2 = Version::new("1.0.2.j-1".to_string());
    // assert!( v.cmp(&v2) == 1 );
}
