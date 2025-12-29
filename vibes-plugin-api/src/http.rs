//! HTTP types for plugin route registration

/// HTTP method for route registration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum HttpMethod {
    Get,
    Post,
    Put,
    Delete,
    Patch,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_http_method_equality() {
        assert_eq!(HttpMethod::Get, HttpMethod::Get);
        assert_ne!(HttpMethod::Get, HttpMethod::Post);
    }

    #[test]
    fn test_http_method_debug() {
        assert_eq!(format!("{:?}", HttpMethod::Get), "Get");
    }
}
