#!/usr/bin/env python3
"""Independent ZIP/OPC/XML oracle over actual host CLI output; no product reader."""
from io import BytesIO
import itertools
from pathlib import Path
import subprocess
import sys
import xml.etree.ElementTree as ET
import zipfile

W = "{http://schemas.openxmlformats.org/wordprocessingml/2006/main}"
P = "{http://schemas.openxmlformats.org/package/2006/relationships}"
C = "{http://schemas.openxmlformats.org/package/2006/content-types}"


def verify(binary, text):
    result = subprocess.run([str(binary)], input=text.encode(), capture_output=True, check=True)
    assert not result.stderr
    with zipfile.ZipFile(BytesIO(result.stdout)) as package:
        assert package.testzip() is None
        assert package.namelist() == ["[Content_Types].xml", "_rels/.rels", "word/document.xml"]
        for entry in package.infolist():
            assert entry.compress_type == zipfile.ZIP_STORED
            assert not entry.flag_bits and not entry.extra and not entry.comment
            assert entry.date_time == (1980, 1, 1, 0, 0, 0)
        relationships = ET.fromstring(package.read("_rels/.rels"))
        assert relationships.tag == P + "Relationships" and len(relationships) == 1
        relation = relationships[0]
        assert relation.tag == P + "Relationship"
        assert relation.attrib == {
            "Id": "rId1", "Target": "word/document.xml",
            "Type": "http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument",
        }
        types = ET.fromstring(package.read("[Content_Types].xml"))
        assert types.tag == C + "Types"
        override = types.find(C + "Override")
        assert override.attrib == {
            "PartName": "/word/document.xml",
            "ContentType": "application/vnd.openxmlformats-officedocument.wordprocessingml.document.main+xml",
        }
        root = ET.fromstring(package.read("word/document.xml"))
        assert root.tag == W + "document"
        body = root.find(W + "body")
        paragraphs = []
        for paragraph in body:
            assert paragraph.tag == W + "p" and len(paragraph) == 1
            run = paragraph[0]
            assert run.tag == W + "r"
            words = []
            for node in run:
                if node.tag == W + "t":
                    assert node.attrib == {"{http://www.w3.org/XML/1998/namespace}space": "preserve"}
                    words.append(node.text or "")
                else:
                    assert node.tag == W + "tab" and not node.attrib
                    words.append("\t")
            paragraphs.append("".join(words))
        assert "\n".join(paragraphs) == text
    return result.stdout


def main():
    binary = Path(sys.argv[1]).resolve()
    cases = ["", "Hallo wereld", "Café — 漢字 😀\n\n  spaties \t tab\t\n",
             '</w:t><w:fldSimple w:instr="HYPERLINK https://example.invalid"/>&',
             "\ufeffBOM is inhoud", "\u202eBidi-inhoud", "a" * 65536, "\n" * 4095]
    for parts in itertools.product(["a", "<&>", "漢字", "\t", "\n"], repeat=3):
        cases.append("".join(parts))
    for text in cases:
        verify(binary, text)
    if len(sys.argv) > 2:
        Path(sys.argv[2]).write_bytes(verify(binary, cases[2]))
    print(f"{len(cases)} actual CLI DOCX exports: ZIP CRC, fixed OPC parts, namespaces and exact Unicode/tab/paragraph text independently verified")


if __name__ == "__main__":
    main()
