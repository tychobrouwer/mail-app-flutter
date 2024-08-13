pub enum FetchMode {
    ALL,
    ENVELOPE,
    BODY,
    UID,
    FLAGS,
}

pub fn string(fetch_mode: FetchMode) -> String {
    match fetch_mode {
        FetchMode::ALL => String::from("(UID FLAGS ENVELOPE BODY.PEEK[])"),
        FetchMode::ENVELOPE => String::from("ENVELOPE"),
        FetchMode::BODY => String::from("BODY"),
        FetchMode::UID => String::from("UID"),
        FetchMode::FLAGS => String::from("FLAGS"),
    }
}
