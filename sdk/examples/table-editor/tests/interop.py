"""Independent Python CSV oracle for the real host consumer, never native evidence."""
import csv
import io
import itertools
from pathlib import Path
import subprocess
import sys

binary = Path(sys.argv[1]).resolve(strict=True)
alphabet = ["", "abc", "Zoë", "a,b", 'a"b', "a\r\nb", "李", " =literal"]
count = 0
for first, second in itertools.product(alphabet, repeat=2):
    rows = [["Naam", "Notitie"], [first, second]]
    source = io.StringIO(newline="")
    csv.writer(source, lineterminator="\r\n").writerows(rows)
    result = subprocess.run(
        [str(binary), "--set", "2", "1", first + "!"],
        input=source.getvalue().encode(),
        capture_output=True,
        check=True,
        timeout=10,
    )
    assert not result.stderr
    actual = list(csv.reader(io.StringIO(result.stdout.decode(), newline=""), strict=True))
    assert actual == [rows[0], [first + "!", second]], (rows, actual)
    count += 1
print(f"Python standard-library csv independently reopened {count} real host CLI edit/export outputs; all exact cell values match.")
