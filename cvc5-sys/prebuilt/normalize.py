#!/usr/bin/env python3
"""Normalize bindgen output so `prebuilt/*.rs` is platform-neutral.

`prebuilt/bindings.rs` and `prebuilt/parser_bindings.rs` are consumed on
docs.rs, which builds on Linux. Regenerating them on macOS is fine as long as
the two known platform artifacts are removed first; this script does that and
is idempotent.

Usage:
    python3 cvc5-sys/prebuilt/normalize.py cvc5-sys/prebuilt/*.rs

1. Symbol mangling. Mach-O prefixes C symbols with an underscore, so bindgen
   emits `#[link_name = "\\u{1}_cvc5_foo"]` on macOS and `"\\u{1}cvc5_foo"` on
   ELF. The leading underscore is stripped.

2. `char32_t` alias chain. cvc5's `include/cvc5/c/cvc5.h` defines `char32_t`
   behind `#ifdef __has_include(<uchar.h>)`. macOS misses that header and falls
   back to `typedef uint_least32_t char32_t`, while glibc supplies it via
   `__uint_least32_t`. Both resolve to a 32-bit unsigned integer; the glibc
   spelling is emitted so Linux regeneration is a no-op diff.
"""

import re
import sys

MAC_ALIASES = "pub type uint_least32_t = u32;\n"
LINUX_ALIASES = (
    "pub type __uint32_t = ::std::os::raw::c_uint;\n"
    "pub type __uint_least32_t = __uint32_t;\n"
)


def normalize(text: str) -> tuple[str, dict[str, int]]:
    stats = {}

    text, n = re.subn(r'(link_name = "\\u\{1\})_', r"\1", text)
    stats["symbols_unmangled"] = n

    if MAC_ALIASES in text:
        text = text.replace(MAC_ALIASES, LINUX_ALIASES, 1)
        stats["char32_alias_rewritten"] = 1
    text, n = re.subn(
        r"^pub type char32_t = uint_least32_t;$",
        "pub type char32_t = __uint_least32_t;",
        text,
        flags=re.M,
    )
    stats["char32_alias_rewritten"] = stats.get("char32_alias_rewritten", 0) + n

    return text, stats


def main(paths: list[str]) -> int:
    if not paths:
        print(__doc__)
        return 2
    for path in paths:
        with open(path) as f:
            original = f.read()
        text, stats = normalize(original)
        if text != original:
            with open(path, "w") as f:
                f.write(text)
        touched = {k: v for k, v in stats.items() if v}
        print(f"{path}: {touched or 'already normalized'}")
        leftover = re.findall(r'link_name = "\\u\{1\}_', text)
        if leftover:
            print(f"  ERROR: {len(leftover)} mangled symbols remain", file=sys.stderr)
            return 1
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
