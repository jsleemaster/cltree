use std::collections::HashSet;
use std::fs;

fn load_labeler_labels() -> HashSet<String> {
    let contents =
        fs::read_to_string(".github/labeler.yml").expect("Failed to read .github/labeler.yml");

    contents
        .lines()
        .filter_map(|line| {
            let trimmed = line.trim();
            if line.starts_with(' ') || trimmed.is_empty() || trimmed.starts_with('#') {
                return None;
            }

            trimmed.strip_suffix(':').map(str::to_string)
        })
        .collect()
}

fn load_dependabot_labels() -> Vec<String> {
    let contents = fs::read_to_string(".github/dependabot.yml")
        .expect("Failed to read .github/dependabot.yml");

    let mut labels = Vec::new();
    let mut labels_indent = None;

    for line in contents.lines() {
        let trimmed = line.trim();
        let indent = line.len() - line.trim_start().len();

        if trimmed == "labels:" {
            labels_indent = Some(indent);
            continue;
        }

        if let Some(base_indent) = labels_indent {
            if !trimmed.is_empty() && indent <= base_indent {
                labels_indent = None;
                continue;
            }

            if let Some(label) = trimmed.strip_prefix("- ") {
                labels.push(label.trim_matches('"').to_string());
            }
        }
    }

    labels
}

#[test]
fn test_dependabot_labels_match_repository_label_set() {
    let repo_labels = load_labeler_labels();
    let dependabot_labels = load_dependabot_labels();

    assert!(
        !dependabot_labels.is_empty(),
        "Dependabot config should define at least one label"
    );

    for label in dependabot_labels {
        assert!(
            repo_labels.contains(&label),
            "Dependabot label '{label}' is not defined in .github/labeler.yml"
        );
    }
}
