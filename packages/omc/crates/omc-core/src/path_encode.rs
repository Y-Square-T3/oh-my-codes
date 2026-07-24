pub fn encode_repo_path(path: &str) -> String {
    path.replace('$', "$24").replace('/', "$2F")
}

pub fn decode_repo_path(encoded: &str) -> String {
    encoded.replace("$2F", "/").replace("$24", "$")
}
