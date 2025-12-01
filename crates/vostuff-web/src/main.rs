#[cfg(feature = "ssr")]
#[tokio::main]
async fn main() {
    // SSR server will be implemented here
    println!("VOStuff Web Server - Coming soon");
}

#[cfg(not(feature = "ssr"))]
fn main() {
    // This binary requires the ssr feature
}
