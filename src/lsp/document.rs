use serde_json::Value;

#[derive(Default)]
pub struct DocumentStore {
    docs: std::collections::HashMap<String, (i32, String)>,
    open: std::collections::HashSet<String>,
}
impl DocumentStore {
    pub fn open(&mut self, uri: &str, version: i32, text: String) {
        self.docs.insert(uri.into(), (version, text));
        self.open.insert(uri.into());
    }
    pub fn set_disk(&mut self, uri: &str, text: String) {
        if !self.open.contains(uri) { self.docs.insert(uri.into(), (0, text)); }
    }
    pub fn remove_disk(&mut self, uri: &str) {
        if !self.open.contains(uri) { self.docs.remove(uri); }
    }
    pub fn close(&mut self, uri: &str) {
        self.open.remove(uri);
        self.docs.remove(uri);
    }
    pub fn clear(&mut self) {
        self.docs.clear();
        self.open.clear();
    }
    pub fn change(&mut self, uri: &str, version: i32, changes: &[Value]) {
        if let Some((v, text)) = self.docs.get_mut(uri) {
            for change in changes {
                if let Some(range) = change.get("range") {
                    let start = position(text, &range["start"]);
                    let end = position(text, &range["end"]);
                    if start <= end {
                        text.replace_range(start..end, change["text"].as_str().unwrap_or(""));
                    }
                } else {
                    *text = change["text"].as_str().unwrap_or("").into();
                }
            }
            *v = version;
        }
    }
    pub fn text(&self, uri: &str) -> Option<&str> {
        self.docs.get(uri).map(|(_, text)| text.as_str())
    }
    pub fn uris(&self) -> impl Iterator<Item = &str> {
        self.docs.keys().map(String::as_str)
    }
}
pub fn position(text: &str, pos: &Value) -> usize {
    let line = pos["line"].as_u64().unwrap_or(0) as usize;
    let character = pos["character"].as_u64().unwrap_or(0) as usize;
    let start = text
        .split_inclusive('\n')
        .take(line)
        .map(str::len)
        .sum::<usize>();
    let line_text = &text[start..];
    let line_end = line_text.find('\n').unwrap_or(line_text.len());
    let line_text = &line_text[..line_end];
    let mut units = 0;
    for (offset, c) in line_text.char_indices() {
        if units >= character {
            return start + offset;
        }
        units += c.len_utf16();
        if units >= character {
            return start + offset + c.len_utf8();
        }
    }
    start + line_end
}
