use mime_guess::{mime, Mime};

use crate::filesystem::DirEntry;

impl DirEntry {
    pub fn html_format(&self) -> String {
        let name = self.name.replace('<', "&lt;").replace('>', "&gt;");
        let mime = self.mime.essence_str();

        let link = if self.is_directory {
            format!("📁 <a href=\"{}/\">{}/</a>", name, name)
        } else {
            format!(
                "{} <a href=\"{}\">{}</a> <span>{}</span>",
                get_mime_emoji(&self.mime),
                name,
                name,
                mime
            )
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

fn get_mime_emoji(mime: &Mime) -> String {
    match mime.type_() {
        mime::TEXT => "📝".to_string(),
        mime::IMAGE => "🖼️".to_string(),
        mime::AUDIO => "🎵".to_string(),
        mime::VIDEO => "🎥".to_string(),
        mime::APPLICATION => match mime.subtype() {
            mime::CSS => "🎨".to_string(),
            mime::JAVASCRIPT => "📜".to_string(),
            mime::JSON => "📜".to_string(),
            mime::XML => "📜".to_string(),
            mime::PDF => "📓".to_string(),
            _ => "📄".to_string(),
        },
        _ => "📄".to_string(),
    }
}
