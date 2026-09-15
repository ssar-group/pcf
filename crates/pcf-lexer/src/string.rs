pub fn unescape_char(escape: char) -> Option<char> {
    Some(match escape {
        'n' => '\n',
        'r' => '\r',
        't' => '\t',
        '\\' => '\\',
        '"' => '"',
        '0' => '\0',
        _ => return None,
    })
}
