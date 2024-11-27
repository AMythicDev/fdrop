/// Goes through the chain of errors in `start` and formats them into a human-readable report
pub fn human_readable_error(start: &dyn std::error::Error) -> String {
    let mut s = String::with_capacity(256);

    let mut err: &dyn std::error::Error = &start;
    s.push_str(err.to_string().as_ref());
    s.push_str("\n\nCaused by:\n");

    let i = 1;
    while let Some(e) = err.source() {
        s.push_str(&format!("{i}.\t{}", e.to_string()));
        err = e;
    }
    s
}
