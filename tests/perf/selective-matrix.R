# Run in a fresh R process with the mx build under review installed.
# From this worktree, Rscript tests/perf/selective-matrix.R uses the rv library.
# This is a functional memory-budget regression, independent of machine speed.
# Set the JIT level before R compiles the function that attaches dimensions.
invisible(compiler::enableJIT(0L))
main <- function() {
    if (.Machine$sizeof.pointer < 8L) {
        message("This long-vector check requires 64-bit R.")
        return(invisible(NULL))
    }

    ns <- asNamespace("miniextendr")
    # A regression that forces the input must fail within a small R heap.
    old_limit <- mem.maxVSize()
    on.exit(mem.maxVSize(old_limit), add = TRUE)
    invisible(mem.maxVSize(min(old_limit, 512)))
    stopifnot(mem.maxVSize() <= 512)

    nr <- 2L
    nc <- 1500000000L
    end <- as.double(nr) * nc
    x <- seq_len(end)
    dim(x) <- c(nr, nc)

    stopifnot(identical(x[[end]], end))
    stopifnot(!ns$rarray_matrix_has_buffer(x))
    stopifnot(identical(ns$rarray_matrix_dims(x), c(nr, nc)))
    stopifnot(identical(ns$rarray_matrix_len(x), end))
    stopifnot(identical(ns$rarray_matrix_cell(x, nr, nc), end))
    stopifnot(identical(ns$rarray_matrix_column(x, 1L), c(1, 2)))
    stopifnot(identical(ns$rarray_matrix_column(x, nc), c(end - 1, end)))
    stopifnot(!ns$rarray_matrix_has_buffer(x))
    message("Read both ends of a 3-billion-element matrix under a ",
            mem.maxVSize(), " MiB R vector-heap cap.")

    if (capabilities("profmem")) {
        profile <- function(op, expected) {
            source <- structure(as.double(seq_len(4096L * 256L)),
                                dim = c(4096L, 256L))
            stopifnot(!ns$rarray_matrix_has_buffer(source))
            path <- tempfile("mx-selective-")
            Rprofmem(path, threshold = 16384L)
            out <- tryCatch(op(source), finally = Rprofmem(NULL))
            stopifnot(identical(out, expected), !ns$rarray_matrix_has_buffer(source))
            events <- grep("^[0-9]+ ", readLines(path), value = TRUE)
            sum(as.numeric(sub(" .*", "", events)))
        }
        dims_bytes <- profile(ns$rarray_matrix_dims, c(4096L, 256L))
        col_bytes <- profile(function(m) ns$rarray_matrix_column(m, 2L),
                             as.double(4096L + seq_len(4096L)))
        message("R allocations above 16 KiB on an 8 MiB compact matrix: ",
                dims_bytes, " B for dimensions, ", col_bytes, " B for one column.")
    }

    # A detached result can outlive the matrix without retaining its resource owner.
    state <- new.env(parent = emptyenv())
    state$released <- FALSE
    finalizer <- function(signal) {
        force(signal)
        function(key) signal$released <- TRUE
    }
    extract <- function() {
        owner <- new.env(parent = emptyenv())
        reg.finalizer(owner, finalizer(state))
        source <- matrix(c(1, 2, 3, 4), nrow = 2L)
        attr(source, "resource_owner") <- owner
        out <- ns$rarray_matrix_column(source, 2L)
        invisible(gc())
        stopifnot(!state$released, identical(attr(source, "resource_owner"), owner))
        out
    }
    out <- extract()
    invisible(gc())
    stopifnot(state$released, identical(out, c(3, 4)))
    message("The detached column remained valid after its source owner finalized.")
}
main()
