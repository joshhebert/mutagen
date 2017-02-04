pub struct Version{
    // This is going to be a string delimited by either a . or a -
    // It can also have letters in it, i.e 1.0.2.k-1
    // The job of this struct is to make sense of it.
    pub data : String
}

impl Version {
    pub fn new( value : String ) -> Version {
        Version {
            data : value,
        }
    }

    ///
    /// -1 if other is greater than self
    /// 0 if equal
    /// 1 if self is greater than other
    pub fn cmp(&self, other: &Version) -> i32 {
        let delimiters = ['.', '-'];
        let mut self_tokens = self.data.split(&delimiters[..]);
        let mut other_tokens = other.data.split(&delimiters[..]);

        loop {
            match self_tokens.next() {
                Some(x) => {
                    match other_tokens.next() {
                        Some(y) => {
                            // We can compare tokens
                            let res = cmp_tok( x.to_string(), y.to_string() );
                            if res != 0 {
                                return res;
                            }
                        },
                        None => {
                            // If we got this far, and other ran out of tokens
                            // first, self is greater than other
                            return 1
                        }
                    }
                },
                None => {
                    match other_tokens.next() {
                        Some(_) => {
                            // If we got this far, and other ran out of tokens
                            // on self, but not other, other is greater than
                            // self
                            return -1
                        },
                        None => {
                            // Our strings our equal, therefore not less than
                            return 0;
                        }
                    }
                }
            }

        }
    }
}


// Helper functions
// 1 if x > y
// 0 if x == y
// -1 if x < y
fn cmp_tok( x : String, y : String ) -> i32 {
    // We make the assumption that a substring here
    // will either be all characters or all numbers
    // If it's a mix, operating procedure is to remove
    // all non-number characters and treat it like a
    // number


    // Attempt to parse to u32
    let tokx = x.parse::<u32>();
    let toky = y.parse::<u32>();
    match tokx {
        Ok(valx) => {
            match toky {
                Ok(valy) => {
                    // Both x and y parse to a u32
                    if valx > valy {
                        return 1;
                    }else if valx < valy {
                        return -1;
                    } else {
                        return 0;
                    }
                },
                Err(_) => {
                    // y doesn't parse to u32, but x did
                    panic!("Incompatible version strings");
                }
            }
        },
        Err(_) => {
            // x doesn't parse to u32, so continue
        },
    }

    // tokx did not parse to a u32 (if toky didn't we'd panic), so we
    // can assume we need to compare them as strings
    if x > y {
        return 1;
    }else if x < y {
        return -1;
    } else {
        return 0;
    }

    // TODO?
    // Maybe we don't need this
    // First test to ensure that the tokens aren't mixed char-int versions
    // If they are, strip the chars and recurse
    // Unfortunately, regex isn't really built for this, so we're better off
    // just iterating over charecters and testing if we have both

}
