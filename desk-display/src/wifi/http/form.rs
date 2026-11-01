use heapless::Vec;

pub fn get_form_value<'a>(body: &[u8], key: &str, output: &'a mut Vec<u8, 128>) -> Option<&'a str> {
    output.clear();

    for pair in body.split(|&b| b == b'&') {
        let Some(eq) = pair.iter().position(|&b| b == b'=') else {
            continue;
        };

        if &pair[..eq] != key.as_bytes() {
            continue;
        }

        if !decode_form_component(&pair[eq + 1..], output) {
            return None;
        }

        return str::from_utf8(output).ok();
    }

    None
}

fn decode_form_component(input: &[u8], output: &mut Vec<u8, 128>) -> bool {
    let mut i = 0;

    while i < input.len() {
        let byte = match input[i] {
            b'+' => {
                i += 1;
                b' '
            }

            b'%' => {
                if i + 2 >= input.len() {
                    return false;
                }

                let Some(high) = hex_digit(input[i + 1]) else {
                    return false;
                };

                let Some(low) = hex_digit(input[i + 2]) else {
                    return false;
                };

                i += 3;
                (high << 4) | low
            }

            b => {
                i += 1;
                b
            }
        };

        if output.push(byte).is_err() {
            return false;
        }
    }

    true
}

fn hex_digit(b: u8) -> Option<u8> {
    match b {
        b'0'..=b'9' => Some(b - b'0'),
        b'a'..=b'f' => Some(b - b'a' + 10),
        b'A'..=b'F' => Some(b - b'A' + 10),
        _ => None,
    }
}

pub fn is_urlencoded_form(headers: &str) -> bool {
    headers.lines().skip(1).any(|line| {
        let Some((name, value)) = line.split_once(':') else {
            return false;
        };

        name.eq_ignore_ascii_case("content-type")
            && value.trim().split(';').next().is_some_and(|value| {
                value.eq_ignore_ascii_case("application/x-www-form-urlencoded")
            })
    })
}
