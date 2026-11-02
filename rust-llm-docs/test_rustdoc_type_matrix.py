"""Tests for rustdoc_type_matrix.py over a small hand-made rustdoc JSON crate."""

import json
import sys
import tempfile
import unittest
from html.parser import HTMLParser
from pathlib import Path

HERE = Path(__file__).parent
sys.path.insert(0, str(HERE))

import rustdoc_type_matrix as tm  # noqa: E402
from rustdoc_type_matrix_html import render_html  # noqa: E402

NO_GENERICS = {"params": [], "where_predicates": []}
META = {"commit": "0123456789abcdef", "generated_at": "2026-01-01T00:00:00+00:00", "dirty": False}


def cfg_feature(name, file="src/lib.rs", line=1):
    return (
        f'#[attr = CfgTrace([NameValue {{ name: "feature", value: Some("{name}"), '
        f"span: {file}:{line}:7: {line}:29 (#0) }}])]"
    )


def generics(**bounds):
    """Type parameters with trait bounds given as {param: [(trait id, trait name), ...]}."""
    return {
        "params": [
            {
                "name": name,
                "kind": {
                    "type": {
                        "bounds": [
                            {"trait_bound": {"trait": {"path": tname, "id": tid, "args": None}, "generic_params": [], "modifier": "none"}}
                            for tid, tname in tids
                        ],
                        "default": None,
                        "is_synthetic": False,
                    }
                },
            }
            for name, tids in bounds.items()
        ],
        "where_predicates": [],
    }


def path(name, item_id, *args):
    ab = {"angle_bracketed": {"args": [{"type": a} for a in args], "constraints": []}} if args else None
    return {"resolved_path": {"path": name, "id": item_id, "args": ab}}


def prim(name):
    return {"primitive": name}


def gen(name):
    return {"generic": name}


def ref(inner):
    return {"borrowed_ref": {"lifetime": None, "is_mutable": False, "type": inner}}


class FixtureCrate:
    """Builds a rustdoc JSON document for a crate named ``miniextendr_api``."""

    def __init__(self):
        self.index = {}
        self.paths = {}
        self.next_id = 1
        self.external = {"1": {"name": "alloc", "html_root_url": None, "path": "x"},
                         "2": {"name": "jiff", "html_root_url": None, "path": "y"}}
        self.root = self.module("miniextendr_api", None, span=("src/lib.rs", 1, 200))

    def _id(self):
        self.next_id += 1
        return self.next_id

    def _add(self, item, parent, segs):
        self.index[str(item["id"])] = item
        if parent is not None:
            self.index[str(parent)]["inner"]["module"]["items"].append(item["id"])
        if segs is not None:
            kind = tm.kind_of(item)
            self.paths[str(item["id"])] = {"crate_id": 0, "path": ["miniextendr_api", *segs], "kind": kind}
        return item["id"]

    def item(self, name, inner, parent, segs, vis="public", docs=None, attrs=(), span=("src/lib.rs", 1, 1)):
        item = {
            "id": self._id(), "crate_id": 0, "name": name, "docs": docs, "links": {},
            "visibility": vis, "attrs": [{"other": a} for a in attrs],
            "span": {"filename": span[0], "begin": [span[1], 1], "end": [span[2], 1]},
            "inner": inner, "deprecation": None,
        }
        return self._add(item, parent, segs)

    def module(self, name, parent, vis="public", segs=(), attrs=(), span=("src/lib.rs", 1, 1)):
        inner = {"module": {"is_crate": parent is None, "items": [], "is_stripped": False}}
        return self.item(name, inner, parent, list(segs) if parent is not None else None, vis, attrs=attrs, span=span)

    def struct(self, name, parent, segs, generics_=NO_GENERICS, vis="public", docs=None, span=("src/lib.rs", 10, 12)):
        inner = {"struct": {"kind": "unit", "generics": generics_, "impls": []}}
        return self.item(name, inner, parent, segs, vis, docs, span=span)

    def trait(self, name, parent, segs):
        inner = {"trait": {"is_auto": False, "is_unsafe": False, "is_dyn_compatible": True, "items": [],
                           "generics": NO_GENERICS, "bounds": [], "implementations": []}}
        return self.item(name, inner, parent, segs, docs=f"The {name} trait.")

    def use(self, name, target, parent, glob=False):
        inner = {"use": {"source": name, "name": name, "id": target, "is_glob": glob}}
        return self.item(name, inner, parent, None)

    def external_path(self, item_id, crate_id, segs, kind="struct"):
        self.paths[str(item_id)] = {"crate_id": crate_id, "path": segs, "kind": kind}

    def impl(self, trait_id, trait_name, for_type, generics_=NO_GENERICS, span=("src/conv.rs", 5, 9),
             of=None, blanket=None):
        inner = {"impl": {
            "is_unsafe": False, "generics": generics_, "provided_trait_methods": [],
            "trait": {"path": trait_name, "id": trait_id, "args": None},
            "for": for_type, "items": [], "is_negative": False, "is_synthetic": False,
            "blanket_impl": blanket,
        }}
        impl_id = self._id()
        self.index[str(impl_id)] = {
            "id": impl_id, "crate_id": 0, "name": None, "docs": None, "links": {}, "visibility": "default",
            "attrs": [], "span": {"filename": span[0], "begin": [span[1], 1], "end": [span[2], 1]},
            "inner": inner, "deprecation": None,
        }
        if of is not None:
            self.index[str(of)]["inner"]["struct"]["impls"].append(impl_id)
        return impl_id

    def document(self):
        return {"root": self.root, "crate_version": "0.1.0", "includes_private": True, "index": self.index,
                "paths": self.paths, "external_crates": self.external, "format_version": 61}


def build_fixture(tmp: Path) -> Path:
    c = FixtureCrate()
    root = c.root
    from_r = c.module("from_r", root, segs=["from_r"])
    into_r = c.module("into_r", root, segs=["into_r"])
    newtype = c.module("newtype", root, segs=["newtype"])
    foo_mod = c.module("foo", root, segs=["foo"])
    hidden = c.module("hidden", root, vis="crate", segs=["hidden"])
    prelude = c.module("prelude", root, segs=["prelude"])
    c.module(
        "jiff_impl", root, segs=["jiff_impl"], attrs=[cfg_feature("jiff")], span=("src/jiff_impl.rs", 1, 100)
    )

    try_from = c.trait("TryFromSexp", from_r, ["from_r", "TryFromSexp"])
    into = c.trait("IntoR", into_r, ["into_r", "IntoR"])
    vec_elem = c.trait("IntoRVecElement", newtype, ["newtype", "IntoRVecElement"])

    foo = c.struct("Foo", foo_mod, ["foo", "Foo"], generics(T=[]),
                   docs="A foo.\n\n```rust\nlet f = Foo::new();\n```")
    qux = c.struct("Qux", foo_mod, ["foo", "Qux"], generics(T=[]), docs="Only some instances convert.")
    bar = c.struct("Bar", foo_mod, ["foo", "Bar"])
    c.struct("LazyDots", foo_mod, ["foo", "LazyDots"], docs="Marker.")
    baz = c.struct("Baz", hidden, ["hidden", "Baz"], docs="Re-exported from a private module.")
    c.struct("Secret", hidden, ["hidden", "Secret"])
    c.use("Baz", baz, root)
    c.use("foo", foo_mod, prelude, glob=True)

    vec_id, timestamp_id, string_id = 9001, 9002, 9003
    c.external_path(vec_id, 1, ["alloc", "vec", "Vec"])
    c.external_path(timestamp_id, 2, ["jiff", "timestamp", "Timestamp"])
    c.external_path(string_id, 1, ["alloc", "string", "String"])

    c.impl(try_from, "TryFromSexp", path("Foo", foo, gen("T")), generics(T=[(try_from, "TryFromSexp")]), of=foo)
    c.impl(try_from, "TryFromSexp", path("Qux", qux, prim("i32")), of=qux)
    c.impl(try_from, "TryFromSexp", ref(path("Baz", baz)), of=baz)
    c.impl(into, "IntoR", path("Bar", bar), of=bar)
    c.impl(vec_elem, "IntoRVecElement", path("Bar", bar), of=bar)
    c.impl(into, "IntoR", path("Vec", vec_id, prim("i32")), span=("src/conv.rs", 20, 22))
    c.impl(into, "IntoR", path("Vec", vec_id, gen("T")), generics(T=[(vec_elem, "IntoRVecElement")]), span=("src/conv.rs", 30, 32))
    c.impl(try_from, "TryFromSexp", path("Vec", vec_id, prim("f64")), span=("src/conv.rs", 40, 42))
    c.impl(try_from, "TryFromSexp", path("Timestamp", timestamp_id), span=("src/jiff_impl.rs", 10, 20))
    c.impl(into, "IntoR", path("String", string_id), span=("src/conv.rs", 50, 52))

    api_json = tmp / "miniextendr_api.json"
    api_json.write_text(json.dumps(c.document()))

    files = {
        "miniextendr-macros/src/lib.rs": (
            "// \"Bar\" in a comment is not a match\n"
            "const NAMES: &[&str] = &[\"Qux\"];\n"
            "fn is_foo(seg: &str) -> bool {\n    seg == \"Foo\"\n}\n"
        ),
        "miniextendr-macros/src/tests.rs": "fn t() { let _ = \"Baz\"; }\n",
        "docs/GUIDE.md": (
            "Use `Foo<T>` here. Bar is only prose.\n\n"
            "```rust\nlet v: Vec<i32> = vec![];\n```\n"
        ),
        "rpkg/src/rust/lib.rs": (
            "// Qux in a comment does not count\n"
            "#[miniextendr]\npub fn take_foo(x: Foo<i32>) -> Vec<f64> {\n    todo!()\n}\n"
            "#[miniextendr]\npub fn untested_foo(x: Foo<u8>) {}\n"
        ),
        "rpkg/tests/testthat/test-foo.R": "test_that('x', expect_equal(take_foo(1), 2))\n",
        "miniextendr-api/tests/bar.rs": "fn t() { let b: Bar = make(); }\n",
        "miniextendr-api/Cargo.toml": (
            "[package]\nname = \"miniextendr-api\"\n\n"
            "[dependencies]\njiff = { version = \"0.2\", optional = true }\n\n"
            "[features]\njiff = [\"dep:jiff\"]\n"
        ),
    }
    for rel, text in files.items():
        p = tmp / "repo" / rel
        p.parent.mkdir(parents=True, exist_ok=True)
        p.write_text(text)
    return api_json


class FixtureMatrixTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.tmp = tempfile.TemporaryDirectory()
        tmp = Path(cls.tmp.name)
        api_json = build_fixture(tmp)
        cls.data = tm.build(api_json, None, tmp / "repo", meta=META)
        cls.types = {r["name"]: r for r in cls.data["tabs"][0]["rows"]}
        cls.converted = {r["name"]: r for r in cls.data["tabs"][1]["rows"]}

    @classmethod
    def tearDownClass(cls):
        cls.tmp.cleanup()

    def status(self, row, col):
        return row["cells"][col]["status"], row["cells"][col]["label"]

    def test_rows_are_reachable_public_items_only(self):
        self.assertNotIn("Secret", self.types)
        self.assertEqual(
            sorted(self.types),
            ["Bar", "Baz", "Foo", "IntoR", "IntoRVecElement", "LazyDots", "Qux", "TryFromSexp"],
        )

    def test_reexport_dedup_and_canonical_path(self):
        foo = self.types["Foo"]
        self.assertEqual(foo["path"], "miniextendr_api::foo::Foo")
        self.assertEqual(foo["reexports"], ["miniextendr_api::prelude::Foo"])
        baz = self.types["Baz"]
        self.assertEqual(baz["path"], "miniextendr_api::Baz")
        self.assertEqual(baz["info"]["module"], "(root)")

    def test_arg_statuses(self):
        self.assertEqual(self.status(self.types["Foo"], "arg"), ("yes", "yes"))
        self.assertEqual(self.status(self.types["Qux"], "arg"), ("partial", "inst"))
        self.assertEqual(self.status(self.types["Baz"], "arg"), ("partial", "&T"))
        self.assertEqual(self.status(self.types["LazyDots"], "arg"), ("yes", "macro"))
        self.assertEqual(self.status(self.types["Bar"], "arg"), ("no", "no"))
        self.assertEqual(self.status(self.types["TryFromSexp"], "arg"), ("yes", "is"))

    def test_bound_on_own_parameter_is_in_evidence(self):
        text = self.types["Foo"]["cells"]["arg"]["evidence"][0]["text"]
        self.assertIn("T: TryFromSexp", text)

    def test_optvec_from_element_trait_bound(self):
        self.assertEqual(self.status(self.types["Bar"], "optvec"), ("partial", "vec"))
        self.assertEqual(self.status(self.types["IntoRVecElement"], "ret"), ("partial", "bound"))

    def test_macro_column_matches_whole_literals_outside_tests(self):
        foo = self.types["Foo"]["cells"]["macro"]
        self.assertEqual(foo["status"], "yes")
        self.assertEqual(foo["evidence"][0]["text"], '"Foo" in fn is_foo')
        self.assertEqual(foo["evidence"][0]["path"], "miniextendr-macros/src/lib.rs")
        self.assertEqual(foo["evidence"][0]["line"], 4)
        self.assertEqual(self.types["Qux"]["cells"]["macro"]["evidence"][0]["text"], '"Qux" in const NAMES')
        self.assertEqual(self.status(self.types["Bar"], "macro"), ("no", "no"))
        self.assertEqual(self.status(self.types["Baz"], "macro"), ("no", "no"))

    def test_quality_columns(self):
        foo, bar, qux = self.types["Foo"], self.types["Bar"], self.types["Qux"]
        self.assertEqual(self.status(foo, "rustdoc")[0], "yes")
        self.assertEqual(self.status(qux, "rustdoc"), ("partial", "docs"))
        self.assertEqual(self.status(bar, "rustdoc"), ("gap", "none"))
        self.assertEqual(self.status(foo, "docs")[0], "yes")
        self.assertEqual(self.status(bar, "docs")[0], "gap")  # prose only
        self.assertEqual(self.status(qux, "rpkg")[0], "gap")  # comment only
        self.assertEqual(self.status(foo, "rtests"), ("yes", "1/2"))
        self.assertEqual(self.status(bar, "rtests")[0], "n/a")
        self.assertEqual(self.status(bar, "rusttests")[0], "yes")
        self.assertEqual(self.status(foo, "rusttests")[0], "gap")
        self.assertEqual(self.status(foo, "crosspkg")[0], "n/a")

    def test_converted_rows_normalize_shapes(self):
        self.assertEqual(sorted(self.converted), ["String", "Vec<T>", "jiff::Timestamp"])
        vec = self.converted["Vec<T>"]
        self.assertEqual(self.status(vec, "ret"), ("yes", "yes"))
        self.assertEqual(self.status(vec, "arg"), ("partial", "inst"))
        self.assertEqual(vec["info"]["bounds"], "T: IntoRVecElement")
        self.assertIn("Vec<T>", vec["cells"]["ret"]["evidence"][1]["text"])  # the covering impl first
        self.assertEqual(self.status(vec, "docs")[0], "yes")
        self.assertEqual(self.status(vec, "rpkg")[0], "yes")
        self.assertEqual(self.status(vec, "rtests"), ("yes", "1/1"))

    def test_feature_from_module_cfg(self):
        ts = self.converted["jiff::Timestamp"]
        self.assertEqual(ts["info"]["feature"], "jiff")
        self.assertEqual(ts["info"]["origin"], "jiff")
        self.assertEqual(self.converted["String"]["info"]["feature"], "always")
        self.assertEqual(self.types["Foo"]["info"]["feature"], "always")

    def test_summary_counts(self):
        summary = self.data["tabs"][0]["summary"]["arg"]
        self.assertEqual(sum(summary.values()), len(self.types))

    def test_output_is_deterministic(self):
        again = tm.build(Path(self.tmp.name) / "miniextendr_api.json", None, Path(self.tmp.name) / "repo", meta=META)
        self.assertEqual(json.dumps(again, sort_keys=True), json.dumps(self.data, sort_keys=True))


class CfgTraceTests(unittest.TestCase):
    def test_feature(self):
        preds = tm.parse_cfg_trace(cfg_feature("serde_json"))
        self.assertEqual(preds, [("nv", "feature", "serde_json")])
        self.assertEqual(tm.feature_text(preds), "serde_json")

    def test_not_and_non_feature(self):
        text = (
            '#[attr = CfgTrace([Not(NameValue { name: "target_arch", value: Some("wasm32"), '
            "span: a.rs:1:11: 1:33 (#0) }, a.rs:1:10: 1:34 (#0))])]"
        )
        preds = tm.parse_cfg_trace(text)
        self.assertEqual(preds, [("not", ("nv", "target_arch", "wasm32"))])
        self.assertEqual(tm.feature_text(preds), 'cfg(not(target_arch = "wasm32"))')
        self.assertEqual(tm.cfg_features(preds[0]), set())

    def test_all_any_and_valueless(self):
        text = (
            '#[attr = CfgTrace([All([NameValue { name: "feature", value: Some("a"), span: l.rs:1:1: 1:2 (#0) }, '
            'NameValue { name: "feature", value: Some("b"), span: l.rs:1:3: 1:4 (#0) }], l.rs:1:0: 1:5 (#0))])]'
        )
        self.assertEqual(tm.feature_text(tm.parse_cfg_trace(text)), "a & b")
        text = text.replace("All(", "Any(")
        self.assertEqual(tm.feature_text(tm.parse_cfg_trace(text)), "(a | b)")
        text = '#[attr = CfgTrace([NameValue { name: "r_4_6", value: None, span: c.rs:1:1: 1:2 (#0) }])]'
        self.assertEqual(tm.feature_text(tm.parse_cfg_trace(text)), "cfg(r_4_6)")

    def test_other_attributes_are_ignored(self):
        self.assertIsNone(tm.parse_cfg_trace("#[attr = Repr(Transparent)]"))
        self.assertEqual(tm.item_cfgs({"attrs": [{"other": "#[attr = CfgAttrTrace]"}]}), [])


class ShapeTests(unittest.TestCase):
    def shape_of(self, text):
        toks = tm.tokenize_rust(text)
        ir, end = tm.parse_type(toks, 0)
        self.assertEqual(end, len(toks), text)
        return tm.shape(ir)

    def test_container_leaves_fold_to_placeholders(self):
        self.assertEqual(self.shape_of("Vec<i32>"), "Vec<T>")
        self.assertEqual(self.shape_of("Vec<String>"), "Vec<T>")
        self.assertEqual(self.shape_of("Vec<Option<f64>>"), "Vec<Option<T>>")
        self.assertEqual(self.shape_of("std::collections::HashMap<String, i32>"), "HashMap<String, V>")
        self.assertEqual(self.shape_of("Result<Vec<u8>, MyError>"), "Result<Vec<T>, E>")
        self.assertEqual(self.shape_of("Vec<&'a str>"), "Vec<&str>")
        self.assertEqual(self.shape_of("(i32, String, f64)"), "(T1, …, Tn)")

    def test_references_keep_named_types(self):
        self.assertEqual(self.shape_of("&'a [i32]"), "&[T]")
        self.assertEqual(self.shape_of("&mut f64"), "&mut T")
        self.assertEqual(self.shape_of("&Path"), "&Path")
        self.assertEqual(self.shape_of("&str"), "&str")
        self.assertEqual(self.shape_of("[u8; 4]"), "[T; N]")

    def test_tokenizer_drops_comments_and_keeps_strings(self):
        toks = tm.tokenize_rust('/* a /* nested */ "x" */ let s = r#"raw "q""#; // "c"\nlet c = \'"\';')
        self.assertEqual([t[1] for t in toks if t[0] == "str"], ['raw "q"'])
        self.assertEqual(toks[-1][2], 2)
        self.assertNotIn("nested", [t[1] for t in toks])

    def test_markdown_code_spans_and_fences(self):
        md = "Prose Foo.\n`Bar` and ``Baz``\n```rust\nlet q: Qux = x;\n```\n"
        self.assertEqual(
            tm.markdown_code(md),
            [("Bar", 2, True), ("Baz", 2, True), ("let q: Qux = x;", 4, False)],
        )


class EvidenceTests(unittest.TestCase):
    def test_cap_and_count(self):
        evidence = [tm.ev(f"hit {i:02}", "a.rs", i) for i in range(1, tm.EVIDENCE_CAP + 6)]
        c = tm.cell("yes", "x", evidence)
        self.assertEqual(len(c["evidence"]), tm.EVIDENCE_CAP)
        self.assertEqual(c["more"], 5)

    def test_duplicates_collapse(self):
        c = tm.cell("yes", "x", [tm.ev("a", "f.rs", 1), tm.ev("a", "f.rs", 1)])
        self.assertEqual(len(c["evidence"]), 1)
        self.assertNotIn("more", c)


class _Collect(HTMLParser):
    def __init__(self):
        super().__init__()
        self.scripts = []
        self.external = []
        self._cur = None
        self.title = ""
        self._in_title = False

    def handle_starttag(self, tag, attrs):
        attrs = dict(attrs)
        if tag == "script":
            if "src" in attrs:
                self.external.append(attrs["src"])
            self._cur = [attrs, ""]
        elif tag == "link":
            self.external.append(attrs.get("href"))
        elif tag == "title":
            self._in_title = True

    def handle_endtag(self, tag):
        if tag == "script":
            self.scripts.append(self._cur)
            self._cur = None
        elif tag == "title":
            self._in_title = False

    def handle_data(self, data):
        if self._cur is not None:
            self._cur[1] += data
        if self._in_title:
            self.title += data


class HtmlSmokeTests(unittest.TestCase):
    def test_page_embeds_the_data_and_nothing_external(self):
        with tempfile.TemporaryDirectory() as d:
            data = tm.build(build_fixture(Path(d)), None, Path(d) / "repo", meta=META)
        data["tabs"][0]["rows"][0]["cells"]["arg"]["evidence"].append({"text": "</script><b>x</b>"})
        page = render_html(data)
        parser = _Collect()
        parser.feed(page)
        parser.close()
        self.assertEqual(parser.title, "miniextendr type matrix")
        self.assertEqual(parser.external, [])
        embedded = [s for s in parser.scripts if s[0].get("type") == "application/json"]
        self.assertEqual(len(embedded), 1)
        self.assertEqual(json.loads(embedded[0][1]), data)
        self.assertIn(':root:not([data-theme="light"])', page)
        self.assertIn(':root[data-theme="dark"]', page)
        self.assertIn(data["commit"][:10], page)


if __name__ == "__main__":
    unittest.main()
