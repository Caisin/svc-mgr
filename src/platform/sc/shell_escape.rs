/// Windows command-line argument escaping for sc.exe.
pub fn escape_for_sc(arg: &str) -> String {
    if arg.is_empty() {
        return "\"\"".to_string();
    }
    if arg.contains(' ') || arg.contains('"') {
        let escaped = arg.replace('"', "\\\"");
        format!("\"{}\"", escaped)
    } else {
        arg.to_string()
    }
}

/// Build a binpath string for sc.exe from program + args.
pub fn build_binpath(program: &str, args: &[String]) -> String {
    let mut parts = vec![escape_for_sc(program)];
    for arg in args {
        parts.push(escape_for_sc(arg));
    }
    parts.join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn escape_empty_string() {
        assert_eq!(escape_for_sc(""), "\"\"");
    }

    #[test]
    fn escape_no_spaces() {
        assert_eq!(escape_for_sc("hello"), "hello");
    }

    #[test]
    fn escape_with_spaces() {
        assert_eq!(escape_for_sc("hello world"), "\"hello world\"");
    }

    #[test]
    fn build_binpath_simple() {
        let result = build_binpath("/usr/bin/app", &[]);
        assert_eq!(result, "/usr/bin/app");
    }

    #[test]
    fn build_binpath_with_args() {
        let args = vec!["--port".to_string(), "8080".to_string()];
        let result = build_binpath("/usr/bin/app", &args);
        assert_eq!(result, "/usr/bin/app --port 8080");
    }

    #[test]
    fn build_binpath_with_spaces_in_path() {
        let result = build_binpath("C:\\Program Files\\app.exe", &[]);
        assert_eq!(result, "\"C:\\Program Files\\app.exe\"");
    }
}
