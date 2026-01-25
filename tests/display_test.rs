use std::time::UNIX_EPOCH;

use treer::cmd::display::{format_date, format_file_size, format_permissions};

#[test]
fn test_format_permissions() {
    let file_644 = format_permissions(0o644, false);
    let file_755 = format_permissions(0o755, false);
    let file_777 = format_permissions(0o777, false);
    let folder_644 = format_permissions(0o644, true);
    let folder_755 = format_permissions(0o755, true);
    let folder_777 = format_permissions(0o777, true);
    assert_eq!(file_644, String::from("[-rw-r--r--]"));
    assert_eq!(file_755, String::from("[-rwxr-xr-x]"));
    assert_eq!(file_777, String::from("[-rwxrwxrwx]"));
    assert_eq!(folder_644, String::from("[drw-r--r--]"));
    assert_eq!(folder_755, String::from("[drwxr-xr-x]"));
    assert_eq!(folder_777, String::from("[drwxrwxrwx]"));
}

#[test]
fn test_format_file_size() {
    assert_eq!(format_file_size(0), String::from("0 B"));
    assert_eq!(format_file_size(999), String::from("999 B"));
    assert_eq!(format_file_size(2048), String::from("2.0 KB"));
    assert_eq!(format_file_size(2560), String::from("2.5 KB"));
    assert_eq!(format_file_size(2690), String::from("2.6 KB"));
    assert_eq!(format_file_size(1048576), String::from("1.0 MB"));
    assert_eq!(format_file_size(3365930), String::from("3.2 MB"));
}

#[test]
fn test_format_date() {
    let date_1 = UNIX_EPOCH + std::time::Duration::from_secs(69696969);
    let date_2 = UNIX_EPOCH + std::time::Duration::from_secs(96969696);
    let date_3 = UNIX_EPOCH + std::time::Duration::from_secs(99999999);
    let date_4 = UNIX_EPOCH + std::time::Duration::from_secs(9999966666);
    let date_5 = UNIX_EPOCH + std::time::Duration::from_secs(6666699999);
    assert_eq!(format_date(date_1), "1972-03-17 16:16:09");
    assert_eq!(format_date(date_2), "1973-01-27 08:01:36");
    assert_eq!(format_date(date_3), "1973-03-03 09:46:39");
    assert_eq!(format_date(date_4), "1986-11-18 08:31:06");
    assert_eq!(format_date(date_5), "1981-04-03 21:06:39");
}
