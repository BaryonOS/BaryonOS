#!/usr/bin/env python3
"""Build ui/design/concepts/orbitaal-v2/index.html from v1 (read-only) + v2.css + new body markup.

Run from the worktree root:  python3 -I build/design-evidence/orbitaal-v2-run2/build_v2.py
Byte-identical slices from v1: the mark symbols, the wallpaper symbols, the boot svg and the loader svg.
Wordmark paths are copied from ui/design/brand/lockup-{dark,light}.svg.
Measured geometry (tethers, --ty, --tb, kern top) lives in GEO and is rewritten after a render pass.
"""
import json, os, re, sys

ROOT = os.getcwd()
V1 = os.path.join(ROOT, 'ui/design/concepts/orbitaal/index.html')
TRI = os.path.join(ROOT, 'ui/design/concepts/triade/index.html')
OUT = os.path.join(ROOT, 'ui/design/concepts/orbitaal-v2/index.html')
HERE = os.path.dirname(os.path.abspath(__file__))
CSS = os.path.join(HERE, 'v2.css')
GEOF = os.path.join(HERE, 'geo.json')

v1 = open(V1, encoding='utf-8').read()
css = open(CSS, encoding='utf-8').read()
GEO = {  # first-pass guesses; overwritten by geo.json after measuring
    'k4_top': 300, 'k4_left': 560, 'teth4_left': 900, 'teth4_top': 790, 'teth4_h': 38,
    'it3_left': 616, 'it3_top': 71, 'it3_w': 12, 'ty3': 62, 'tb3': 70,
}
if os.path.exists(GEOF):
    GEO.update(json.load(open(GEOF)))


def one(hay, needle):
    assert hay.count(needle) == 1, (needle[:80], hay.count(needle))
    return hay.index(needle)


def slice_between(hay, start, end_after):
    i = one(hay, start)
    j = hay.index(end_after, i) + len(end_after)
    return hay[i:j]


# ---------- sprite (from v1) ----------
sprite = slice_between(v1, '<!-- ======================= SPRITE', '</defs>\n</svg>')
marks_v1 = [slice_between(v1, '<symbol id="%s"' % k, '</symbol>') for k in ('mk-p', 'mk-l', 'mk-c')]
wp_v1 = [slice_between(v1, '<symbol id="%s"' % k, '</symbol>') for k in ('wp-dk', 'wp-lt')]


def rep(s, a, b):
    one(s, a)
    return s.replace(a, b)


haven_old = slice_between(sprite, '<symbol id="i-haven"', '</symbol>')
sprite = rep(sprite, haven_old, '<symbol id="i-haven" viewBox="0 0 24 24"><path d="M3.5 13h17l-2.4 4.8a2 2 0 0 1-1.8 1.2H7.7a2 2 0 0 1-1.8-1.2z"/><path d="M12 13V5.8"/><path d="M12 7l5.2 6"/><circle cx="12" cy="4.4" r="1.7" style="fill:var(--pc,currentColor);stroke:none"/></symbol>')
sprite = rep(sprite, slice_between(sprite, '<symbol id="st-net"', '</symbol>'),
             '<symbol id="st-net" viewBox="0 0 14 14"><circle cx="7" cy="7" r="4.6" style="fill:none;stroke:var(--c);stroke-width:1.75"/></symbol>')
sprite = rep(sprite, slice_between(sprite, '<symbol id="st-err"', '</symbol>'),
             '<symbol id="st-err" viewBox="0 0 14 14"><rect x="2.5" y="2.5" width="9" height="9" rx="1.5" style="fill:var(--c)"/></symbol>')
sprite = rep(sprite, '<!-- status shapes: never colour alone (solid = this device, shelled = own NAS, leaving orbit = internet) -->',
             '<!-- G5 place/state shapes, always with an icon and text: solid dot = this device, ring with dot = NAS, open ring = internet, square = error -->')
tri_src = open(TRI, encoding='utf-8').read()
lifted = []
for k in ('coll', 'huis', 'archief', 'web', 'klok', 'apps', 'user', 'a11y'):
    sym = slice_between(tri_src, '<symbol id="i-%s"' % k, '</symbol>')
    assert sym.count('fill="currentColor" stroke="none"') == 1, k
    lifted.append(' ' + sym.replace('fill="currentColor" stroke="none"', 'style="fill:var(--pc,currentColor);stroke:none"'))
sprite = rep(sprite, '<symbol id="i-chev" viewBox="0 0 24 24"><path d="M10 7l5 5-5 5"/></symbol>\n',
             '<symbol id="i-chev" viewBox="0 0 24 24"><path d="M10 7l5 5-5 5"/></symbol>\n <!-- lifted from Triade (collections, third-party web app, settings categories) -->\n' + '\n'.join(lifted) + '\n')
sprite = rep(sprite, ' <!-- mark-primary.svg (dark surfaces) -->',
             ' <!-- G8 exemption pending the project owner\'s decision: the marks, the boot animation and the wallpapers keep the three charges -->\n <!-- mark-primary.svg (dark surfaces) -->')


def lockup_paths(fn):
    t = open(os.path.join(ROOT, 'ui/design/brand', fn), encoding='utf-8').read()
    ps = re.findall(r'<path fill="[^"]*" d="[^"]*"/>', t)
    assert len(ps) == 2, fn
    return ps


wm_dk, wm_lt = lockup_paths('lockup-dark.svg'), lockup_paths('lockup-light.svg')
sprite = rep(sprite, '</defs>\n</svg>',
             ' <!-- wordmark v2 (Saira-derived outlines, copied byte-identically from ui/design/brand/lockup-*.svg; owner-redirectable) -->\n'
             ' <symbol id="wm-dk" viewBox="309 119 535 88">' + ''.join(wm_dk) + '</symbol>\n'
             ' <symbol id="wm-lt" viewBox="309 119 535 88">' + ''.join(wm_lt) + '</symbol>\n</defs>\n</svg>')
for m in marks_v1 + wp_v1:
    one(sprite, m)

boot_svg = slice_between(v1, '<svg class="boot anim"', '</svg>')
loader_svg = slice_between(v1, '<svg class="anim" width="32" height="32"', '</svg>')
stages = slice_between(v1, '<div style="text-align:center"><svg width="40" height="40" viewBox="-250 -250 1500 1500"><g fill="none" stroke="#AFC0FF" stroke-opacity=".35" stroke-width="14"><circle cx="480" cy="565" r="700"/>',
                       '<div class="lab">sweep</div></div>')

# ---------- building blocks ----------
NUC = '<button class="ob%s" aria-label="BaryonOS-menu%s"%s><span class="nuc"><svg viewBox="0 0 1000 1000"><use href="#mk-c"/></svg></span></button>'
ZOEK = '<button class="ob" aria-label="Zoeken"><svg class="ic"><use href="#i-zoeken"/></svg></button>'
SEP = '<i class="sep" aria-hidden="true"></i>'
SEP3 = '<i class="sep" role="separator" aria-label="Apps van derden"></i>'
CLOCK = '<div class="clock"><span class="t-bodys dn num">14:32</span><span class="t-cap tx2">do <span class="num">13</span> nov</span></div>'


def nuc(state=''):
    cls = {'': '', 'act': ' act', 'open': ' open act'}[state]
    extra = ' aria-expanded="true"' if state == 'open' else ''
    return NUC % (cls, ', actief' if state else '', extra)


def app(name, icon, state='', tp=False):
    cls = 'ob' + {'': '', 'run': ' run', 'act': ' run act'}[state]
    lab = name + {'': '', 'run': ', geopend', 'act': ', actief'}[state] + (' · app van derden' if tp else '')
    return '<button class="%s" aria-label="%s"><span class="tile%s"><svg class="ic"><use href="#i-%s"/></svg></span></button>' % (cls, lab, ' tp' if tp else '', icon)


PLACES = [('p-dev', 's-dev', 'i-device', 'Dit apparaat'), ('p-nas', 's-nas', 'i-nas', 'Thuis-NAS'), ('p-net', 's-net', 'i-internet', 'Internet')]


def tri(mode, net_id=''):
    states = {'idle': ['lit', 'lit', 'off'], 'ask': ['lit', 'lit', 'ask'], 'cmp': ['lit', 'lit', 'off']}[mode]
    cols = []
    for (p, s, ic, lab), st in zip(PLACES, states):
        colcls = 'tri-col %s%s' % (p, '' if st == 'off' else ' ' + st)
        nodecls = 'node %s%s' % (s, {'lit': '', 'off': ' off', 'ask': ' ask'}[st])
        nid = ' id="%s"' % net_id if (net_id and p == 'p-net') else ''
        cols.append('<div class="%s"><i class="%s"%s></i><span class="lbl"><svg class="ic"><use href="#%s"/></svg>%s</span></div>'
                    % (colcls, nodecls, nid, ic, '' if mode == 'cmp' else lab))
    if mode == 'idle':
        text = '<span class="t-bodys dn">Op dit apparaat en Thuis-NAS</span><span class="t-cap tx2">Geen internet</span>'
        aria = 'Privacy: je werk staat op dit apparaat en op Thuis-NAS. Geen internet.'
    elif mode == 'ask':
        text = '<span class="t-bodys dn">Webvenster vraagt internet</span><span class="t-cap tx2">Wacht op jouw keuze</span>'
        aria = 'Privacy: Webvenster vraagt toegang tot internet. Dit apparaat en Thuis-NAS zijn actief.'
    else:
        text = '<span class="t-bodys dn">Apparaat en NAS · geen internet</span>'
        aria = 'Privacy: je werk staat op dit apparaat en op Thuis-NAS. Geen internet.'
    return ('<div class="tri%s" role="status" aria-label="%s"><div class="tri-track" aria-hidden="true">%s</div><div class="tri-text" aria-hidden="true">%s</div></div>'
            % ({'idle': '', 'ask': ' ask', 'cmp': ' cmp'}[mode], aria, ''.join(cols), text))


def orbit(parts, extra=''):
    return '<nav class="orbit" aria-label="Orbit"%s>\n  %s\n </nav>' % (extra, '\n  '.join(parts))


FIRST = lambda st=('', '', ''): [app('Bestanden', 'bestanden', st[0]), app('Notities', 'notities', st[1]), app('Instellingen', 'instellingen', st[2])]

WIN_WC = '<div class="wc"><button aria-label="Minimaliseren"><svg class="ic"><use href="#i-min"/></svg></button><button aria-label="Maximaliseren"><svg class="ic"><use href="#i-max"/></svg></button><button aria-label="Sluiten"><svg class="ic"><use href="#i-close"/></svg></button></div>'


def ptag(p, shape, icon, text, cls='t-cap tx2', style=''):
    return '<span class="ptag %s %s"%s><svg class="st"><use href="#%s"/></svg><svg class="ic"><use href="#%s"/></svg>%s</span>' % (p, cls, style, shape, icon, text)


def pill(p, shape, icon, text, err=False):
    return '<span class="pill %s"><svg class="st"><use href="#%s"/></svg><svg class="ic"><use href="#%s"/></svg>%s</span>' % ('err' if err else p, shape, icon, text)


# ---------- continuity (ab1 + HC mini) ----------
PAGE1 = '<span class="page"><i style="top:9px;width:22px;height:4px;background:#4A5470"></i><i style="top:19px;width:32px"></i><i style="top:26px;width:30px"></i><i style="top:33px;width:32px"></i><i style="top:40px;width:20px"></i></span>'
PAGE2 = '<span class="page"><svg viewBox="0 0 48 60"><g fill="none" stroke="#7B869E" stroke-width="1.2"><rect x="7" y="9" width="34" height="42" rx="1"/><path d="M7 30h15V9M22 38v13M30 30h11M30 30v8"/></g><circle cx="35" cy="17" r="2.2" fill="#7B869E"/></svg></span>'
PAGE3 = '<span class="page"><svg viewBox="0 0 48 60"><g fill="none" stroke="#7B869E" stroke-width="1.2"><rect x="8" y="12" width="6" height="6" rx="1.5"/><rect x="8" y="26" width="6" height="6" rx="1.5"/><rect x="8" y="40" width="6" height="6" rx="1.5"/></g><path d="M9.5 15l1.5 1.5 2.5-3" fill="none" stroke="#4A5470" stroke-width="1.4"/><g fill="#B9C1D2"><rect x="18" y="13.5" width="20" height="3" rx="1.5"/><rect x="18" y="27.5" width="16" height="3" rx="1.5"/><rect x="18" y="41.5" width="19" height="3" rx="1.5"/></g></svg></span>'
ITEMS = [
    (PAGE1, 'Tuinplan', 'Tekst · Huis · vandaag <span class="num">09:40</span>'),
    (PAGE2, 'Plattegrond', 'Tekening · Huis · gisteren <span class="num">18:12</span>'),
    (PAGE3, 'Boodschappen', 'Lijst · Vandaag · <span class="num">11</span> nov.'),
]


def continuity(hc=False):
    traj = ('<div class="traj" aria-hidden="true"><svg width="16" height="286" viewBox="0 0 16 286">'
            '<path d="M6 20Q14 150 6 280" fill="none" stroke="var(--tx2)" stroke-opacity=".45" stroke-width="1.2"/>'
            '<circle cx="9" cy="86" r="7" fill="var(--tx1)" opacity=".12"/><circle cx="9" cy="86" r="3.5" fill="var(--tx1)"/>'
            '<circle cx="10" cy="162" r="2.6" fill="var(--tx2)"/><circle cx="8.2" cy="238" r="2.6" fill="var(--tx2)" opacity=".7"/></svg></div>')
    its = []
    for n, (pg, name, meta) in enumerate(ITEMS):
        fx = ' fx' if (hc and n == 0) else ''
        its.append('<a class="it%s" href="#">%s<span><div class="t-bodys">%s</div><div class="t-body meta">%s</div></span></a>' % (fx, pg, name, meta))
    plate = ('<section class="plate-g frost" aria-label="Verder waar je was">\n   %s\n   <div class="t-title">Verder waar je was</div>\n   <div style="margin-top:6px">\n    %s\n   </div>\n  </section>'
             % (traj, '\n    '.join(its)))
    return '<div class="cont">\n  <div class="t-disp">Goedemiddag, Noor</div>\n  %s\n </div>' % plate


# ---------- Bestanden sidebar (ab3 + ab4) ----------
def side(full=True):
    rows = ['<div class="sr"><svg class="ic"><use href="#i-recent"/></svg><span class="t-body">Recent</span></div>',
            '<div class="sr"><svg class="ic"><use href="#i-desk"/></svg><span class="t-body">Bureaublad</span></div>',
            '<div class="sr"><svg class="ic"><use href="#i-doc"/></svg><span class="t-body">Documenten</span></div>',
            '<div class="sr"><svg class="ic"><use href="#i-image"/></svg><span class="t-body">Afbeeldingen</span></div>',
            '<div class="sr"><svg class="ic"><use href="#i-download"/></svg><span class="t-body">Downloads</span></div>',
            '<div class="sh t-cap">Collecties</div>',
            '<div class="sr"><svg class="ic"><use href="#i-coll"/></svg><span class="t-body">Vandaag</span></div>',
            ('<div class="sr sel" aria-current="true"><svg class="ic"><use href="#i-huis"/></svg><span class="t-bodys">Huis</span></div>' if full else
             '<div class="sr" style="background:var(--hov2)"><svg class="ic"><use href="#i-huis"/></svg><span class="t-bodys">Huis</span></div>'),
            '<div class="sr"><svg class="ic"><use href="#i-archief"/></svg><span class="t-body">Archief</span></div>',
            '<div class="sh t-cap">Plaatsen</div>',
            '<div class="sr"><svg class="ic"><use href="#i-device"/></svg><span class="t-body">Dit apparaat</span></div>']
    if full:
        rows.append('<div class="sr" style="height:48px;align-items:flex-start;padding-top:5px"><svg class="ic" style="margin-top:1px"><use href="#i-nas"/></svg><span><div class="t-body">Thuis-NAS</div><div class="t-cap tx2" style="display:flex;align-items:center;gap:4px"><svg class="ic" style="width:14px;height:14px"><use href="#i-check"/></svg>Gesynchroniseerd</div></span></div>')
        rows.append('<div class="sr" style="position:absolute;left:12px;right:12px;bottom:14px"><svg class="ic"><use href="#i-trash"/></svg><span class="t-body">Prullenbak</span></div>')
    else:
        rows.append('<div class="sr"><svg class="ic"><use href="#i-nas"/></svg><span class="t-body">Thuis-NAS</span></div>')
    return '<aside class="side">\n    ' + '\n    '.join(rows) + '\n   </aside>'


LIST = [('i-doc', 'Tuinplan', 'Tekst', 'vandaag <span class="num">09:40</span>', 'Dit apparaat + Thuis-NAS'),
        ('i-draw', 'Plattegrond', 'Tekening', 'gisteren <span class="num">18:12</span>', 'Dit apparaat + Thuis-NAS'),
        ('i-image', "Foto's border", '<span class="num">24</span> afbeeldingen', '<span class="num">10</span> nov.', 'Thuis-NAS'),
        ('i-pdf', 'Offerte schuur', 'PDF', '<span class="num">8</span> nov.', 'Dit apparaat'),
        ('i-list', 'Boodschappen', 'Lijst', '<span class="num">11</span> nov.', 'Dit apparaat')]


def rows(sel_first, hov_first=False):
    out = ['<div class="lh cols t-cap"><span>Naam</span><span>Soort</span><span>Gewijzigd</span><span>Opslag</span></div>']
    for n, (ic, nm, kind, when, where) in enumerate(LIST):
        if n == 0 and sel_first:
            out.append('<div class="lrw cols sel t-body" id="row3" aria-selected="true"><span class="n"><svg class="ic"><use href="#%s"/></svg><span class="t-bodys">%s</span></span><span>%s</span><span>%s</span><span>%s</span></div>' % (ic, nm, kind, when, where))
        else:
            st = ' style="background:var(--hov2)"' if (n == 0 and hov_first) else ''
            out.append('<div class="lrw cols t-body"%s><span class="n"><svg class="ic"><use href="#%s"/></svg>%s</span><span>%s</span><span>%s</span><span>%s</span></div>' % (st, ic, nm, kind, when, where))
    return out


# ---------- kern ----------
def kern(theme, kid, style, full=True):
    bond = ('<div class="kbond">'
            '<div class="kb-row"><span class="kb-node"><svg class="ic"><use href="#i-web"/></svg></span><span class="kb-lbl t-body">App</span><span class="kb-val t-body">Webvenster · geïnstalleerd 12 nov.</span></div>'
            '<div class="kb-row"><span class="kb-node"><svg class="ic"><use href="#i-notities"/></svg></span><span class="kb-lbl t-body">Wat</span><span class="kb-val t-body">Tuinplan, alleen dit document</span></div>'
            '<div class="kb-row p-net"><span class="kb-node"><svg class="ic"><use href="#i-internet"/></svg></span><span class="kb-lbl t-body">Waarheen</span><span class="kb-val t-bodys">tuinplanner.example</span></div>'
            '</div>')
    rest = ''
    if full:
        rest = '<div class="kb-dur"><svg class="ic"><use href="#i-klok"/></svg><span class="kb-lbl t-body">Hoe lang</span><span class="kb-val t-body">Eenmalig</span></div>'
    if full is True:
        rest += ('<div class="kw"><svg class="ic"><use href="#i-warn"/></svg><span class="t-body">Dit programma kan de toegestane inhoud naar een internetdienst sturen.</span></div>'
                '<div style="display:flex;justify-content:flex-end;gap:12px;margin-top:18px"><button class="btn sec lg">Eenmalig toestaan</button><button class="btn pri lg fx">Annuleren</button></div>')
    return ('<div class="kern %s" id="%s" style="%s" role="alertdialog" aria-labelledby="%st">\n'
            '  <div class="kband"><svg viewBox="0 0 1000 1000" aria-hidden="true"><use href="#mk-c"/></svg><span class="t-body dn">Vraag van het systeem · boven alle apps</span></div>\n'
            '  <div class="kbody"%s>\n   <div class="t-title" id="%st">Webvenster wil Tuinplan naar internet sturen</div>\n   %s%s\n  </div>\n </div>'
            % (theme, kid, style, kid, '' if full is True else ' style="padding:14px 24px 12px"', kid, bond, rest))


# ---------- artboards ----------
HEAD = '''<header class="head">
 <div class="disc"><b>Static design board — not a working OS.</b> Inter 4.001 is a reversible lead selection (type-1.0); the Saira-derived wordmark is owner-redirectable. All sample data is fictional.</div>
 <div>
  <div class="nm"><svg viewBox="0 0 1000 1000" role="img" aria-label="BaryonOS"><use href="#mk-p"/></svg>
   <div><h1>Orbitaal v2</h1><div class="sub">Direction B · v2 · BaryonOS 2030 · kinetic, atmospheric, spatial</div></div></div>
 </div>
 <p>BaryonOS as a quiet piece of physics: layered atmospheric depth, lit by one rim light from the upper left along the mark's −7° lean. <b>Colour now means place only</b> — green for this device, blue for the NAS, red for the internet — and every place or state mark pairs a shape, an icon and text. Pending the project owner's decision, the wallpapers, the marks and the boot keep the three charge colours as identity, not as state. Familiar paths stay intact: Orbit at the bottom centre, a launcher with search, controls on the right.</p>
 <p><b>Signature — light-blades.</b> One tapered, neutral blade marks focus, selection and the active item inside Orbit's top edge. Orbit carries a three-node privacy instrument; trusted questions are opaque, take the opposite theme's material, veil the apps and are tethered to the node that asks.</p>
</header>'''

CAP = {
    1: '<b>Bureaublad — dark.</b> v2: recent work on a calm frosted plate with its trajectory in the gutter; a three-node privacy instrument; one neutral blade marks the active item.',
    2: '<b>Menu + Zoeken — dark.</b> v2: neutral focus and selection, an Enter keycap on the focused row only, place pills with shape + icon + text; opaque fallback over windows.',
    3: '<b>Bestanden — dark.</b> v2: collection icons replace dots; the selected row is tethered to the inspector; versions, access and storage share one bond timeline.',
    4: '<b>Vensters + systeemvraag — light.</b> v2: a scrim veils all apps; the question uses the dark material with bonded rows and a red tether to the asking Internet node.',
    5: '<b>App-familie — light.</b> v2: categories with icons, access as a place map per app, third-party apps grouped and marked, the wordmark above Over BaryonOS.',
    6: '<b>Systeem.</b> v2: Inter roles, wordmark boot, 24 icons, a light question on dark, components with the shape legend, one token table for both themes, high contrast.',
}


def cap(n):
    return '<div class="cap">%s</div>' % CAP[n]


def ab1():
    return '''<section class="ab dk wall-dk" id="ab1">
 <svg class="wp" viewBox="0 0 1440 900" aria-hidden="true"><use href="#wp-dk"/></svg>
 %s
 %s
</section>''' % (continuity(), orbit([nuc('act'), ZOEK, SEP] + FIRST() + [SEP, tri('idle'), CLOCK]))


def ab2():
    lp = '<span class="page" style="width:34px;height:42px;border-radius:5px"><i style="left:6px;top:7px;width:15px;height:3px;background:#4A5470"></i><i style="left:6px;top:14px;width:22px;height:2px"></i><i style="left:6px;top:19px;width:20px;height:2px"></i><i style="left:6px;top:24px;width:22px;height:2px"></i></span>'
    return '''<section class="ab dk wall-dk" id="ab2">
 <svg class="wp" viewBox="0 0 1440 900" aria-hidden="true"><use href="#wp-dk"/></svg>
 <div style="position:absolute;inset:0;background:radial-gradient(900px 620px at 50%% 55%%,rgba(3,4,10,.55),rgba(3,4,10,.25))"></div>

 <div class="launch rim" style="left:290px;top:190px;width:860px;height:608px" role="dialog" aria-label="BaryonOS-menu">
  <div class="lbg"></div>
  <i class="bl" style="left:30px;width:420px"></i>
  <div class="lsearch">
   <svg class="ic" style="color:var(--tx2)"><use href="#i-zoeken"/></svg>
   <span class="t-title" aria-label="Zoek apps, bestanden en instellingen">tuin<span class="caret"></span></span>
  </div>
  <div class="lbody">
   <div class="lres">
    <div class="grp t-title">Bestanden</div>
    <div class="lr f selb" aria-selected="true">
     %s
     <span><div class="t-bodys"><mark>Tuin</mark>plan</div><div class="t-body tx2">Tekst · Huis · vandaag <span class="num">09:40</span></div></span>
     <kbd class="kc">Enter</kbd>
    </div>
    <div class="lr">
     <span class="page" style="width:34px;height:42px;border-radius:5px"><svg viewBox="0 0 48 60"><g fill="none" stroke="#7B869E" stroke-width="1.6"><rect x="7" y="9" width="34" height="42" rx="1"/><path d="M7 30h15V9M22 38v13M30 30h11"/></g></svg></span>
     <span><div class="t-bodys">Plattegrond</div><div class="t-body tx2">Tekening · Huis · bevat ‘tuin’</div></span>
    </div>
    <div class="lr">
     <span style="width:34px;height:42px;border-radius:5px;flex:none;display:grid;grid-template-columns:1fr 1fr;gap:2px;overflow:hidden;box-shadow:0 6px 14px rgba(0,0,0,.45)"><i style="background:linear-gradient(160deg,#7FB069,#2F5D3A)"></i><i style="background:linear-gradient(160deg,#B68FD8,#5A3E86)"></i><i style="background:linear-gradient(160deg,#CBB4E6,#7FA27A)"></i><i style="background:linear-gradient(160deg,#6E9B5A,#264430)"></i></span>
     <span><div class="t-bodys">Foto's border</div><div class="t-body tx2"><span class="num">24</span> afbeeldingen · Huis</div></span>
    </div>
    <div class="grp t-title" style="margin-top:6px">Acties</div>
    <div class="lr a"><svg class="ic s20" style="color:var(--tx2);margin:0 7px"><use href="#i-plus"/></svg><span class="t-body">Nieuw document ‘tuin’</span></div>
    <div class="lr a"><svg class="ic s20" style="color:var(--tx2);margin:0 7px"><use href="#i-bestanden"/></svg><span class="t-body">Zoek ‘tuin’ in Bestanden</span></div>
    <div class="grp t-title" style="margin-top:6px">Assistent</div>
    <div class="lr">
     <span class="tile s32" style="margin:0 1px"><svg class="ic"><use href="#i-assistent"/></svg></span>
     <span><div class="t-bodys">Vat Tuinplan samen</div><div class="t-body tx2">Draait lokaal · geen internet</div></span>
     %s
    </div>
   </div>
   <div style="padding:28px 28px 0 30px;position:relative">
    <div class="page" style="width:172px;height:218px;border-radius:12px;box-shadow:18px 26px 48px rgba(0,0,0,.55),inset 0 0 0 1px rgba(255,255,255,.7)">
     <div class="t-cap" style="position:absolute;left:16px;top:16px;color:#1A2032">Tuinplan 2027</div>
     <i style="left:16px;top:44px;width:136px"></i><i style="left:16px;top:52px;width:128px"></i><i style="left:16px;top:60px;width:138px"></i><i style="left:16px;top:68px;width:70px"></i>
     <i style="left:16px;top:84px;width:132px"></i><i style="left:16px;top:92px;width:138px"></i><i style="left:16px;top:100px;width:104px"></i>
     <i style="left:16px;top:120px;width:80px;height:46px;border-radius:4px;background:linear-gradient(160deg,#CFE6C6,#9CC79E)"></i><i style="left:102px;top:120px;width:52px;height:46px;border-radius:4px;background:linear-gradient(160deg,#E2D6F0,#B9A6D6)"></i>
     <i style="left:16px;top:178px;width:120px"></i><i style="left:16px;top:186px;width:96px"></i>
    </div>
    <div class="t-title" style="margin-top:26px">Tuinplan</div>
    <div class="t-body tx2">Tekst · Huis · vandaag <span class="num">09:40</span></div>
    <div style="display:flex;gap:8px;margin-top:16px;flex-wrap:wrap">
     %s
     %s
    </div>
   </div>
  </div>
  <div class="lfoot">
   <span class="ava" aria-hidden="true">N</span><span class="t-bodys">Noor</span>
   <span class="t-cap tx3" style="margin-left:auto;margin-right:8px">Enter opent · ↑↓ kiest · Esc sluit</span>
   <button class="btn qui lg"><svg class="ic"><use href="#i-lock"/></svg>Vergrendelen</button>
   <button class="btn qui lg"><svg class="ic"><use href="#i-power"/></svg>Aan/uit</button>
  </div>
 </div>

 %s
</section>''' % (lp, ptag('p-dev', 'st-dev', 'i-device', 'Op dit apparaat', style=' style="margin-left:auto"'),
                 pill('p-dev', 'st-dev', 'i-device', 'Op dit apparaat'), pill('p-nas', 'st-nas', 'i-nas', 'Thuis-NAS'),
                 orbit([nuc('open'), ZOEK, SEP] + FIRST() + [SEP, tri('idle'), CLOCK]))


def ab3():
    g = GEO
    return '''<section class="ab dk wall-dk" id="ab3">
 <svg class="wp" viewBox="0 0 1440 900" aria-hidden="true"><use href="#wp-dk"/></svg>
 <div class="win on" style="left:100px;top:52px;width:1240px;height:724px" role="dialog" aria-label="Huis — Bestanden">
  <i class="bl"></i>
  <div class="wb"><div class="fm">
   <div class="bar">
    <div class="bz"><span class="tile s28"><svg class="ic"><use href="#i-bestanden"/></svg></span><span class="t-bodys ttl">Huis — Bestanden</span></div>
    <button class="nb" aria-label="Terug"><svg class="ic"><use href="#i-back"/></svg></button>
    <button class="nb" aria-label="Vooruit" style="opacity:.45"><svg class="ic"><use href="#i-fwd"/></svg></button>
    <div class="crumb"><span class="t-body tx2">Collecties</span><span class="t-body tx3">›</span><span class="t-bodys">Huis</span></div>
    <div class="inp" style="margin-left:auto;width:250px;height:34px"><svg class="ic"><use href="#i-zoeken"/></svg><span class="t-body ph">Zoek in Huis</span></div>
    <button class="btn" style="background:var(--hov2);height:34px"><svg class="ic" style="width:16px;height:16px"><use href="#i-plus"/></svg>Nieuw</button>
    %s
   </div>
   %s
   <div class="pane">
    <div style="position:relative">
     %s
     <div class="sbar t-cap"><svg class="ic" style="width:14px;height:14px"><use href="#i-check"/></svg><span><span class="num">5</span> items · alles bijgewerkt</span></div>
    </div>
    <i class="itether" id="it3" aria-hidden="true" style="left:%gpx;top:%gpx;width:%gpx"></i>
    <div class="insp rim" id="insp3" style="--ty:%gpx">
     <div style="display:flex;gap:16px;align-items:center">
      <span class="page" style="width:52px;height:66px;border-radius:7px"><i style="top:10px;width:22px;height:4px;background:#4A5470"></i><i style="top:20px;width:34px"></i><i style="top:27px;width:32px"></i><i style="top:34px;width:34px"></i><i style="top:41px;width:22px"></i><i style="top:50px;width:30px"></i></span>
      <div><div class="t-title">Tuinplan</div><div class="t-body tx2">Tekst · <span class="num">1.284</span> woorden</div></div>
     </div>
     <div class="trio" id="trio3" style="--tb:%gpx">
      <div class="tsec"><div class="tsh"><span class="bring"><svg class="ic"><use href="#i-versies"/></svg></span><span class="t-bodys">Versies</span></div>
       <div class="tbody">
        <div class="vr now"><i></i><span class="t-bodys">Nu</span><span class="t-body num" style="margin-left:auto">09:40</span></div>
        <div class="vr"><i></i><span class="t-body">Versie 2</span><span class="t-body" style="margin-left:auto">gisteren <span class="num">18:12</span></span></div>
        <div class="vr"><i></i><span class="t-body">Versie 1</span><span class="t-body" style="margin-left:auto"><span class="num">11</span> nov. <span class="num">20:05</span></span></div>
        <button class="btn sec">Vergelijken en herstellen</button>
       </div></div>
      <div class="tsec"><div class="tsh"><span class="bring"><svg class="ic"><use href="#i-toegang"/></svg></span><span class="t-bodys">Toegang</span></div>
       <div class="tbody">
        <div class="acc"><span class="tile s28"><svg class="ic"><use href="#i-notities"/></svg></span><div class="tx"><span class="t-body">Notities mag lezen en wijzigen</span><span class="t-body tx2">Tot je het sluit</span></div></div>
        <button class="btn sec"><svg class="ic"><use href="#i-toegang"/></svg>Intrekken</button>
       </div></div>
      <div class="tsec" style="padding-bottom:0"><div class="tsh"><span class="bring"><svg class="ic"><use href="#i-device"/></svg></span><span class="t-bodys">Opslag</span></div>
       <div class="tbody">
        <div class="str p-dev"><svg class="st"><use href="#st-dev"/></svg><svg class="ic"><use href="#i-device"/></svg><span class="t-body">Dit apparaat</span></div>
        <div class="str p-nas"><svg class="st"><use href="#st-nas"/></svg><svg class="ic"><use href="#i-nas"/></svg><span><div class="t-body">Thuis-NAS</div><div class="t-cap tx2">Versleutelde kopie · <span style="white-space:nowrap">bijgewerkt <span class="num">09:41</span></span></div></span></div>
       </div></div>
     </div>
    </div>
   </div>
  </div></div>
 </div>

 %s
</section>''' % (WIN_WC.replace('<div class="wc">', '<div class="wc" style="margin-left:8px">'), side(True), '\n     '.join(rows(True)),
                 g['it3_left'], g['it3_top'], g['it3_w'], g['ty3'], g['tb3'],
                 orbit([nuc(), ZOEK, SEP, app('Bestanden', 'bestanden', 'act'), app('Notities', 'notities', 'run'), app('Instellingen', 'instellingen'), SEP, tri('idle'), CLOCK]))


def ab4():
    g = GEO
    return '''<section class="ab lt wall-lt" id="ab4">
 <svg class="wp" viewBox="0 0 1440 900" aria-hidden="true"><use href="#wp-lt"/></svg>

 <div class="win off" style="left:56px;top:40px;width:900px;height:600px" aria-label="Huis — Bestanden">
  <div class="wb"><div class="fm">
   <div class="bar">
    <div class="bz"><span class="tile s28"><svg class="ic"><use href="#i-bestanden"/></svg></span><span class="t-bodys ttl">Huis — Bestanden</span></div>
    <button class="nb" aria-label="Terug"><svg class="ic"><use href="#i-back"/></svg></button>
    <div class="crumb"><span class="t-body tx3">Collecties</span><span class="t-body tx3">›</span><span class="t-body tx3">Huis</span></div>
    %s
   </div>
   %s
   <div class="pane" style="grid-template-columns:1fr">
    <div>
     %s
    </div>
   </div>
  </div></div>
 </div>

 <div class="win on" style="left:404px;top:80px;width:916px;height:640px" role="dialog" aria-label="Tuinplan — Notities">
  <i class="bl"></i>
  <div class="wb"><div class="nt">
   <div class="bar">
    <span class="tile s28"><svg class="ic"><use href="#i-notities"/></svg></span><span class="t-bodys ttl">Tuinplan — Notities</span>
    %s
    %s
   </div>
   <div class="nbody">
    <div class="t-title">Tuinplan 2027</div>
    <p class="t-read">Voorjaar: de border langs de schutting opnieuw indelen. Lavendel en salie vooraan, daarachter twee rijen vaste planten die tot in oktober bloeien.</p>
    <p class="t-read">Moestuin: vier bakken van 120 × 80 cm, tomaten op het zuiden. De regenton aansluiten op de afvoer van de schuur.<span class="tcaret"></span></p>
   </div>
  </div></div>
 </div>

 <div class="scrim" aria-hidden="true"></div>
 %s
 <i class="tether" id="teth4" aria-hidden="true" style="left:%gpx;top:%gpx;height:%gpx"></i>

 %s
</section>''' % (WIN_WC, side(False), '\n     '.join(rows(False, True)),
                 ptag('p-dev', 'st-dev', 'i-device', 'Opgeslagen op dit apparaat · versie 3', style=' style="margin-left:auto"'),
                 WIN_WC.replace('<div class="wc">', '<div class="wc" style="margin-left:14px">'),
                 kern('dk', 'k4', 'left:%gpx;top:%gpx;width:600px' % (g['k4_left'], g['k4_top'])),
                 g['teth4_left'], g['teth4_top'], g['teth4_h'],
                 orbit([nuc(), ZOEK, SEP, app('Bestanden', 'bestanden', 'run'), app('Notities', 'notities', 'act'), app('Instellingen', 'instellingen'), SEP3,
                        app('Webvenster', 'web', 'run', tp=True), SEP, tri('ask', 'net4'), CLOCK]))


def pm_cell(p, shape, icon, state, text):
    off = state == 'off'
    return ('<span class="pm-c %s%s" role="cell"><i class="node %s%s"></i><svg class="ic"><use href="#%s"/></svg><span class="t-body">%s</span></span>'
            % (p, ' no' if off else '', shape, ' off' if off else '', icon, text))


def ab5():
    heads = ''.join('<span class="pm-h %s" role="columnheader"><i class="node %s"></i><span class="lbl t-bodys"><svg class="ic"><use href="#%s"/></svg>%s</span></span>' % (p, s, ic, lab) for p, s, ic, lab in PLACES)
    # MF8c: one place map row per app (shape + icon + text per cell), then a caption line with the app's data scope or history and a quiet Beheren control
    TGL = '<span class="tgl" role="switch" aria-checked="true" aria-label="Assistent gebruiken"></span>'
    apps = [
        ('notities', 'Notities', '', [('on', 'lezen, wijzigen'), ('on', 'lezen, wijzigen'), ('off', 'geen')], 'Documenten die je opent · geen internet', ''),
        ('web', 'Webvenster', ' tp', [('off', 'geen'), ('off', 'geen'), ('on', 'vraagt elke keer')], 'App van derden · <span class="num">1</span> keer toegestaan vandaag', ''),
        ('image', "Foto's", '', [('on', 'alleen lezen'), ('on', 'alleen lezen'), ('off', 'geen')], 'Afbeeldingen in Huis', ''),
        ('assistent', 'Assistent', '', [('on', 'op dit apparaat'), ('off', 'geen'), ('off', 'geen')], 'Vat documenten samen die je opent', TGL),
    ]
    prow = []
    for ic, nm, tp, cells, scope, extra in apps:
        cs = ''.join(pm_cell(p, sh, pic, st, tx) for (p, sh, pic, _), (st, tx) in zip(PLACES, cells))
        prow.append('<div class="pm-g" role="rowgroup"><div class="pm-r" role="row"><span class="pm-app" role="rowheader"><span class="tile s32%s"><svg class="ic"><use href="#i-%s"/></svg></span><span class="t-bodys">%s</span></span>%s</div>'
                    '<div class="pm-sub" role="row"><span class="pm-subc" role="cell" aria-colspan="4"><span class="t-cap tx2 grow">%s</span>%s'
                    '<button class="pm-mg t-cap" aria-label="%s beheren">Beheren<svg class="ic"><use href="#i-chev"/></svg></button></span></div></div>'
                    % (tp, ic, nm, cs, scope, extra, nm))
    nav = [('i-device', 'Systeem'), ('i-desk', 'Apparaten'), ('i-internet', 'Netwerk en internet'), ('i-image', 'Weergave'), ('i-apps', 'Apps'), ('i-user', 'Gebruikers'),
           ('i-nas', 'Opslag en NAS'), ('i-klok', 'Tijd en taal'), ('i-a11y', 'Toegankelijkheid')]
    navh = ''.join('<div class="ni"><svg class="ic"><use href="#%s"/></svg><span class="t-body">%s</span></div>' % n for n in nav)
    navh += '<div class="ni sel" aria-current="page"><svg class="ic"><use href="#i-toegang"/></svg><span class="t-bodys">Privacy en beveiliging</span></div>'
    navh += '<div class="ni"><svg class="ic"><use href="#i-download"/></svg><span class="t-body">Updates</span></div>'
    return '''<section class="ab lt wall-lt" id="ab5">
 <svg class="wp" viewBox="0 0 1440 900" aria-hidden="true"><use href="#wp-lt"/></svg>

 <div class="win off" style="left:48px;top:34px;width:800px;height:640px" aria-label="Sterrenkaart — Haven">
  <div class="wb"><div class="hv">
   <div class="bar">
    <span class="tile s28"><svg class="ic"><use href="#i-haven"/></svg></span><span class="t-bodys ttl">Sterrenkaart — Haven</span>
    %s
   </div>
   <div class="hbody">
    <div class="appic">
     <svg viewBox="0 0 88 88" width="88" height="88" style="position:absolute;inset:0"><g stroke="#C9D8FF" stroke-opacity=".7" stroke-width="1.2" fill="none"><path d="M18 60 34 44 50 50 66 26"/><path d="M50 50 58 68"/></g><g fill="#FFFFFF"><circle cx="18" cy="60" r="2.2"/><circle cx="34" cy="44" r="2.6"/><circle cx="50" cy="50" r="2.2"/><circle cx="58" cy="68" r="1.8"/></g><circle cx="66" cy="26" r="4.2" fill="#FFE7A8"/><circle cx="66" cy="26" r="10" fill="#FFE7A8" opacity=".18"/><g fill="#fff" opacity=".5"><circle cx="24" cy="22" r=".9"/><circle cx="72" cy="56" r=".9"/><circle cx="40" cy="70" r=".8"/></g></svg>
    </div>
    <div class="t-title" style="margin-top:16px">Sterrenkaart</div>
    <div class="t-body tx2">Open Hemel Stichting</div>
    <div class="t-body tx2">Gratis · <span class="num">18 MB</span></div>
    <button class="btn pri lg" style="margin-top:16px;min-width:150px">Installeren</button>
    <div class="t-bodys" style="margin-top:28px">Wat deze app kan vragen</div>
    <div style="width:360px;margin-top:6px">
     <div class="kv"><svg class="ic"><use href="#i-loc"/></svg><span class="t-body">Locatie — alleen als je het toestaat, om de hemel boven jou te tonen</span></div>
     <div class="kv"><svg class="ic"><use href="#i-internet"/></svg><span class="t-body">Internet — sterrencatalogus bijwerken</span></div>
     <div class="kv"><svg class="ic"><use href="#i-nofiles"/></svg><span class="t-body">Geen toegang tot je bestanden</span></div>
    </div>
    <div class="card hnote"><svg class="ic s20"><use href="#i-lock"/></svg><span class="t-body tx2">Installeren geeft geen toegang. De app vraagt het pas wanneer dat nodig is.</span></div>
    <div class="shot" style="left:430px;top:24px;width:330px;height:200px"></div>
    <div class="shot" style="left:430px;top:236px;width:160px;height:110px;background:radial-gradient(70%% 70%% at 40%% 40%%,#2C4C9A,#0A1230 75%%)"></div>
   </div>
  </div></div>
 </div>

 <div class="win on" style="left:440px;top:96px;width:960px;height:690px" role="dialog" aria-label="Privacy en beveiliging — Instellingen">
  <i class="bl"></i>
  <div class="wb"><div class="stg">
   <div class="bar">
    <span class="tile s28"><svg class="ic"><use href="#i-instellingen"/></svg></span><span class="t-bodys ttl">Privacy en beveiliging — Instellingen</span>
    %s
   </div>
   <nav class="snav" aria-label="Categorieën">
    <div class="inp" style="margin:0 0 10px"><svg class="ic"><use href="#i-zoeken"/></svg><span class="t-body ph">Zoek in Instellingen</span></div>
    %s
    <div class="ni about" role="link" tabindex="0"><svg role="img" aria-label="Baryon OS" width="98" height="16"><use href="#wm-lt"/></svg><span class="t-body">Over BaryonOS</span></div>
   </nav>
   <div class="spane">
    <div class="t-title">Privacy en beveiliging</div>
    <div class="t-bodys sech">Toegang per app en per plaats</div>
    <div class="card pm" role="table" aria-label="Toegang per app en per plaats">
     <div class="pm-r hd" role="row"><span class="t-cap tx2" role="columnheader">App</span>%s</div>
     %s
    </div>
    <div class="t-bodys sech">Vergrendelen</div>
    <div class="card">
     <div class="srow" style="min-height:50px"><svg class="ic" style="color:var(--tx2)"><use href="#i-lock"/></svg><span class="grow t-body">Na 5 minuten</span><svg class="ic s18" style="color:var(--tx3)"><use href="#i-chev"/></svg></div>
     <div class="srow" style="min-height:50px"><svg class="ic" style="color:var(--tx2)"><use href="#i-toegang"/></svg><span class="grow t-body">Herstelcode bewaard op papier · 2 nov.</span><svg class="ic s18" style="color:var(--tx3)"><use href="#i-chev"/></svg></div>
    </div>
   </div>
  </div></div>
 </div>

 %s
</section>''' % (WIN_WC, WIN_WC, navh, heads, '\n     '.join(prow),
                 orbit([nuc(), ZOEK, SEP, app('Bestanden', 'bestanden', 'run'), app('Notities', 'notities', 'run'), app('Instellingen', 'instellingen', 'act'),
                        app('Haven', 'haven', 'run'), SEP3, app('Webvenster', 'web', '', tp=True), SEP, tri('cmp'), CLOCK]))


ICONS = [('zoeken', 'Zoeken'), ('bestanden', 'Bestanden'), ('notities', 'Notities'), ('instellingen', 'Instellingen'), ('haven', 'Haven'), ('device', 'Apparaat'),
         ('nas', 'NAS'), ('internet', 'Internet'), ('versies', 'Versies'), ('toegang', 'Toegang'), ('power', 'Aan/uit'), ('lock', 'Vergrendel'),
         ('assistent', 'Assistent'), ('coll', 'Collectie'), ('huis', 'Huis'), ('archief', 'Archief'), ('web', 'Webvenster'), ('klok', 'Tijd'),
         ('apps', 'Apps'), ('user', 'Gebruiker'), ('a11y', 'Toegankelijk'), ('warn', 'Waarschuwing'), ('image', 'Afbeelding'), ('download', 'Download')]

TOK = [('surface.base', '#070912', '#E4E9F2'), ('surface.chrome', '#0D1020', '#EDF0F7'), ('surface.window', '#12162A', '#FBFCFE'),
       ('surface.raised', '#1A1F38', '#E3E8F1'), ('surface.inset', '#080A16', '#FFFFFF'), ('line.hairline', '#232A47', '#D8DEE9'),
       ('line.boundary', '#5D6890', '#78819A'), ('text.primary', '#EEF1F8', '#111726'), ('text.secondary', '#A9B2CB', '#424B62'),
       ('text.tertiary', '#8C95B2', '#58627D'),
       ('focus.ring', '#EEF1F8', '#111726'), ('select.fill', ('rgba(220,230,245,.085)', '#DCE6F5'), ('rgba(18,24,33,.065)', '#121821')),
       ('blade', '#E6EBF5', '#2B3245'), ('place.device', '#42D27F', '#127A43'), ('place.nas', '#5AA2FF', '#1061D6'),
       ('place.internet', '#FF5C50', '#CC3328'), ('status.error', '#FF5C50', '#CC3328'), ('system.kern', '#0B0E1B', '#EEF1F5'),
       ('scrim', ('rgba(3,5,9,.55)', '#030509'), ('rgba(222,229,238,.40)', '#DEE5EE')), ('launch.opaque', '#0F1328', '#F4F6FB')]


def tok_cell(v):
    bg, tx = v if isinstance(v, tuple) else (v, v)
    return '<span><i style="background:%s"></i>%s</span>' % (bg, tx)


def tb(theme, focused, icon, title):
    if theme == 'dk':
        bg = '#0D1020;box-shadow:inset 0 0 0 1px %s' % ('#2B3352' if focused else '#20263F')
    else:
        bg = '%s;box-shadow:inset 0 0 0 1px %s' % (('#FBFCFE', '#C9D0DD') if focused else ('#EDF0F7', '#D8DEE9'))
    blade = '<i class="bl" style="left:14px;width:150px"></i>' if focused else ''
    wc = WIN_WC if focused else WIN_WC.replace('<button ', '<button style="color:var(--tx3)" ')
    return ('<div class="tb" style="background:%s;margin-top:4px">%s<span class="tile s28" style="width:24px;height:24px;border-radius:7px"><svg class="ic" style="width:16px;height:16px"><use href="#i-%s"/></svg></span><span class="t-bodys%s">%s</span>%s</div>'
            % (bg, blade, icon, '' if focused else ' tx3', title, wc))


def ab6():
    icons = ''.join('<div class="icc"><svg class="ic"><use href="#i-%s"/></svg><span class="lab">%s</span></div>' % ic for ic in ICONS)
    groups = []
    for part in (TOK[:10], TOK[10:]):
        groups.append('<div class="tt">' + ''.join('<span>%s</span>%s%s' % (n, tok_cell(d), tok_cell(l)) for n, d, l in part) + '</div>')
    hc_orbit = orbit([nuc('act'), ZOEK, SEP] + FIRST() + [SEP, tri('idle'), CLOCK])
    c1_win = ('<div class="win on dk" style="left:12px;top:8px;width:476px;height:420px" aria-hidden="true"><div class="wb"><div class="fm">'
              '<div class="bar"><div class="bz"><span class="tile s28"><svg class="ic"><use href="#i-bestanden"/></svg></span><span class="t-bodys ttl">Huis — Bestanden</span></div></div>'
              '<aside class="side"><div class="sr"><svg class="ic"><use href="#i-recent"/></svg><span class="t-body">Recent</span></div><div class="sr"><svg class="ic"><use href="#i-coll"/></svg><span class="t-body">Vandaag</span></div></aside>'
              '<div class="pane" style="grid-template-columns:1fr"><div>' + ''.join(rows(False)[:3]) + '</div></div></div></div></div>')
    B = lambda cls, label, extra='': '<button class="btn %s"%s>%s</button>' % (cls, extra, label)
    lrow = lambda cls, ic, nm: '<div class="row %s t-body"><svg class="ic s20"><use href="#i-%s"/></svg><span%s>%s</span></div>' % (cls, ic, ' class="t-bodys"' if 'sel' in cls else '', nm)
    matrix = ('<div class="cg">'
              '<span></span><span class="lab">default</span><span class="lab">hover</span><span class="lab">focus</span><span class="lab">disabled · error · selected</span>'
              '<span class="lab">Primary</span>' + B('pri', 'Openen') + B('pri h', 'Openen') + B('pri fx', 'Openen') + B('pri dis', 'Openen', ' disabled') +
              '<span class="lab">Secondary</span>' + B('sec', 'Beheren') + B('sec h', 'Beheren') + B('sec fx', 'Beheren') + B('sec dis', 'Beheren', ' disabled') +
              '<span class="lab">Quiet</span>' + B('qui', 'Sluiten') + B('qui h', 'Sluiten') + B('qui fx', 'Sluiten') + B('qui dis', 'Sluiten', ' disabled') +
              '<span class="lab">Input</span>'
              '<div class="inp"><svg class="ic"><use href="#i-zoeken"/></svg><span class="t-body ph">Zoek</span></div><span class="lab tx3">no hover</span>'
              '<div class="inp fx"><svg class="ic"><use href="#i-zoeken"/></svg><span class="t-body">tuin<span class="caret" style="height:18px;vertical-align:-4px"></span></span></div>'
              '<div class="inp err" role="group" aria-label="Ongeldige naam"><svg class="ic" style="color:var(--red)"><use href="#i-warn"/></svg><span class="t-body">tuin/</span></div>'
              '<span class="lab">List row</span>' + lrow('', 'pdf', 'Offerte') + lrow('h', 'list', 'Lijst') + lrow('fx', 'image', "Foto's") + lrow('sel', 'doc', 'Tuinplan') +
              '</div>')
    orbit_row = ('<div class="orow"><span class="lab" style="width:64px">Orbit</span>'
                 '<span class="ob" style="width:40px;height:40px" aria-hidden="true"><span class="nuc"><svg viewBox="0 0 1000 1000"><use href="#mk-c"/></svg></span></span>'
                 '<span class="ob" style="width:40px;height:40px" aria-hidden="true"><svg class="ic"><use href="#i-zoeken"/></svg></span>'
                 '<span class="tile"><svg class="ic"><use href="#i-bestanden"/></svg></span><span class="tile tp"><svg class="ic"><use href="#i-web"/></svg></span>'
                 '<span class="lab" style="margin-left:8px">system = bare glyph · app = tile · third party = inner outline</span></div>')
    hc_leg = ''.join('<span><i style="background:%s"></i>%s</span>' % kv for kv in
                     [('#000000', '#000000 surfaces'), ('#FFFFFF', '#FFFFFF text, 2 px lines'), ('#FFE34D', '#FFE34D focus'),
                      ('#7CF0A8', '#7CF0A8 device'), ('#8EC5FF', '#8EC5FF NAS'), ('#FF8A80', '#FF8A80 internet')])
    return """<section class="ab sys" id="ab6">

 <!-- A1 boot convergence (live) with the v2 wordmark below the mark -->
 <div class="blk" style="left:24px;top:16px;width:300px;height:266px">
  <div class="bh">Convergence boot<span>live · loops with a hold</span></div>
  <div class="plate d" style="height:238px;background:radial-gradient(220px 160px at 50%% 40%%,#1A2150,#0A0C1A 80%%)">
   %s
   <svg class="wm" role="img" aria-label="Baryon OS" width="122" height="20" style="position:absolute;left:89px;top:140px"><use href="#wm-dk"/></svg>
   <div style="position:absolute;right:14px;top:14px;display:flex;flex-direction:column;align-items:center;gap:4px">
    %s
    <span class="lab">loader</span>
   </div>
   <div style="position:absolute;left:12px;right:12px;bottom:8px;display:flex;justify-content:space-between">
    %s
   </div>
  </div>
 </div>

 <!-- A2 type -->
 <div class="blk" style="left:24px;top:290px;width:300px;height:222px">
  <div class="bh">Type<span>Inter 4.001 · cv05 cv08 ss01</span></div>
  <div class="ty">
   <span class="t-disp">Noor</span><span class="lab">Display 48/56 · 400</span>
   <span class="t-title">Tuinplan</span><span class="lab">Title 22/28 · 600</span>
   <span class="t-read">Voorjaar</span><span class="lab">Reading 16/26 · 400</span>
   <span class="t-bodys">Herstellen</span><span class="lab">Body strong 15/22 · 600</span>
   <span class="t-body">Illustratie</span><span class="lab">Body 15/22 · 400</span>
   <span class="t-cap">Gesynchroniseerd</span><span class="lab">Caption 13/18 · 400</span>
   <span class="t-bodys"><span class="num">09:41 · 18 MB</span></span><span class="lab">Tabular figures</span>
  </div>
 </div>

 <!-- A3 MF11 launcher material + G7 keycap -->
 <div class="blk dk" style="left:24px;top:520px;width:300px;height:156px">
  <div class="bh">Launcher material<span>blur · opaque</span></div>
  <div style="display:grid;grid-template-columns:140px 140px;column-gap:20px;align-items:start">
   <div class="cmpg"><div class="spec wall-dk" style="width:140px;height:56px"><div class="lbg" style="background:linear-gradient(180deg,rgba(20,24,46,.86),rgba(11,13,27,.92));backdrop-filter:blur(30px) saturate(1.4)"></div><div class="tx" style="top:8px"><div class="t-bodys">Tuinplan</div><div class="t-cap tx2">Tekst · Huis</div></div></div><span class="lab">Boven achtergrond: vervaagd</span>
    <span style="display:flex;align-items:center;gap:8px"><kbd class="kc" style="margin-left:0">Enter</kbd><span class="lab">focused row</span></span></div>
   <div class="cmpg"><div class="spec" style="width:140px;height:56px;background:repeating-linear-gradient(0deg,#FBFCFE 0 14px,#D8DEE9 14px 15px)"><div class="lbg" style="background:var(--launch-opaque)"></div><div class="tx" style="top:8px"><div class="t-bodys">Tuinplan</div><div class="t-cap tx2">Tekst · Huis</div></div></div><span class="lab">Boven vensters: ondoorzichtig, geen vervaging</span></div>
  </div>
 </div>

 <!-- A4 title area + depth -->
 <div class="blk" style="left:24px;top:684px;width:300px;height:212px">
  <div class="bh">Title area<span>dark shown · light on artboards 4–5</span></div>
  <div class="dk">%s%s</div>
  <div class="bh" style="margin-top:10px">Depth<span>material per level</span></div>
  <div class="dps6">
   <div><div class="dp wall-dk" style="box-shadow:inset 0 0 0 1px #2B3352"></div><div class="lab">L0<br>wall</div></div>
   <div><div class="dp" style="background:#12162A;box-shadow:8px 12px 20px rgba(0,0,0,.6),inset 1px 1px 0 rgba(226,233,255,.4),inset 0 0 0 1px #2B3352"></div><div class="lab">L1<br>window</div></div>
   <div><div class="dp" style="background:linear-gradient(180deg,rgba(40,48,90,.8),rgba(16,19,38,.9));box-shadow:10px 16px 24px rgba(0,0,0,.6),inset 1px 1px 0 rgba(226,233,255,.45)"></div><div class="lab">L2<br>menu</div></div>
   <div><div class="dp" style="border-radius:12px;background:linear-gradient(180deg,#252B4D,#0F1222);box-shadow:0 10px 22px rgba(0,0,0,.6),inset 0 0 0 1px rgba(200,212,255,.25)"></div><div class="lab">L3<br>Orbit</div></div>
   <div><div class="dp" style="background:#0B0E1B;box-shadow:0 12px 26px rgba(0,0,0,.6),inset 0 0 0 1.5px #A9B3C9"></div><div class="lab">L4 kern<br>on light</div></div>
   <div><div class="dp" style="background:#EEF1F5;box-shadow:0 12px 26px rgba(0,0,0,.6),inset 0 0 0 1.5px #FFFFFF"></div><div class="lab">L4 kern<br>on dark</div></div>
  </div>
 </div>

 <!-- B1 motion -->
 <div class="blk" style="left:340px;top:16px;width:500px;height:193px">
  <div class="bh">Motion<span>transform + opacity · orbital-out (.16,.84,.24,1)</span></div>
  <div class="mt" style="grid-template-columns:120px 92px 1fr;row-gap:3px">
   <b>Boot convergence</b><span><span class="num">1400</span> ms</span><span>orbits converge · bond · blades · sweep</span>
   <b>Focus blade</b><span><span class="num">260</span> ms</span><span>sweeps the top edge from the upper left</span>
   <b>Active blade</b><span><span class="num">320</span> ms</span><span>translateX to the active item</span>
   <b>Selection blade</b><span><span class="num">160</span> ms</span><span>grows from the row centre (scaleY)</span>
   <b>Menu</b><span><span class="num">220 / 140</span> ms</span><span>rises 12 px from the mark</span>
   <b>System question</b><span><span class="num">240</span> ms</span><span>rises from Orbit, no overshoot</span>
   <b>Ask halo</b><span><span class="num">2400</span> ms</span><span>Internet node only, while asking</span>
   <b>Reduced motion</b><span><span class="num">0</span> ms</span><span>nothing moves; boot rests on the mark</span>
  </div>
 </div>

 <!-- B2 icons -->
 <div class="blk" style="left:340px;top:217px;width:500px;height:218px">
  <div class="bh">Icons<span>24 grid · 1.6 stroke · one solid particle at a joint</span></div>
  <div class="icg">%s</div>
 </div>

 <!-- C1 trusted question on a dark artboard: light material, cropped below the duration row -->
 <div class="blk" style="left:340px;top:443px;width:500px;height:430px">
  <div class="bh">System question on dark · crop<span>light material · scrim</span></div>
  <div class="plate dk wall-dk" style="height:402px">
   <svg class="wp" viewBox="0 0 1440 900" aria-hidden="true" style="left:-640px;top:-180px"><use href="#wp-dk"/></svg>
   %s
   <div class="scrim" aria-hidden="true"></div>
   %s
  </div>
 </div>

 <!-- HC crop of artboard 1: greeting, plate and Orbit (source x 96-1300, y 176-884) -->
 <div class="blk" style="left:856px;top:16px;width:560px;height:205px">
  <div class="bh">High contrast<span>Bureaublad crop at 25 %% · 2 px boundaries, opaque plate, yellow focus</span></div>
  <div style="display:flex;gap:16px;align-items:flex-start">
   <div style="width:301px;height:177px;overflow:hidden;border-radius:12px;box-shadow:inset 0 0 0 1px #3A4366;position:relative;flex:none">
    <div class="dk hc" style="position:absolute;left:-24px;top:-44px;width:1440px;height:900px;transform:scale(.25);transform-origin:0 0">
     <svg class="wp" viewBox="0 0 1440 900" aria-hidden="true"><g transform="rotate(-7 926 435)" fill="none" stroke="#FFFFFF" stroke-opacity=".55" stroke-width="2"><ellipse cx="926" cy="435" rx="600" ry="205"/><ellipse cx="926" cy="435" rx="410" ry="136"/><ellipse cx="926" cy="435" rx="232" ry="76"/></g><circle cx="878" cy="198" r="9" fill="#FF8A80"/><circle cx="1296" cy="477" r="9" fill="#8EC5FF"/><circle cx="605" cy="630" r="9" fill="#7CF0A8"/></svg>
     %s
     %s
    </div>
   </div>
   <div class="tt hcl">%s</div>
  </div>
 </div>

 <!-- C2 components: state matrix, Orbit items, shape legend -->
 <div class="blk dk" style="left:856px;top:227px;width:560px;height:414px">
  <div class="bh">Components<span>dark shown; light uses the same geometry</span></div>
  <div class="plate d" style="padding:10px 14px">
   %s
   %s
   <div style="display:flex;gap:6px;margin-top:8px">%s%s%s%s</div>
   <div class="note" style="margin-top:6px">Status nooit alleen met kleur: volle stip = apparaat, ring met stip = NAS, open ring = internet, vierkant = fout; altijd met icoon en tekst.</div>
  </div>
 </div>

 <!-- C3 tokens -->
 <div class="blk" style="left:856px;top:647px;width:560px;height:250px">
  <div class="bh">Tokens<span>name · donker · licht · the kern takes the opposite theme's material</span></div>
  <div class="plate d" style="padding:10px 10px">
   <div class="tk">%s%s</div>
   <div class="lab" style="margin-top:4px">select.fill (8.5 / 6.5 %%) and scrim (55 / 40 %%) are alpha over the surface below.</div>
  </div>
 </div>
</section>""" % (boot_svg, loader_svg, stages, tb('dk', True, 'bestanden', 'Bestanden'), tb('dk', False, 'bestanden', 'Bestanden'),
                 icons, c1_win, kern('lt', 'k6', 'left:20px;top:70px;width:460px', full='dur'),
                 continuity(hc=True), hc_orbit, hc_leg,
                 matrix, orbit_row,
                 pill('p-dev', 'st-dev', 'i-device', 'Dit apparaat'), pill('p-nas', 'st-nas', 'i-nas', 'Thuis-NAS'),
                 pill('p-net', 'st-net', 'i-internet', 'Internet'), pill('', 'st-err', 'i-warn', 'Fout', err=True),
                 groups[0], groups[1])


# ---------- assemble ----------
head_html = v1[:one(v1, '<style>')]
head_html = rep(head_html, '<title>BaryonOS — Orbitaal</title>', '<title>BaryonOS — Orbitaal v2</title>')
body = '\n'.join([
    '<style>\n' + css + '</style>\n</head>\n<body>\n',
    sprite, '',
    '<!-- ======================= BOARD HEADER ======================= -->', HEAD, '',
    '<!-- ======================= 1 · BUREAUBLAD — DARK ======================= -->', cap(1), ab1(), '<div class="after"></div>', '',
    '<!-- ======================= 2 · MENU + ZOEKEN — DARK ======================= -->', cap(2), ab2(), '<div class="after"></div>', '',
    '<!-- ======================= 3 · BESTANDEN — DARK ======================= -->', cap(3), ab3(), '<div class="after"></div>', '',
    '<!-- ======================= 4 · VENSTERS + SYSTEEMVRAAG — LIGHT ======================= -->', cap(4), ab4(), '<div class="after"></div>', '',
    '<!-- ======================= 5 · APP-FAMILIE — LIGHT ======================= -->', cap(5), ab5(), '<div class="after"></div>', '',
    '<!-- ======================= 6 · SYSTEEM ======================= -->', cap(6), ab6(), '<div class="after"></div>', '',
    '</body>\n</html>\n'])
html = head_html + body

# ---------- self-checks ----------
for m in marks_v1 + wp_v1 + [boot_svg]:
    assert html.count(m) == 1, m[:60]
for p in wm_dk + wm_lt:
    assert html.count(p) == 1
assert '--app' not in html and '--priv-bg' not in html and 'Funnel' not in html and 'autofocus' not in html
assert html.count('href="#wm-dk"') == 1 and html.count('href="#wm-lt"') == 1
ids = re.findall(r'\sid="([^"]+)"', html)
dup = sorted({i for i in ids if ids.count(i) > 1})
assert not dup, dup
defined = set(ids)
missing = sorted({h for h in re.findall(r'href="#([^"]+)"', html) if h not in defined} | {u for u in re.findall(r'url\(#([^)]+)\)', html) if u not in defined})
assert not missing, missing
assert 'Je werk blijft op dit apparaat' not in html
for n in range(1, 7):
    assert len(re.sub('<[^>]+>', '', CAP[n])) <= 175, n
os.makedirs(os.path.dirname(OUT), exist_ok=True)
open(OUT, 'w', encoding='utf-8').write(html)
print('wrote', OUT, len(html), 'bytes')
