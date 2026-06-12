#[inline]
pub fn is_mobile(ua: &str) -> bool {
    ua.contains("iPhone")
        || ua.contains("Android")
        || ua.contains("Mobile")
        || ua.contains("iPad")
        || ua.contains("Windows Phone")
}
