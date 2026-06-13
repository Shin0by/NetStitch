pub const APP_TITLE: &str = "NetStitch";

pub fn app_window_title() -> String {
    app_window_title_with_version(&netstitch_shared::runtime_build_version())
}

fn app_window_title_with_version(version: &str) -> String {
    format!(
        "NetStitch ver. {version} ({})",
        netstitch_shared::runtime_platform_label()
    )
}

#[cfg(test)]
mod tests {
    use super::{app_window_title, app_window_title_with_version};

    #[test]
    fn window_title_uses_runtime_version_contract() {
        assert_eq!(
            app_window_title_with_version("1.1.0.1"),
            format!(
                "NetStitch ver. 1.1.0.1 ({})",
                netstitch_shared::runtime_platform_label()
            )
        );
        assert!(app_window_title().starts_with("NetStitch ver. "));
        assert!(
            app_window_title()
                .ends_with(&format!("({})", netstitch_shared::runtime_platform_label()))
        );
    }
}
