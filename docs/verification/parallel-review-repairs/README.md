# Gerichte reparaties na CodeRabbit-review op3e41bbf9

De native80VM-gate hoort bij3e41bbf9 en blijft onder die binding bewaard.
Deze vervolgstap verandert hosttooling en voegt zelfstandige gebruikerslibs toe;
alle bestaande kernel/loader/M6-/qsim-probebron en hun buildinputs zijn
bytegelijk gebleven. native-input-applicability.json benoemt de exacte scope.
Nieuwe gerichte native runs en de G0-capture volgen op het volgende checkpoint.

Afgehandeld: directe nieuwe nativebewijspaden, historische B02bronbinding,
expliciete ADR-invarianten/weerleggingscriteria, gedeelde SDKbuildomgeving,
vroege weigering van QEMUbuildmappen buiten checkout en begrensde gitquery.
I09 herstelt standalone Rust-aanroepselectie en targetverwisseling binnen
sysroot: eigen rood/groen bewijs met21tests en echte --strict in ../parallel-i09/.
De parentmanifest vanqsim-core en de nieuwejobcodec zijn nu ook input van de
algemene bronfingerprint; een gerichte regressie bewaakt de eerdere omissie.

Het voorgestelde linkerprobleem is in onze gepinde Rusttoolchain niet
reproduceerbaar: de oude SDKbuild slaagt ook met rust-lld buiten het initiële
PATH. Dat echte resultaat blijft bewaard; er is geen rode build verzonnen.
De nieuwe scriptversie gebruikt alsnog de gedeelde gecontroleerde omgeving.
Beide echte native voorbeeldbuilds zijn succesvol met eigen verse targets;
exacte opdrachten, exits, script-/binaryhashes staan in sdk-linker-checks.json.
Dit zijn cross-compilaties, geen gestart gebruikersprogramma.

De buildrootregressies testen allevier runners op externe/similarlynamedpaden
en symlinkontsnapping vóór firmware/toolinvocatie of directorycreatie.
De grote native suites worden niet herhaald voor wijzigingen die hun
gecontroleerde kernel-/dienstbron en gedrag niet veranderen; nieuwe
hostcontroles en gerichte actuele runs controleren de gewijzigde grenzen.
