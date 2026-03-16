#[cfg(feature = "web")]
#[tokio::test]
async fn test_ui_smoke() {
    // This test simulates starting the server and checking if the UI is served correctly.
    // Since we can't easily spin up the full Actix server in a unit test without binding ports,
    // we will verify the Dioxus app rendering logic directly if possible, or simulate a request.

    // However, Dioxus Fullstack integration is tight with the server.
    // We can test if the App component renders without panic.

    use dioxus::prelude::*;
    use rr_ui::ui::app::App;

    // Render the app to a string (Virtual DOM)
    let mut vdom = VirtualDom::new(App);
    let _ = vdom.rebuild_to_vec();

    // If we reached here, the App component structure is valid and didn't panic during initial render.
    assert!(true);
}

#[cfg(feature = "web")]
#[tokio::test]
#[ignore = "Requires dx build fullstack step to populate dist folder"]
async fn test_embedded_assets() {
    use rr_ui::ui::WebStatic;
    use rust_embed::RustEmbed;

    // Verify layout.css is embedded
    let css = WebStatic::get("layout.css");
    assert!(css.is_some(), "layout.css should be embedded");

    let css_file = css.unwrap();
    let css_content = std::str::from_utf8(&css_file.data).unwrap();
    assert!(
        css_content.contains("body"),
        "layout.css should contain CSS content"
    );

    // Verify favicon.svg is embedded
    let favicon = WebStatic::get("favicon.svg");
    assert!(favicon.is_some(), "favicon.svg should be embedded");
}
