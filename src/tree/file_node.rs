use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct FileNode {
    pub path: PathBuf,
    pub name: String,
    pub depth: usize,
    pub is_dir: bool,
}

impl FileNode {
    pub fn new(path: PathBuf, name: String, depth: usize, is_dir: bool) -> Self {
        Self {
            path,
            name,
            depth,
            is_dir,
        }
    }

    #[allow(dead_code)]
    pub fn icon(&self) -> &'static str {
        if self.is_dir {
            "▸ "
        } else {
            "· "
        }
    }

    pub fn expanded_icon(&self, expanded: bool) -> &'static str {
        if self.is_dir {
            if expanded {
                "▾ "
            } else {
                "▸ "
            }
        } else {
            "· "
        }
    }

    pub fn display_color(&self) -> ratatui::style::Color {
        use ratatui::style::Color;
        if self.is_dir {
            Color::Rgb(209, 164, 73)
        } else {
            self.file_type_color()
        }
    }

    fn file_type_color(&self) -> ratatui::style::Color {
        use ratatui::style::Color;
        let ext = self.path.extension().and_then(|e| e.to_str()).unwrap_or("");

        match ext.to_lowercase().as_str() {
            // Rust
            "rs" => Color::Rgb(255, 150, 50),
            // JavaScript/TypeScript
            "js" | "mjs" | "cjs" => Color::LightYellow,
            "ts" | "mts" | "cts" => Color::Rgb(50, 150, 255),
            "jsx" | "tsx" => Color::Rgb(100, 200, 255),
            // Python
            "py" | "pyw" | "pyi" => Color::Rgb(80, 180, 80),
            // Web
            "html" | "htm" => Color::Rgb(230, 120, 50),
            "css" | "scss" | "sass" | "less" => Color::Rgb(180, 100, 255),
            "vue" | "svelte" => Color::LightGreen,
            // Data/Config
            "json" => Color::LightYellow,
            "yaml" | "yml" | "toml" => Color::Rgb(180, 180, 180),
            "xml" => Color::Rgb(200, 150, 50),
            "sql" => Color::Rgb(200, 200, 50),
            // Docs
            "md" | "markdown" => Color::Rgb(100, 180, 255),
            "txt" => Color::Rgb(180, 180, 180),
            // Shell
            "sh" | "bash" | "zsh" | "fish" => Color::LightGreen,
            // Go
            "go" => Color::Cyan,
            // Java/Kotlin
            "java" => Color::Rgb(255, 100, 100),
            "kt" | "kts" => Color::Rgb(200, 120, 255),
            // C/C++
            "c" | "h" => Color::Rgb(100, 150, 255),
            "cpp" | "cc" | "cxx" | "hpp" => Color::Rgb(100, 150, 255),
            // Ruby
            "rb" => Color::LightRed,
            // Config/Lock
            "lock" => Color::DarkGray,
            "env" | "gitignore" | "dockerignore" => Color::DarkGray,
            // Images
            "png" | "jpg" | "jpeg" | "gif" | "svg" | "ico" | "webp" => Color::LightMagenta,
            // Default
            _ => Color::Rgb(180, 180, 180),
        }
    }

    #[allow(dead_code)]
    pub fn tree_prefix(&self, is_last: bool) -> String {
        if self.depth == 0 {
            return String::new();
        }

        let mut prefix = String::new();
        for _ in 0..self.depth.saturating_sub(1) {
            prefix.push_str("│   ");
        }

        if is_last {
            prefix.push_str("└── ");
        } else {
            prefix.push_str("├── ");
        }

        prefix
    }
}
