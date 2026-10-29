use super::*;

/// Serial-path proof (#1055): the builder API compiles and is reachable
/// without `rayon`, registering native and character columns. This builds
/// the registration machinery without invoking R (no `build()` call, which
/// needs the R runtime), so it runs in the plain `cargo test` harness on
/// both feature settings — the primary guarantee is that this module
/// compiles and the surface is callable serially.
#[test]
fn builder_surface_is_callable_without_rayon() {
    let builder = RDataFrameBuilder::new(8)
        .column::<f64>("x", |chunk, offset| {
            for (i, slot) in chunk.iter_mut().enumerate() {
                *slot = (offset + i) as f64;
            }
        })
        .column_str("label", |i| Some(format!("row{i}")));
    assert_eq!(builder.nrow, 8);
    assert_eq!(builder.names, vec!["x".to_string(), "label".to_string()]);
    assert_eq!(builder.columns.len(), 2);
}
