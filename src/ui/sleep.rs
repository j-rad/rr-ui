pub async fn sleep(millis: u64) {
    #[cfg(target_arch = "wasm32")]
    crate::ui::sleep::sleep(millis as u32 as u64).await;
    #[cfg(not(target_arch = "wasm32"))]
    tokio::time::sleep(std::time::Duration::from_millis(millis)).await;
}
