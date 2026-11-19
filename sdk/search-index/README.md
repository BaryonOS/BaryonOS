# Bounded search index

`nieuw-os-search-index` is an allocator-free `no_std`, `forbid(unsafe_code)`
host-testable library. It is temporarily a standalone workspace because the
shared SDK manifest and lockfile have another owner. SDK membership, native
integration and U05 acceptance remain open.

`Index<E, B, T>` holds at most E records **per domain**, including deleted-object
revision tombstones. B limits the combined bytes of name, collection labels,
metadata text and content per record, and query bytes. T limits whitespace tokens
per record/query; each token is bounded by B bytes. The catalogue currently has
two domains, with `object / 2 == domain`. `query::<R>` bounds returned identities.
Capacity failures are explicit; updates and rebuilds do not evict or partially
publish records. Tombstones consume space until a newer authenticated epoch.

Search uses AND across exact whitespace-delimited tokens, with ASCII case folding
and otherwise unchanged UTF-8. Unicode whitespace is recognized. There is no
Unicode normalization, non-ASCII case folding, stemming, fuzzy search, ranking or
substring matching. Field boundaries always separate tokens. Collections and
metadata are searchable text supplied by the adapter, not a new catalogue format.

Each index and its correlation handles must remain bound to one catalogue
service/owner lifetime; numeric epochs are not cross-service credentials.
The trusted adapter must derive domain and epoch from an actually authenticated
catalogue session, never app request bytes. `grant` accepts only strictly increasing
epochs; `revoke` clears that domain's records and invalidates existing handles,
preventing their replay. Querying requires currently valid handles for every
requested domain, at most one per domain. An empty grant list
returns no hits. A new grant clears only its domain's cache. Update and delete
require strictly increasing nonzero catalogue revisions; delete receives the
confirmed deletion revision. Results contain domain, object and revision, not
authority to open the document. Callers must reread through the authenticated
catalogue session and honor later revocation and revision changes.

`rebuild` consumes a complete, current catalogue snapshot for one granted domain.
It validates the whole replacement before publication, preserves intact revision
floors/tombstones, and refuses duplicate, foreign or stale records. The adapter
must independently establish snapshot freshness and completeness. To recover from
corruption it may replace corrupt cache state using that trusted snapshot; corrupt
cache revisions cannot serve as trusted freshness evidence. SHA-256 covers cached
identities, text and token offsets to detect accidental corruption before use.
This is not a MAC, hostile-memory protection, authentication or an authority grant.
No cache serialization, persistence, IPC or native enforcement is implemented.

Results are returned by value after all grants and all requested-domain records
have been validated. Errors never expose a prefix of matching results. Denied
domains are not inspected for query validation or matching. Previously returned
copies cannot be recalled; the adapter must discard them on session invalidation.

For example, establish `let grant = index.grant(session_domain, session_epoch)?`,
then pass a `Document` with catalogue `object`, `revision`, `name()` and `value()`
(as content), together with separately validated collection/metadata text to
`index.update(grant, document)?`. Search with
`index.query::<8>(&[grant], b"meeting")?`. These calls model trusted lifecycle
inputs and do not themselves authenticate a session.
