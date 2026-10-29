use super::*;

#[test]
fn test_data_frame_error_display() {
    let err = DataFrameError::NotDataFrame;
    assert_eq!(err.to_string(), "object does not inherit from data.frame");

    let err = DataFrameError::NoNames;
    assert_eq!(err.to_string(), "data.frame has no column names");

    let err = DataFrameError::UnequalLengths {
        expected: 3,
        columns: vec![("y".to_string(), 5)],
    };
    assert_eq!(err.to_string(), "column \"y\" has length 5 (expected 3)");

    let err = DataFrameError::UnnamedColumns;
    assert_eq!(
        err.to_string(),
        "cannot create data frame from unnamed list elements"
    );

    let err = DataFrameError::NoSuchColumn("g".to_string());
    assert_eq!(err.to_string(), "no such column: \"g\"");
}
