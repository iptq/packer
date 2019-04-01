use std::path::Path;

#[derive(Default)]
pub struct IgnoreFilter {
    #[cfg(feature = "ignore")]
    patterns: Vec<glob::Pattern>,
}

impl IgnoreFilter {
    #[cfg(feature = "ignore")]
    pub fn add_pattern(&mut self, pattern: impl AsRef<str>) {
        let pattern = glob::Pattern::new(pattern.as_ref()).expect("Could not compile pattern");
        self.patterns.push(pattern);
    }

    #[cfg(feature = "ignore")]
    pub fn should_ignore(&self, path: impl AsRef<Path>) -> bool {
        let path = path.as_ref();
        for pattern in &self.patterns {
            if pattern.matches_path(path) {
                return true;
            }
        }
        false
    }
}
