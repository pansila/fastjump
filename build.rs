
fn main() {
    // to clear up a space for the new output of build log
    // sometimes it's hard to see where the build log is started if
    // there are too many errors
    // it's not ideal to do it this way though, need a better one
    if let Ok(_) = std::env::var("RUST_BUILDLOG_CLEAR_SPACE") {
        (1..15).for_each(|_| {println!("cargo:warning=");})
    }
}
