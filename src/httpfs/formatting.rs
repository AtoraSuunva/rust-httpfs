use crate::filesystem::DirEntry;

impl DirEntry {
    pub fn html_format(&self) -> String {
        let name = self.name.replace('<', "&lt;").replace('>', "&gt;");

        let link = if self.is_directory {
            format!("<a href=\"{}/\">{}</a>", name, name)
        } else {
            format!("<a href=\"{}\">{}</a>", name, name)
        };

        format!("<li>{}</li>", link)
    }

    pub fn plaintext_format(&self) -> String {
        format!(
            "{}{} [{}]",
            self.name,
            if self.is_directory { "/" } else { "" },
            if self.is_directory {
                "dir"
            } else {
                self.mime.essence_str()
            },
        )
    }
}
