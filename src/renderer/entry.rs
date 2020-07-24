pub(crate) struct Entry {
    pub entry: ash::Entry,
}

impl Entry {
    pub fn new() -> Self {
        Self {
            entry: ash::Entry::new().unwrap(),
        }
    }
}
