pub fn unique_temp_path(label: &str) -> std::path::PathBuf {
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("system clock should be after unix epoch")
        .as_nanos();
    std::env::temp_dir().join(format!(
        "blind-browser-{label}-{}-{timestamp}",
        std::process::id()
    ))
}

pub fn write_skill_document(root: &std::path::Path, skill_name: &str, content: &str) {
    let skill_dir = root.join(skill_name);
    std::fs::create_dir_all(&skill_dir).expect("skill directory should be created");
    std::fs::write(skill_dir.join("SKILL.md"), content).expect("skill document should be written");
}
