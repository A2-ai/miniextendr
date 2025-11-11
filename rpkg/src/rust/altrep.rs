use miniextendr_api::miniextendr_module;

// Import-only module for ALTREP C-callables exposed by miniextendr-api.
miniextendr_module! {
    mod altrep;

    extern fn C_altrep_compact_int;
    extern fn C_altrep_from_doubles;
    extern fn C_altrep_from_strings;
    extern fn C_altrep_from_logicals;
    extern fn C_altrep_from_raw;
    extern fn C_altrep_from_list;
}

