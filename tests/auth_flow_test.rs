use rr_ui::ui::state::{UIState, SyncStatus, CoreConnectivity};
use dioxus::prelude::*;

#[test]
fn test_ui_state_initialization() {
    let mut vdom = VirtualDom::new(|| {
        let state = UIState::new();
        
        assert_eq!(*state.is_authenticated.read(), false);
        assert_eq!(*state.status.read(), SyncStatus::Initial);
        assert_eq!(*state.core_status.read(), CoreConnectivity::CoreOffline);
        
        rsx! { div {} }
    });
    let _ = vdom.rebuild_in_place();
}

#[test]
fn test_auth_transition() {
    let mut vdom = VirtualDom::new(|| {
        let mut state = UIState::new();
        
        // Simulate login success
        state.is_authenticated.set(true);
        state.auth_token.set(Some("test-token".to_string()));
        
        assert_eq!(*state.is_authenticated.read(), true);
        assert!(state.auth_token.read().is_some());
        
        rsx! { div {} }
    });
    let _ = vdom.rebuild_in_place();
}

#[test]
fn test_sync_status_transition() {
    let mut vdom = VirtualDom::new(|| {
        let mut state = UIState::new();
        
        state.status.set(SyncStatus::Live);
        assert_eq!(*state.status.read(), SyncStatus::Live);
        
        rsx! { div {} }
    });
    let _ = vdom.rebuild_in_place();
}
