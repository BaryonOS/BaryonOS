# Tussentijds checkpoint, geen volledige groenclaim

De snapshot running-suite.json behoudt bewust `status: running` en `passed: false`. De opgeslagen groepen en vier geselecteerde guestrecords zijn werkelijk uitgevoerd; de volledige M6a-poort en daaropvolgende regressie/audit waren bij deze snapshot nog niet afgerond.

Dit bewaart tussentijdse voortgang bij een sessie-/tegoedonderbreking. Definitief bewijs komt afzonderlijk in de bovenliggende suite.json en guest-runs.json na volledige raw/hash/imagecontrole. Broncommit en fingerprint staan in ../candidate.json.
