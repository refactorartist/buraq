use mongodb::bson::{Document, doc};
use serde::{Deserialize, Serialize};
use std::vec::Vec;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SortDirection {
    Ascending,
    Descending,
}

impl SortDirection {
    fn as_i32(&self) -> i32 {
        match self {
            SortDirection::Ascending => 1,
            SortDirection::Descending => -1,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Sort<T> {
    pub field: T,
    pub direction: SortDirection,
}

impl<T> Sort<T> {
    pub fn builder() -> SortBuilder<T> {
        SortBuilder::default()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SortBuilder<T> {
    sorts: Vec<Sort<T>>,
}

impl<T> Default for SortBuilder<T> {
    fn default() -> Self {
        Self { sorts: Vec::new() }
    }
}

impl<T> SortBuilder<T>
where
    T: Into<String> + Clone,
{
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_sort(mut self, field: T, direction: SortDirection) -> Self {
        self.sorts.push(Sort { field, direction });
        self
    }

    pub fn ascending(self, field: T) -> Self {
        self.add_sort(field, SortDirection::Ascending)
    }

    pub fn descending(self, field: T) -> Self {
        self.add_sort(field, SortDirection::Descending)
    }

    pub fn build(self) -> Vec<Sort<T>> {
        self.sorts.clone()
    }

    pub fn to_document(self) -> Document {
        let mut doc = Document::new();
        for sort in self.sorts {
            doc.insert(sort.field.into(), sort.direction.as_i32());
        }
        doc
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sort_direction_as_i32() {
        assert_eq!(SortDirection::Ascending.as_i32(), 1);
        assert_eq!(SortDirection::Descending.as_i32(), -1);
    }

    #[test]
    fn test_sort_builder_new() {
        let builder: SortBuilder<String> = SortBuilder::new();
        assert!(builder.sorts.is_empty());
    }

    #[test]
    fn test_sort_builder_add_sort() {
        let builder = SortBuilder::new()
            .add_sort("name".to_string(), SortDirection::Ascending)
            .add_sort("age".to_string(), SortDirection::Descending);

        let sorts = builder.build();
        assert_eq!(sorts.len(), 2);
        assert_eq!(sorts[0].field, "name");
        assert!(matches!(sorts[0].direction, SortDirection::Ascending));
        assert_eq!(sorts[1].field, "age");
        assert!(matches!(sorts[1].direction, SortDirection::Descending));
    }

    #[test]
    fn test_sort_builder_ascending_descending() {
        let builder = SortBuilder::new()
            .ascending("name".to_string())
            .descending("age".to_string());

        let sorts = builder.build();
        assert_eq!(sorts.len(), 2);
        assert_eq!(sorts[0].field, "name");
        assert!(matches!(sorts[0].direction, SortDirection::Ascending));
        assert_eq!(sorts[1].field, "age");
        assert!(matches!(sorts[1].direction, SortDirection::Descending));
    }

    #[test]
    fn test_sort_builder_to_document() {
        let doc = SortBuilder::new()
            .ascending("name".to_string())
            .descending("age".to_string())
            .to_document();

        assert_eq!(doc.get_i32("name").unwrap(), 1);
        assert_eq!(doc.get_i32("age").unwrap(), -1);
    }
}
