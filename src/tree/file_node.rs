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
            "📁"
        } else {
            self.file_icon()
        }
    }

    pub fn expanded_icon(&self, expanded: bool) -> &'static str {
        if self.is_dir {
            if expanded {
                "📂"
            } else {
                "📁"
            }
        } else {
            self.file_icon()
        }
    }

    fn file_icon(&self) -> &'static str {
        let ext = self.path.extension().and_then(|e| e.to_str()).unwrap_or("");

        match ext.to_lowercase().as_str() {
            // Rust
            "rs" => "🦀",
            // JavaScript/TypeScript
            "js" | "mjs" | "cjs" => "📜",
            "ts" | "mts" | "cts" => "📘",
            "jsx" | "tsx" => "⚛️",
            // Web
            "html" | "htm" => "🌐",
            "css" | "scss" | "sass" | "less" => "🎨",
            "vue" | "svelte" => "💚",
            // Python
            "py" | "pyw" | "pyi" => "🐍",
            // Data
            "json" => "📋",
            "yaml" | "yml" => "📝",
            "toml" => "⚙️",
            "xml" => "📰",
            "csv" => "📊",
            "sql" => "🗃️",
            // Docs
            "md" | "markdown" => "📖",
            "txt" => "📄",
            "pdf" => "📕",
            "doc" | "docx" => "📘",
            // Config
            "env" => "🔐",
            "gitignore" | "dockerignore" => "🙈",
            "lock" => "🔒",
            // Shell
            "sh" | "bash" | "zsh" | "fish" => "🐚",
            "ps1" | "bat" | "cmd" => "💻",
            // Images
            "png" | "jpg" | "jpeg" | "gif" | "svg" | "ico" | "webp" => "🖼️",
            // Go
            "go" => "🐹",
            // Java/Kotlin
            "java" => "☕",
            "kt" | "kts" => "🟣",
            // C/C++
            "c" | "h" => "🔵",
            "cpp" | "cc" | "cxx" | "hpp" => "🔷",
            // Ruby
            "rb" => "💎",
            // PHP
            "php" => "🐘",
            // Swift
            "swift" => "🦅",
            // Misc
            "zip" | "tar" | "gz" | "rar" | "7z" => "📦",
            "log" => "📋",
            "exe" | "dll" | "so" | "dylib" => "⚡",
            _ => "📄",
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
