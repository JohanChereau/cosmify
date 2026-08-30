use sysinfo::System;

pub(crate) fn minecraft_processes() -> Vec<String> {
    let system = System::new_all();
    let mut names = system
        .processes()
        .values()
        .filter_map(|process| {
            let name = process.name().to_string_lossy().into_owned();
            let lower = name.to_ascii_lowercase();
            let looks_like_minecraft = lower == "minecraft.windows.exe"
                || lower == "minecraft.windows"
                || lower == "minecraft.exe"
                || lower == "minecraft"
                || lower.contains("minecraft.windows");
            looks_like_minecraft.then_some(name)
        })
        .collect::<Vec<_>>();
    names.sort();
    names.dedup();
    names
}
