pub struct Version{
    major : u32,
    minor : u32,
    patch : u32,
}

impl Version {
    fn less_than(&self, other: &Version) -> bool {
        // Ew
        if self.major < other.major {
            return true;
        }else if self.major > other.major {
            return false;
        }

        // Major versions are the same
        if self.minor < other.minor {
            return true;
        }else if self.minor > other.minor {
            return false;
        }

        // Minor versions are the same
        if self.patch < other.patch {
            return true;
        }else if self.patch > other.patch {
            return false;
        }

        // Same version string
        return false;
    }
}
