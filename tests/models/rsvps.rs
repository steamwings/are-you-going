use are_you_going::models::rsvps::normalize_phone;

#[tokio::test]
async fn test_phone_normalization() {
    let result = normalize_phone("+14434631334");
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "+14434631334");
}

#[tokio::test]
async fn test_phone_normalization_no_plus() {
    let result = normalize_phone("4434631334");
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "+14434631334");
}

#[tokio::test]
async fn test_phone_normalization_with_formatting() {
    let result = normalize_phone("(443) 463-1334");
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "+14434631334");
}

#[tokio::test]
async fn test_phone_normalization_invalid() {
    let result = normalize_phone("abc");
    assert!(result.is_err());
}
