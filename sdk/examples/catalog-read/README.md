# Native example app: reading the catalogue

This example is an actual `no_std` user program for the existing Baryon OS
int80 ABI. It requests one LIST, then reads each listed document with GET
through the delegated catalogue endpoint. It passes on no write, block,
device or network permissions and uses no fictitious document contents.

The app uses the reusable [typed catalogue client](../../catalog-client/README.md):
the existing B01 client plus the native SDK transport adapter. It does not
implement a second wire client. GET returns a checked `Document` or explicit
`NotFound`; an unexpected deletion terminates the example task.

The app validates kernel sender, tag, canonical envelope, request ID, epoch,
reply format and revision/name/length from the catalogue snapshot. Unexpected
received capabilities are closed. A concurrently changed catalogue, rejected
request or missing reply ends the experiment with an error code. Only
pre-publication `EAGAIN` is retried within a fixed budget; a queued RPC is
never automatically replayed.

```sh
python3 -B sdk/examples/catalog-read/build.py
```

Default output is `build/sdk-catalog-read/x86_64-unknown-none/release/catalog-read`.
A custom `CARGO_TARGET_DIR` changes the destination. The builder uses the
app's own linker script, preserves W^X segments and starts no QEMU process.

## Startup precondition and evidence limits

The example uses only existing M6c startup fields of the private page at
`0x40010000`: role 1/2 at offset 0, service epoch at 32, receive endpoint at 56,
send endpoint at 64 and service PID at 176. An authorised bootstrap must
populate those fields. They do not let the app create capabilities. The
separate support file keeps this temporary fixture adapter visibly outside
the reusable SDK.

The existing M6c acceptance manifest expects its own probe. Replacing that
ELF with this program is therefore **not a supported startup command**.
An ordinary app launcher or explicit additional test manifest is still
required to start this standalone example as a new process. The SDK can be
linked separately into the existing M6c probe to test the actual IPC path.

Exit 0 means this app validated its requests; offsets 240/248 retain local
diagnostics (document count/total content bytes). These fields are neither
kernel-issued PASS markers nor durability evidence. A successful build is
compilation evidence only; no new native Materiaal user action or physical
hardware support is claimed.
