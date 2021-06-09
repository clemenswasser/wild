pub struct Entry {
    pub entry: ash::Entry,
}

impl Entry {
    pub fn new() -> Self {
        Self {
            entry: unsafe { ash::Entry::new() }.unwrap(),
        }
    }
}
