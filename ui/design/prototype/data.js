// Sample data for the BaryonOS prototype. Fictional; nothing here is read from
// or sent to a real system. Times are fixed so screenshots are reproducible.
export const DATA = {
  now: { iso: '2026-11-13T09:41', clock: '09:41', date: 'vrijdag 13 november', short: 'vr 13 nov' },
  user: { id: 'noor', name: 'Noor', full: 'Noor de Vries', initials: 'N' },
  users: [
    { id: 'noor', name: 'Noor', initials: 'N' },
    { id: 'sem', name: 'Sem', initials: 'S' },
  ],
  device: { name: 'Werkplek van Noor', version: 'BaryonOS 0.9 · simulatie', storage: { used: '182 GB', total: '512 GB', ratio: 0.36 } },
  nas: { name: 'Thuis-NAS', online: true, used: '1,4 TB', total: '4 TB', ratio: 0.35 },

  // Orbit order; `pinned` apps stay in Orbit when closed.
  apps: [
    { id: 'bestanden', pinned: true },
    { id: 'notities', pinned: true },
    { id: 'browser', pinned: true },
    { id: 'fotos', pinned: true },
    { id: 'haven', pinned: true },
    { id: 'instellingen', pinned: true },
    { id: 'rekenmachine', pinned: false },
    { id: 'muziek', pinned: false },
  ],

  collections: [
    { id: 'tuin', name: 'Tuin' },
    { id: 'verhuizing', name: 'Verhuizing' },
    { id: 'reizen', name: 'Reizen' },
    { id: 'cursus', name: 'Cursus pottenbakken' }, // intentionally empty
  ],

  files: [
    {
      id: 'tuinplan', name: 'Tuinplan', kind: 'note', place: 'documents', collection: 'tuin',
      storage: 'both', sync: 'ok', changed: 'Vandaag 09:12', favourite: true, recent: 1,
      grants: [{ app: 'notities', rights: 'readWrite', until: 'close' }],
      versions: [
        { n: 1, when: 'ma 9 nov 20:14', body: 'Moestuin achterin\n- Tomaten langs het hek\n- Pompoen in de hoek\n\nVoortuin\n- Lavendel bij de deur' },
        { n: 2, when: 'di 10 nov 19:02', body: 'Moestuin achterin\n- Tomaten langs het hek\n- Pompoen in de hoek\n- Bonen tegen de schutting\n\nVoortuin\n- Lavendel bij de deur' },
        { n: 3, when: 'gisteren 21:40', body: 'Moestuin achterin\n- Tomaten langs het hek\n- Bonen tegen de schutting\n- Courgette naast de compostbak\n\nVoortuin\n- Lavendel bij de deur\n- Vlinderstruik bij het raam' },
        { n: 4, when: 'vandaag 09:12', body: 'Moestuin achterin\n- Tomaten langs het hek (vier planten)\n- Bonen tegen de schutting\n- Courgette naast de compostbak\n\nVoortuin\n- Lavendel bij de deur\n- Vlinderstruik bij het raam\n- Regenton aansluiten op de goot' },
      ],
    },
    {
      id: 'plattegrond', name: 'Plattegrond nieuwe woning', kind: 'drawing', place: 'documents', collection: 'verhuizing',
      storage: 'both', sync: 'ok', changed: 'Gisteren 16:30', favourite: false, recent: 2, grants: [],
      versions: [{ n: 1, when: 'di 10 nov 14:05' }, { n: 2, when: 'gisteren 16:30' }],
    },
    {
      id: 'boodschappen', name: 'Boodschappen', kind: 'list', place: 'desktop', collection: null,
      storage: 'local', sync: 'ok', changed: 'Vandaag 08:05', favourite: false, recent: 3, grants: [],
      versions: [{ n: 1, when: 'vandaag 08:05', body: 'Havermout\nAppels\nKoffiebonen\nZaaigrond 40 l' }],
    },
    {
      id: 'huurcontract', name: 'Huurcontract Spoorstraat 12', kind: 'pdf', place: 'documents', collection: 'verhuizing',
      storage: 'nas', sync: 'ok', changed: 'ma 2 nov', favourite: true, recent: 0, grants: [],
      versions: [{ n: 1, when: 'ma 2 nov 11:20' }],
    },
    {
      id: 'begroting', name: 'Begroting verbouwing', kind: 'sheet', place: 'documents', collection: 'verhuizing',
      storage: 'both', sync: 'busy', changed: 'Vandaag 09:30', favourite: false, recent: 4, grants: [],
      versions: [{ n: 1, when: 'wo 4 nov 20:00' }, { n: 2, when: 'vandaag 09:30' }],
    },
    {
      id: 'lofoten', name: 'Reisplan Lofoten', kind: 'note', place: 'documents', collection: 'reizen',
      storage: 'nas', sync: 'ok', changed: 'zo 1 nov', favourite: false, recent: 0, grants: [],
      versions: [{ n: 1, when: 'zo 1 nov 17:45', body: 'Vlucht naar Bodø op 14 maart\nVeerboot naar Moskenes\nHuisje in Reine, drie nachten' }],
    },
    {
      id: 'zaaikalender', name: 'Zaaikalender 2027', kind: 'sheet', place: 'downloads', collection: 'tuin',
      storage: 'local', sync: 'ok', changed: 'do 5 nov', favourite: false, recent: 0, grants: [],
      versions: [{ n: 1, when: 'do 5 nov 10:10' }],
    },
  ],

  trash: [],

  // Photos are drawn by the prototype from these palettes; no image files.
  photos: [
    { id: 'p1', title: 'Reine bij schemering', date: '14 mrt', hues: [222, 262, 18] },
    { id: 'p2', title: 'Moestuin in juni', date: '21 jun', hues: [118, 84, 46] },
    { id: 'p3', title: 'Duinen bij Schoorl', date: '3 aug', hues: [38, 200, 28] },
    { id: 'p4', title: 'Noorderlicht', date: '15 mrt', hues: [150, 190, 260] },
    { id: 'p5', title: 'Herfst in het Amsterdamse Bos', date: '18 okt', hues: [24, 12, 48] },
    { id: 'p6', title: 'Nieuwe woonkamer', date: '9 nov', hues: [32, 210, 40] },
    { id: 'p7', title: 'Regen op het raam', date: '11 nov', hues: [208, 220, 196] },
    { id: 'p8', title: 'Pottenbakkersschijf', date: '2 nov', hues: [20, 30, 8] },
  ],

  haven: {
    featured: 'sterrenkaart',
    categories: ['Creatief', 'Leren', 'Productiviteit', 'Hulpmiddelen'],
    catalog: [
      {
        id: 'sterrenkaart', name: 'Sterrenkaart', maker: 'Atelier Zwerk', version: '2.3', size: '18 MB', category: 'Leren',
        tagline: 'De hemel boven je, ster voor ster uitgelegd.',
        wants: ['location'], installed: false,
      },
      {
        id: 'kleurwiel', name: 'Kleurwiel', maker: 'Studio Prisma', version: '1.8', size: '9 MB', category: 'Creatief',
        tagline: 'Kleurpaletten maken en vergelijken.', wants: ['files'], installed: false,
      },
      {
        id: 'metronoom', name: 'Metronoom', maker: 'Tik & Tel', version: '4.0', size: '3 MB', category: 'Hulpmiddelen',
        tagline: 'Strak in de maat, ook zonder internet.', wants: [], installed: false,
      },
      {
        id: 'logboek', name: 'Logboek', maker: 'Veldwerk', version: '1.2', size: '6 MB', category: 'Productiviteit',
        tagline: 'Een dagboek dat op je eigen apparaat blijft.', wants: ['files'], installed: false,
      },
    ],
  },

  // The installed third-party web app used for the permission journey.
  webvenster: { origin: 'tuinplanner.example', title: 'Tuinplanner', file: 'tuinplan' },

  notifications: [
    { id: 'n1', app: 'instellingen', title: 'BaryonOS is bijgewerkt', body: 'Versie 0.9 is geïnstalleerd. Je kunt de vorige versie terugzetten via Herstel.', when: '07:58' },
    { id: 'n2', app: 'bestanden', title: 'Thuis-NAS bijgewerkt', body: '12 bestanden staan versleuteld op Thuis-NAS.', when: '08:30' },
  ],

  // Recent use of granted access, shown in Instellingen › Privacy en beveiliging.
  accessHistory: [
    { app: 'notities', what: 'Tuinplan lezen en wijzigen', when: 'Vandaag 09:12' },
  ],
}
