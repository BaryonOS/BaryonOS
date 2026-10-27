// Synthetic examples only. No personal files, live device identifiers or accounts.
export const STORAGE_KEY = 'nieuw-os-prototype-v1';
export const NAS_IDENTITY = 'NAS-DEMO-01';
export const COLLECTIONS = ['Vandaag', 'Huis', 'Archief'];

export function createFixtureState() {
  return {
    schemaVersion: 1,
    onboarding: { step: 1, completed: false, name: 'Lokaal profiel', reducedMotion: false },
    objects: [
      {
        id: 'obj-001', title: 'Tuinplan', mediaType: 'text', collectionId: 'Huis',
        revision: 3, localState: 'saved', nasState: 'local',
        versions: [
          { id: 1, label: 'Eerste ideeën', text: 'Een tuin om in te blijven.\n\nEen bank in de ochtendzon, kruiden bij de keuken en ruimte voor wilde bloemen.\n\nBegin klein: eerst de bodem, dan de planten.' },
          { id: 2, label: 'Indeling toegevoegd', text: 'Een tuin om in te blijven.\n\nEen bank in de ochtendzon, kruiden bij de keuken en ruimte voor wilde bloemen.\n\nLangs de muur komt een verhoogde border. Het midden blijft open. Een smal pad verbindt het terras met de schuur.' },
          { id: 3, label: 'Planten en volgende stappen', text: 'Een tuin om in te blijven.\n\nEen bank in de ochtendzon, kruiden bij de keuken en ruimte voor wilde bloemen.\n\nDe indeling\nLangs de muur komt een verhoogde border. Het midden blijft open. Een smal pad verbindt het terras met de schuur.\n\nWat we planten\nRozemarijn en tijm bij de keuken. Salie in de zonnige border. Wilde bloemen langs het pad, zodat er het hele seizoen iets bloeit.\n\nDe volgende stap\nDe maten controleren en de plekken met ochtendzon op de plattegrond aangeven.' },
        ],
      },
      {
        id: 'obj-002', title: 'Plattegrond', mediaType: 'image', collectionId: 'Huis',
        revision: 1, localState: 'saved', nasState: 'local',
        versions: [{ id: 1, label: 'Tuinindeling', text: 'Fictieve tuin: terras, border, open ruimte en schuur. De tekening heeft geen schaal.' }],
      },
      {
        id: 'obj-003', title: 'Boodschappen', mediaType: 'text', collectionId: 'Vandaag',
        revision: 1, localState: 'saved', nasState: 'local',
        versions: [{ id: 1, label: 'Eerste lijst', text: 'Voor het weekend\n\nBrood\nTomaten\nOlijfolie\nVerse kruiden\n\nEven kijken wat er nog in huis is.' }],
      },
    ],
    tools: [
      { id: 'notities', name: 'Notities', description: 'Schrijven, lezen en rustig verder werken.', installed: true, removable: false, types: ['text'] },
      { id: 'beeldbewerker', name: 'Beeldbewerker', description: 'Een lokaal voorbeeldpakket voor afbeeldingen.', installed: false, removable: true, types: ['image'] },
      { id: 'webvenster', name: 'Webvenster', description: 'Voorbeeld van een gereedschap met afzonderlijk internetrecht.', installed: true, removable: false, types: ['text', 'image'] },
    ],
    selectedObjectId: null, lastOpenedId: 'obj-001', draft: null,
    mode: 'read', restoreRevision: null, grants: [], grantRequest: null,
    activeGrantId: null, lastGrantState: null,
    nas: { state: 'unpaired', identity: null, datasets: ['Huis'] },
    ai: { state: 'ready', availability: 'ready', context: ['obj-001'], response: '', runId: 0 },
    install: { state: 'idle', toolId: null },
    conditions: { invalidPackage: false },
    message: null, counter: 0,
  };
}
