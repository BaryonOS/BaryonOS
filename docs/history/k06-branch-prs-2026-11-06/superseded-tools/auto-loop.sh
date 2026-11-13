#!/usr/bin/env bash
set -euo pipefail

echo "🔄 Start de autonome nieuw-os Agent Loop..."

# Alleen in een schone werkboom op een werkbranch: `git add -A` hieronder
# commit dan uitsluitend wat deze loop zelf heeft gemaakt.
BRANCH=$(git branch --show-current)
if [ -z "$BRANCH" ] || [ "$BRANCH" = "main" ] || [ "$BRANCH" = "master" ]; then
    echo "❌ Start de loop op een eigen werkbranch, niet op '${BRANCH:-detached HEAD}'." >&2
    exit 2
fi
if [ -n "$(git status --porcelain)" ]; then
    echo "❌ Werkboom niet schoon; commit of stash eerst je eigen werk." >&2
    exit 2
fi
SPEC_FILE=$(mktemp "${TMPDIR:-/tmp}/nieuw_os_task.XXXXXX")
trap 'rm -f "$SPEC_FILE"' EXIT
PREVIOUS_TASK=""

while true; do
    echo "--------------------------------------------------"
    echo "📋 [Stap 1] Bepaal eerstvolgende openstaande taak via progress.json..."
    
    # Zoek de eerste openstaande taak op uit docs/progress.json
    NEXT_TASK=$(python3 -c "
import json
data = json.load(open('docs/progress.json'))
for track in data['tracks']:
    for gate in track['gates']:
        if gate['status'] == 'open':
            print(f\"{track['id']}:{gate['id']} - {gate['title']}\")
            exit(0)
print('ALL_DONE')
")

    if [ "$NEXT_TASK" = "ALL_DONE" ]; then
        echo "🎉 Alle mijlpalen in de roadmap zijn volbracht!"
        break
    fi

    # Een geslaagde gate sluit geen criterium (native bewijs en review blijven nodig):
    # dezelfde open taak twee keer achter elkaar betekent stoppen, niet herhalen.
    if [ "$NEXT_TASK" = "$PREVIOUS_TASK" ]; then
        echo "⏸️  $NEXT_TASK staat nog open na een geslaagde iteratie; de loop stopt voor review." >&2
        exit 3
    fi
    PREVIOUS_TASK="$NEXT_TASK"

    echo "🎯 Volgende doel gedetecteerd: $NEXT_TASK"

    # Tijdelijke specificatie voor deze iteratie (eigen mktemp-bestand, zie boven)
    echo "Implementeer de volgende openstaande mijlpaal: $NEXT_TASK. Raadpleeg de gehoste Graphify MCP voor afhankelijkheden en structuren. Schrijf minimale, veilige code volgens AGENTS.md en Ponytail." > "$SPEC_FILE"

    echo "🚀 [Stap 2] Start schone Claude Code sessie voor deze taak..."
    
    # Start een vers, non-interactief proces (voorkomt context rot volledig)
    claude -p "Lees $SPEC_FILE. Analyseer de codebase, implementeer de benodigde wijzigingen strikt volgens AGENTS.md en Ponytail, en zorg dat alle onderdelen correct zijn opgebouwd." \
        --allowedTools "mcp__graphify__*,Read,Write,Edit,MultiEdit,Bash"

    echo "🔍 [Stap 3] Verifieer via de uniforme gate..."
    if ./tools/gate.sh; then
        echo "✅ Gate geslaagd voor: $NEXT_TASK"
        
        # Leg de wijzigingen vast in Git
        git add -A
        git commit -m "feat(auto): autonome implementatie van $NEXT_TASK"
        
        echo "🔄 Voortgang vastgelegd. Start direct volgende iteratie met schone context..."
    else
        echo "❌ Gate gefaald voor $NEXT_TASK. De loop pauzeert zodat je kunt ingrijpen." >&2
        exit 2
    fi

    # Korte pauze tussen iteraties
    sleep 3
done
