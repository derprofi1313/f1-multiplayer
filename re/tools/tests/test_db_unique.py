import collections
import json
import unittest
from pathlib import Path

DB = Path(__file__).resolve().parents[2] / "db"


class UniqueKeys(unittest.TestCase):
    def test_signature_ids_are_unique(self):
        sigs = json.loads((DB / "signatures.json").read_text())["signatures"]
        dup = sorted(k for k, n in collections.Counter(s["id"] for s in sigs).items() if n > 1)
        self.assertEqual(dup, [], f"{len(dup)} signature id(s) appear more than once, e.g. {dup[:5]}")

    def test_function_and_global_addresses_are_unique(self):
        for table in ("functions", "globals"):
            rows = (DB / f"{table}.tsv").read_text().split("\n")[1:]
            dup = sorted(k for k, n in collections.Counter(r.split("\t")[0] for r in rows if r).items() if n > 1)
            self.assertEqual(dup, [], f"{table}.tsv has duplicate addresses, e.g. {dup[:5]}")


if __name__ == "__main__":
    unittest.main()
