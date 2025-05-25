pub struct Pagination {
    pub page: u32,
    pub limit: u32,
}

impl Pagination {
    pub fn skip(&self) -> u64 {
        ((self.page - 1) * self.limit) as u64
    }

    pub fn limit(&self) -> i64 {
        self.limit as i64
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pagination_creation() {
        let pagination = Pagination { page: 1, limit: 10 };
        assert_eq!(pagination.page, 1);
        assert_eq!(pagination.limit, 10);
    }

    #[test]
    fn test_pagination_with_zero_values() {
        let pagination = Pagination { page: 0, limit: 0 };
        assert_eq!(pagination.page, 0);
        assert_eq!(pagination.limit, 0);
    }

    #[test]
    fn test_pagination_with_large_values() {
        let pagination = Pagination {
            page: 999999,
            limit: 999999,
        };
        assert_eq!(pagination.page, 999999);
        assert_eq!(pagination.limit, 999999);
    }
}
