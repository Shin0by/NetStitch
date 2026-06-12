fn main() {
    println!("Known connector manifests:");
    for manifest in netstitch_connectors::manifests() {
        let aliases = manifest
            .process_aliases
            .iter()
            .map(|alias| format!("{}:{}", alias.os.as_str(), alias.name))
            .collect::<Vec<_>>()
            .join(",");
        let sources = manifest
            .discovery_sources
            .iter()
            .map(|source| format!("{}:{}={}", source.os.as_str(), source.kind, source.detail))
            .collect::<Vec<_>>()
            .join(" | ");
        println!(
            "- {} ({}) processes={} aliases={} sources={} manual_only={}",
            manifest.display_name,
            manifest.id,
            manifest.process_names.join(","),
            aliases,
            sources,
            manifest.manual_only
        );
    }

    println!();
    println!("Detected installed applications:");
    let apps = netstitch_connectors::discover_installed_apps();
    if apps.is_empty() {
        println!("- none");
        return;
    }

    for app in apps {
        println!(
            "- {} ({}) process={} path={}",
            app.display_name,
            app.connector_id,
            app.process_name,
            app.exe_path.display()
        );
    }
}
