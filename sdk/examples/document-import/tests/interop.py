#!/usr/bin/env python3
"""Independent Python ZIP/XML fixtures passed through the real DOCX CLI."""
import io
import pathlib
import struct
import subprocess
import sys
import warnings
import zipfile
import xml.etree.ElementTree as ET

WORD = "http://schemas.openxmlformats.org/wordprocessingml/2006/main"
RELS = "http://schemas.openxmlformats.org/package/2006/relationships"
TYPES = "http://schemas.openxmlformats.org/package/2006/content-types"
MAIN_TYPE = "application/vnd.openxmlformats-officedocument.wordprocessingml.document.main+xml"
OFFICE = "http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument"
XML_SPACE = "{http://www.w3.org/XML/1998/namespace}space"


def main_xml(paragraphs):
    root = ET.Element(f"{{{WORD}}}document")
    body = ET.SubElement(root, f"{{{WORD}}}body")
    for text in paragraphs:
        p = ET.SubElement(body, f"{{{WORD}}}p")
        # Multiple real runs must concatenate without invented spaces.
        for value in (text[:len(text) // 2], text[len(text) // 2:]):
            r = ET.SubElement(p, f"{{{WORD}}}r")
            t = ET.SubElement(r, f"{{{WORD}}}t", {XML_SPACE: "preserve"})
            t.text = value
    sect = ET.SubElement(body, f"{{{WORD}}}sectPr")
    ET.SubElement(sect, f"{{{WORD}}}pgSz", {f"{{{WORD}}}w": "11906"})
    return ET.tostring(root, encoding="utf-8", xml_declaration=True)


def parts(paragraphs):
    types = ET.Element(f"{{{TYPES}}}Types")
    ET.SubElement(types, f"{{{TYPES}}}Override",
                  {"PartName": "/word/document.xml", "ContentType": MAIN_TYPE})
    rels = ET.Element(f"{{{RELS}}}Relationships")
    ET.SubElement(rels, f"{{{RELS}}}Relationship",
                  {"Id": "rId1", "Type": OFFICE, "Target": "word/document.xml"})
    return {
        "[Content_Types].xml": ET.tostring(types),
        "_rels/.rels": ET.tostring(rels),
        "word/document.xml": main_xml(paragraphs),
    }


class Unseekable(io.BytesIO):
    def seekable(self):
        return False

    def seek(self, *args):
        raise io.UnsupportedOperation("force streaming ZIP data descriptors")


def package(entries, compression=zipfile.ZIP_DEFLATED, descriptor=False):
    stream = Unseekable() if descriptor else io.BytesIO()
    with warnings.catch_warnings():
        warnings.simplefilter("ignore", UserWarning)
        with zipfile.ZipFile(stream, "w", compression=compression) as out:
            for name, value in entries:
                out.writestr(name, value)
    return stream.getvalue()


def cli(binary, source, success=None, args=("--plain-text",)):
    result = subprocess.run([binary, *args], input=source, capture_output=True, timeout=5)
    if success is None:
        assert result.returncode != 0 and result.stdout == b"" and result.stderr, result
    else:
        assert result.returncode == 0 and result.stdout == success and result.stderr == b"", result


def main():
    binary = str(pathlib.Path(sys.argv[1]).resolve())
    target = pathlib.Path(sys.argv[2])
    target.parent.mkdir(parents=True, exist_ok=True)
    count = 0
    for compression in (zipfile.ZIP_STORED, zipfile.ZIP_DEFLATED):
        for descriptor in (False, True):
            for i in range(20):
                paragraphs = [f" Café & <漢字> 😀 {i}", "", "\tspaties  blijven ", "einde", ""]
                paragraphs[0] *= i + 1
                source = package(parts(paragraphs).items(), compression, descriptor)
                cli(binary, source, "\n".join(paragraphs).encode())
                count += 1
                target.write_bytes(source)
    basic = parts(["invoer", "", "einde"])
    valid = package(basic.items(), zipfile.ZIP_STORED)
    failures = 0

    def deny(source, args=("--plain-text",)):
        nonlocal failures
        cli(binary, source, args=args)
        failures += 1

    for stop in range(len(valid)):
        deny(valid[:stop])
    for args in ((), ("--help",), ("--plain-text", "--unknown")):
        deny(valid, args)
    deny(valid + b"trailing")
    deny(package([*basic.items(), ("word/document.xml", main_xml(["duplicate"]))]))
    deny(package([*basic.items(), ("WORD/DOCUMENT.XML", main_xml(["case collision"]))]))
    deny(package([*basic.items(), ("../escape", b"x")]))
    deny(package([*basic.items(), ("large", b"a" * (1024 * 1024 + 1))]))
    for key in basic:
        deny(package((k, v) for k, v in basic.items() if k != key))
    for entry_count in (511, 512):
        source = package([*basic.items(), *((f"extra/{i}", b"") for i in range(entry_count - 3))])
        cli(binary, source, b"invoer\n\neinde")
        count += 1
    deny(package([*basic.items(), *((f"extra/{i}", b"") for i in range(510))]))
    for fragment in (
        "<w:tbl/>", "<w:p><w:r><w:fldChar/></w:r></w:p>",
        "<w:p><w:hyperlink/></w:p>",
        "<w:p><w:r><w:drawing/></w:r></w:p>",
        "<w:p><w:r><w:t xmlns:w='urn:fake'>forged</w:t></w:r></w:p>",
        "<w:p><w:pPr><w:t>hidden omission</w:t></w:pPr></w:p>",
        "<w:sectPr><w:headerReference/></w:sectPr>",
    ):
        changed = dict(basic)
        changed["word/document.xml"] = (
            f'<w:document xmlns:w="{WORD}"><w:body>{fragment}</w:body></w:document>'
        ).encode()
        deny(package(changed.items()))
    for old, new in (
        (b'Target="word/document.xml"', b'xmlns:evil="urn:fake" evil:Target="word/document.xml"'),
        (b'Target="word/document.xml"', b'TargetMode="External" Target="https://invalid.invalid/x"'),
        (OFFICE.encode(), b"urn:not-office"),
    ):
        changed = dict(basic)
        changed["_rels/.rels"] = basic["_rels/.rels"].replace(old, new)
        deny(package(changed.items()))
    changed = dict(basic)
    changed["[Content_Types].xml"] = basic["[Content_Types].xml"].replace(MAIN_TYPE.encode(), b"application/vnd.ms-word.document.macroEnabled.main+xml")
    deny(package(changed.items()))
    changed = dict(basic)
    changed["word/document.xml"] = b"<!DOCTYPE x [<!ENTITY ext SYSTEM 'file:///does-not-exist'>]><x>&ext;</x>"
    deny(package(changed.items()))
    # Mutate metadata that the independent writer produced.
    for at, mask in ((6, 1), (8, 0x40), (14, 1), (30, 1)):
        corrupt = bytearray(valid)
        corrupt[at] ^= mask
        deny(bytes(corrupt))
    bomb = dict(basic)
    bomb["word/document.xml"] = b"a" * (1024 * 1024 + 1)
    compressed = bytearray(package(bomb.items()))
    cd = compressed.find(b"PK\x01\x02")
    while cd >= 0:
        name_len, extra_len, comment_len = struct.unpack_from("<HHH", compressed, cd + 28)
        name = bytes(compressed[cd + 46:cd + 46 + name_len])
        if name == b"word/document.xml":
            local = struct.unpack_from("<I", compressed, cd + 42)[0]
            struct.pack_into("<I", compressed, cd + 24, 16)
            struct.pack_into("<I", compressed, local + 22, 16)
            break
        cd = compressed.find(b"PK\x01\x02", cd + 46 + name_len + extra_len + comment_len)
    deny(bytes(compressed))
    # A signed descriptor can legally be stored without its optional signature.
    streaming = package(basic.items(), zipfile.ZIP_DEFLATED, descriptor=True)
    positions = []
    with zipfile.ZipFile(io.BytesIO(streaming)) as reader:
        for info in reader.infolist():
            name_len, extra_len = struct.unpack_from("<HH", streaming, info.header_offset + 26)
            data_start = info.header_offset + 30 + name_len + extra_len
            descriptor = data_start + info.compress_size
            assert streaming[descriptor:descriptor + 4] == b"PK\x07\x08"
            positions.append(descriptor)
    without = bytearray(streaming)
    for position in reversed(positions):
        del without[position:position + 4]
    original_cd = struct.unpack_from("<I", streaming, len(streaming) - 6)[0]
    cd = original_cd - len(positions) * 4
    for _ in positions:
        local = struct.unpack_from("<I", without, cd + 42)[0]
        struct.pack_into("<I", without, cd + 42, local - 4 * sum(p < local for p in positions))
        sizes = struct.unpack_from("<HHH", without, cd + 28)
        cd += 46 + sum(sizes)
    struct.pack_into("<I", without, len(without) - 6, original_cd - len(positions) * 4)
    cli(binary, bytes(without), b"invoer\n\neinde")
    count += 1
    # Independently confirm that the published fixture is a valid three-part DOCX ZIP.
    with zipfile.ZipFile(target) as package_reader:
        assert package_reader.testzip() is None
        assert len(package_reader.namelist()) == 3
        assert ET.fromstring(package_reader.read("word/document.xml")).tag == f"{{{WORD}}}document"
    print(f"{count} exact DOCX-to-text outputs (80 content, 2 entry boundaries, 1 unsigned descriptor); {failures} malformed/unsupported/CLI denials")


if __name__ == "__main__":
    main()
