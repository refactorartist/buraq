pub struct Pagination {
    pub page: Option<u32>,
    pub limit: Option<u32>,
}

impl Pagination {
    pub fn skip(&self) -> u64 {
        if self.page.is_none() || self.limit.is_none() {
            0
        } else {
            ((self.page.unwrap() - 1) * self.limit.unwrap()) as u64
        }
    }

    pub fn limit(&self) -> i64 {
        if self.limit.is_none() {
            10
        } else {
            self.limit.unwrap() as i64
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pagination_creation() {
        let pagination = Pagination { page: Some(1), limit: Some(10) };
        assert_eq!(pagination.page, Some(1));
        assert_eq!(pagination.limit, Some(10));
    }

    #[test]
    fn test_pagination_with_zero_values() {
        let pagination = Pagination { page: Some(0), limit: Some(0) };
        assert_eq!(pagination.page, Some(0));
        assert_eq!(pagination.limit, Some(0));
    }

    #[test]
    fn test_pagination_with_large_values() {
        let pagination = Pagination {
            page: Some(999999),
            limit: Some(999999),
        };
        assert_eq!(pagination.page, Some(999999));
        assert_eq!(pagination.limit, Some(999999));
    }
}
