//! Minimal `multipart/form-data` reader for the attachment upload; avoids a dependency for one endpoint.

pub struct Part {
    pub name: String,
    pub filename: Option<String>,
    pub data: Vec<u8>,
}

fn find(hay: &[u8], needle: &[u8], from: usize) -> Option<usize> {
    hay.get(from..)?
        .windows(needle.len())
        .position(|w| w == needle)
        .map(|p| p + from)
}

pub fn boundary(content_type: &str) -> Option<String> {
    let (ty, rest) = content_type.split_once(';')?;
    if !ty.trim().eq_ignore_ascii_case("multipart/form-data") {
        return None;
    }
    let v = rest
        .split(';')
        .find_map(|p| p.trim().strip_prefix("boundary="))?;
    Some(v.trim_matches('"').to_string()).filter(|b| !b.is_empty())
}

fn param<'a>(header: &'a str, key: &str) -> Option<&'a str> {
    let at = header.find(&format!("{key}=\""))? + key.len() + 2;
    let end = header[at..].find('"')? + at;
    Some(&header[at..end])
}

pub fn parse(body: &[u8], boundary: &str) -> Option<Vec<Part>> {
    let delim = format!("--{boundary}").into_bytes();
    let mut parts = Vec::new();
    let mut pos = find(body, &delim, 0)? + delim.len();
    loop {
        if body.get(pos..pos + 2)? == b"--" {
            return Some(parts);
        }
        let head_start = pos + 2;
        let head_end = find(body, b"\r\n\r\n", head_start)?;
        let head = String::from_utf8_lossy(&body[head_start..head_end]).into_owned();
        let data_start = head_end + 4;
        let next = find(body, &[b"\r\n".as_slice(), &delim].concat(), data_start)?;
        let disposition = head
            .lines()
            .find(|l| l.to_ascii_lowercase().starts_with("content-disposition"))?;
        parts.push(Part {
            name: param(disposition, "name")?.to_string(),
            filename: param(disposition, "filename").map(String::from),
            data: body[data_start..next].to_vec(),
        });
        pos = next + 2 + delim.len();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_fields_and_files() {
        let body = b"--XX\r\nContent-Disposition: form-data; name=\"comment\"\r\n\r\nhi\r\n--XX\r\nContent-Disposition: form-data; name=\"file\"; filename=\"a b.txt\"\r\nContent-Type: text/plain\r\n\r\nline\r\n--XX\r\n\r\n--XX--\r\n";
        let parts = parse(body, "XX");
        assert!(parts.is_none(), "malformed trailing part is rejected");
        let ok = &body[..body.len() - "--XX\r\n\r\n--XX--\r\n".len()];
        let parts = parse(&[ok, b"--XX--\r\n"].concat(), "XX").unwrap();
        assert_eq!(parts.len(), 2);
        assert_eq!(
            (parts[0].name.as_str(), parts[0].data.as_slice()),
            ("comment", b"hi".as_slice())
        );
        assert_eq!(parts[1].filename.as_deref(), Some("a b.txt"));
        assert_eq!(parts[1].data, b"line");
        assert_eq!(
            boundary("multipart/form-data; boundary=\"XX\"").as_deref(),
            Some("XX")
        );
        assert_eq!(boundary("application/json"), None);
    }
}
