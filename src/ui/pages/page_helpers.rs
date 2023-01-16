use ellipse::Ellipse;
pub fn get_column_string(s: &str, width: usize) -> String {
    if s.len() > width {
        if width == 0 {
            return "".to_string();
        } else if width == 1 {
            return ".".to_string();
        } else if width == 2 {
            return "..".to_string();
        } else if width == 3 {
            return "...".to_string();
        }
        return s.truncate_ellipse(width - 3).to_string();
    } else if s.len() < width {
        let mut spaces = "".to_owned();
        for _ in 0..(width - s.len()) {
            spaces.push(' ');
        }
        return s.to_string() + &spaces;
    }
    s.to_string()
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_column_string() {
        let text1 = "";
        let text2 = "test";
        let text3 = "testme";
        let text4 = "testmetest";

        let width = 0;

        assert_eq!(get_column_string(text4, width), "".to_owned());

        let width = 1;

        assert_eq!(get_column_string(text4, width), ".".to_owned());

        let width = 2;

        assert_eq!(get_column_string(text4, width), "..".to_owned());

        let width = 3;

        assert_eq!(get_column_string(text4, width), "...".to_owned());

        let width = 4;

        assert_eq!(get_column_string(text4, width), "t...".to_owned());

        let width = 6;

        assert_eq!(get_column_string(text1, width), "      ".to_owned());
        assert_eq!(get_column_string(text2, width), "test  ".to_owned());
        assert_eq!(get_column_string(text3, width), "testme".to_owned());
        assert_eq!(get_column_string(text4, width), "tes...".to_owned());
    }
}
