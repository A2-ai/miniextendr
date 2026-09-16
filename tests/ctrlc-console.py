#!/usr/bin/env python3
"""Exercise actual terminal Ctrl+C against an installed ctrlc-enabled package.

Run after installing rpkg with its detected features plus ctrlc, with R_LIBS
pointing at that installation. Requires a POSIX pseudo-terminal; no Python dependencies.
The child R has a controlling terminal. Writing its VINTR byte makes the kernel
send SIGINT to the foreground process group, just like typing Ctrl+C in a console.
"""

import errno
import os
import pty
import select
import signal
import sys
import time


def main():
    child, terminal = pty.fork()
    if child == 0:
        os.execvp("R", ["R", "--vanilla", "--quiet", "--no-save", "--no-readline"])

    buffered = b""

    def send(command):
        os.write(terminal, ("{ " + command + " }\n").encode())

    def expect(marker, timeout=30):
        nonlocal buffered
        deadline = time.monotonic() + timeout
        marker = marker.encode()
        while marker not in buffered:
            remaining = deadline - time.monotonic()
            if remaining <= 0:
                raise AssertionError(f"Timed out waiting for {marker!r}: {buffered!r}")
            if not select.select([terminal], [], [], remaining)[0]:
                continue
            try:
                chunk = os.read(terminal, 65536)
            except OSError as error:
                if error.errno != errno.EIO:
                    raise
                chunk = b""
            if not chunk:
                raise AssertionError(f"R exited waiting for {marker!r}: {buffered!r}")
            sys.stdout.buffer.write(chunk)
            sys.stdout.buffer.flush()
            buffered += chunk
        _, buffered = buffered.split(marker, 1)

    def ctrl_c():
        os.write(terminal, b"\x03")

    def done():
        expect("CTRLC_OK\r\n")

    try:
        expect("> ")
        send('library(miniextendr); stopifnot(!miniextendr:::ctrlc_handler_installed()); cat("CTRLC_OK\\n")')
        done()
        for function in ("ctrlc_wait", "ctrlc_wait_worker"):
            for _ in range(2):
                send('before <- miniextendr:::ctrlc_drop_count(); '
                     f'caught <- tryCatch(miniextendr:::{function}(30L), interrupt = identity); '
                     'stopifnot(identical(class(caught), c("rust_interrupt", "interrupt", "condition")), '
                     'identical(caught$kind, "interrupt"), '
                     'miniextendr:::ctrlc_drop_count() == before + 1L, '
                     'miniextendr:::ctrlc_wait(0L) == 42L); cat("CTRLC_OK\\n")')
                expect("CTRLC_READY\r\n")
                ctrl_c()
                done()
        # Without a handler, cancellation returns control to the actual prompt.
        send('before <- miniextendr:::ctrlc_drop_count(); miniextendr:::ctrlc_wait(30L)')
        expect("CTRLC_READY\r\n")
        ctrl_c()
        expect("> ")
        send('stopifnot(miniextendr:::ctrlc_drop_count() == before + 1L); cat("CTRLC_OK\\n")')
        done()
        # R's original handler must still interrupt ordinary R evaluation.
        send('caught <- tryCatch({cat("NATIVE_READY\\n"); Sys.sleep(30)}, interrupt = identity); '
             'stopifnot(inherits(caught, "interrupt"), !inherits(caught, "rust_interrupt")); cat("CTRLC_OK\\n")')
        expect("NATIVE_READY\r\n")
        ctrl_c()
        done()
        # Suspending interrupts must allow the owned Rust resource to complete.
        send('before <- miniextendr:::ctrlc_drop_count(); start <- proc.time()[[3L]]; '
             'caught <- tryCatch({suspendInterrupts(miniextendr:::ctrlc_wait(1L)); Sys.sleep(0.01)}, interrupt = identity); '
             'stopifnot(inherits(caught, "interrupt"), !inherits(caught, "rust_interrupt"), '
             'proc.time()[[3L]] - start >= 0.9, miniextendr:::ctrlc_drop_count() == before + 1L); cat("CTRLC_OK\\n")')
        expect("CTRLC_READY\r\n")
        ctrl_c()
        done()
        send('q("no", status = 0L)')
        # Drain output before waitpid: on macOS, a PTY child may stay in
        # exit until the master consumes the remaining terminal output.
        while True:
            if not select.select([terminal], [], [], 30)[0]:
                raise AssertionError("R did not close its console after q()")
            try:
                chunk = os.read(terminal, 65536)
            except OSError as error:
                if error.errno != errno.EIO:
                    raise
                break
            if not chunk:
                break
            sys.stdout.buffer.write(chunk)
            sys.stdout.buffer.flush()
        pid, status = os.waitpid(child, 0)
        assert pid == child and os.waitstatus_to_exitcode(status) == 0, status
        child = None
        print("PASS: main/worker cleanup, repeated interrupts, uncaught interrupt, native R, suspendInterrupts")
    finally:
        os.close(terminal)
        if child is not None:
            # Terminate only the test process we created, including on assertion failure.
            try:
                os.kill(child, signal.SIGTERM)
            except ProcessLookupError:
                pass
            os.waitpid(child, 0)


if __name__ == "__main__":
    main()
