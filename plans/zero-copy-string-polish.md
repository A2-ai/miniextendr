# Zero-Copy String Polish

Follow-up optimizations for the r-native-arrow string work. None are correctness issues — all are performance refinements.

## 1. Batch encoding check via CE tag

`charsxp_to_cow` calls `from_utf8` on every element. In a UTF-8 locale (which we assert at init), CE_UTF8 and CE_NATIVE strings are guaranteed valid UTF-8. We can skip validation entirely for those.

**Approach**: Check `Rf_getCharCE(charsxp)` first. If CE_UTF8 or CE_NATIVE (in our known-UTF-8 locale), use `from_utf8_unchecked` and return `Cow::Borrowed` directly. Only fall through to `from_utf8` → translate for CE_LATIN1/CE_BYTES.

**Why it matters**: `from_utf8` is O(n) in string length. For a 1M-element vector of 100-byte strings, that's 100MB of validation we know will pass. The CE tag check is O(1) per element.

**Risk**: `from_utf8_unchecked` is UB if the bytes aren't valid UTF-8. We trust R's CE tag + our locale assertion. If R ever lies about CE_UTF8 (it shouldn't), we'd get UB. The current `from_utf8` approach is a safety net — removing it trades defense-in-depth for speed.

## 2. CHARSXP cache cost during serialization

Serialization does N `Rf_mkCharLenCE` calls, each probing R's global CHARSXP hash table. For borrowed strings the cache hit is guaranteed, so it's zero-allocation, but each probe hashes the string content (O(string_length)) and does a table lookup.

**Approach**: For `ProtectedStrVec` (which wraps an existing STRSXP), serialization could return the STRSXP itself — `serialized_state()` just returns `self.as_sexp()`. No per-element work at all. R already knows how to serialize an STRSXP.

This doesn't apply to `Vec<Cow<str>>` (which isn't backed by a single SEXP), only to `ProtectedStrVec`. For Vec<Cow>, the current approach (N cache probes) is the best we can do without tracking the source SEXP.

**Alternative**: `Vec<Cow<str>>` could optionally carry a "source SEXP" field. If all elements were borrowed from one STRSXP, serialization returns that SEXP. But this adds a field to every Vec and complicates the type — probably not worth it.

## 3. Double protection in ProtectedStrVec

`ProtectedStrVec::try_from_sexp` adds `Rf_protect` even though `.Call` arguments are already protected by R. One extra protect stack slot, zero functional impact.

**Approach**: Add `ProtectedStrVec::from_sexp_trusted(sexp: SEXP) -> Self` that wraps without protecting. Use it in the generated `.Call` wrappers where we know R has already protected the argument. Keep the current `TryFromSexp` impl (which protects) as the safe default.

**Why punt**: The cost is one `i32` increment on R's protect stack. Literally unmeasurable. Only worth doing if we're also optimizing the generated wrapper codegen for other reasons.
