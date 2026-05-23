use rr_ui::ui::app::Route;
use dioxus::prelude::*;
use std::str::FromStr;

#[test]
fn test_route_parsing() {
    // Verify that "/" matches Home
    let route = Route::from_str("/").unwrap();
    assert_eq!(route, Route::Home {});

    // Verify that "/login" matches Login
    let route = Route::from_str("/login").unwrap();
    assert_eq!(route, Route::Login {});

    // Verify that "/panel" matches Dashboard
    let route = Route::from_str("/panel").unwrap();
    assert_eq!(route, Route::Dashboard {});
}

#[test]
fn test_route_to_string() {
    assert_eq!(Route::Home {}.to_string(), "/");
    assert_eq!(Route::Login {}.to_string(), "/login");
    assert_eq!(Route::Dashboard {}.to_string(), "/panel");
}
