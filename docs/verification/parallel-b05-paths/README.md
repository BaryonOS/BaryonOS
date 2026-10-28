# B05a — bewijsverwijzingen bij een eigen M6b-buildmap

De vier host-/vergelijkingsgroepen in `tools/test_m6b.py` verwezen letterlijk
naar `build/logs`. Met `NOS_BUILD_DIR` kwamen de werkelijke logs elders terecht;
de ongewijzigde capturecontrole weigerde die verkeerde verwijzingen terecht.

De groepen verwijzen nu naar `BUILD / 'logs/…'`. `record` bewaart deze
absolute paden als tekst. De drie hostlogs liggen daardoor direct in de
daadwerkelijk gekozen `BUILD/logs`; de vergelijking verwijst naar het bestand
in dezelfde map. De eisen aan exacte bestandsnamen, commandoregels, aparte
fysieke bestanden, bronbinding en verplichte 32 groepen/28 VM's zijn niet
veranderd. De archiefbestemming blijft één directe child van
`docs/verification`.

Op basis `41bac3c173d464a286017d31558335d70899c434` slagen de bestaande
14 capturetests en 7 harnastests met een afzonderlijke `NOS_BUILD_DIR`.
`git diff --check` slaagt. `checks.json` bindt opdrachten, exitstatussen en
bronhashes; `capture.txt` en `harness.txt` bevatten de ruwe testuitvoer.

```sh
NOS_BUILD_DIR="$PWD/build/parallel-b05-paths" python3 -B -m unittest discover -s tests -p test_m6b_capture.py -v
NOS_BUILD_DIR="$PWD/build/parallel-b05-paths" python3 -B -m unittest discover -s tests -p test_m6b_harness.py -v
```

Dit zijn hostcontroles, geen native M6b-uitvoering. De volledige native gate
en capture volgen pas na de door de hoofdagent bevestigde bronfreeze. Er is
geen nieuwe roadmapscore of duurzaamheidsclaim door deze padcorrectie.
