fn main() {
    tracing_subscriber::fmt().json().init();
    tracing::info!("misty media worker foundation ready");
}
