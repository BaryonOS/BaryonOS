//! Host tool for B03 (ADR-049): generate labelled signing keys, sign kernel
//! images into `NOSMAN01` manifests, pack a `NOSSLOT1` slot store and verify
//! bundles. Every byte comes from `update-core`, so what this tool writes is
//! exactly what the UEFI loader and the ring-3 update service check. Keys
//! under `tools/update-keys/TEST-*` are test anchors for the gate; release
//! signing is a separate offline procedure and never runs in CI.
#![forbid(unsafe_code)]
use std::{
    collections::BTreeMap,
    env, fs,
    io::Read,
    path::{Path, PathBuf},
    process::ExitCode,
};
use update_core::{
    image_sector, manifest_for, manifest_sector, verify_image, verify_manifest, Header, Manifest,
    SigningKeys, TrustAnchors, ED25519_PUBLIC, INBOX_IMAGE, INBOX_MANIFEST, MANIFEST_LEN,
    MAX_IMAGE_LEN, MLDSA65_PUBLIC, SECTOR, STORE_SECTORS,
};

type Options = BTreeMap<String, String>;

fn main() -> ExitCode {
    match run(env::args().skip(1).collect()) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("update-tool: {error}");
            ExitCode::FAILURE
        }
    }
}

const USAGE: &str = "usage:
  update-tool keygen --dir DIR --label LABEL
  update-tool sign --keys DIR --label LABEL --image FILE --version N [--min-version M] --out FILE
  update-tool verify --keys DIR --label LABEL --manifest FILE [--image FILE]
  update-tool verify-pinned --manifest FILE [--image FILE]
  update-tool pack --out FILE --active A|B [--generation N]
       [--slot-a-manifest FILE --slot-a-image FILE] [--slot-b-manifest FILE --slot-b-image FILE]
       [--inbox-manifest FILE --inbox-image FILE]
  update-tool header --active A|B [--generation N]
  update-tool inspect --manifest FILE";

fn run(args: Vec<String>) -> Result<(), String> {
    let Some(command) = args.first() else {
        return Err(USAGE.to_string());
    };
    let options = parse(&args[1..])?;
    match command.as_str() {
        "keygen" => keygen(&options),
        "sign" => sign(&options),
        "verify" => verify(&options),
        "verify-pinned" => verify_pinned(&options),
        "pack" => pack(&options),
        "header" => header(&options),
        "inspect" => inspect(&options),
        other => Err(format!("unknown command {other}\n{USAGE}")),
    }
}

fn parse(args: &[String]) -> Result<Options, String> {
    let mut options = Options::new();
    let mut index = 0;
    while index < args.len() {
        let key = args[index]
            .strip_prefix("--")
            .ok_or_else(|| format!("expected --option, found {}", args[index]))?;
        let value = args
            .get(index + 1)
            .ok_or_else(|| format!("--{key} needs a value"))?;
        if options.insert(key.to_string(), value.clone()).is_some() {
            return Err(format!("--{key} given twice"));
        }
        index += 2;
    }
    Ok(options)
}

fn required<'a>(options: &'a Options, key: &str) -> Result<&'a str, String> {
    options
        .get(key)
        .map(String::as_str)
        .ok_or_else(|| format!("missing --{key}\n{USAGE}"))
}

fn number(options: &Options, key: &str, default: Option<u64>) -> Result<u64, String> {
    match (options.get(key), default) {
        (Some(text), _) => text
            .parse::<u64>()
            .map_err(|_| format!("--{key} must be a decimal number")),
        (None, Some(value)) => Ok(value),
        (None, None) => Err(format!("missing --{key}")),
    }
}

fn slot(options: &Options) -> Result<u8, String> {
    match required(options, "active")? {
        "A" | "a" | "0" => Ok(0),
        "B" | "b" | "1" => Ok(1),
        other => Err(format!("--active must be A or B, found {other}")),
    }
}

fn read_exact(path: &Path, length: usize) -> Result<Vec<u8>, String> {
    let bytes = fs::read(path).map_err(|e| format!("{}: {e}", path.display()))?;
    if bytes.len() != length {
        return Err(format!(
            "{}: expected {length} bytes, found {}",
            path.display(),
            bytes.len()
        ));
    }
    Ok(bytes)
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

fn write_new(path: &Path, bytes: &[u8]) -> Result<(), String> {
    if path.exists() {
        return Err(format!("{} exists; refusing to overwrite", path.display()));
    }
    fs::write(path, bytes).map_err(|e| format!("{}: {e}", path.display()))
}

fn key_paths(dir: &Path, label: &str) -> [PathBuf; 5] {
    [
        dir.join(format!("{label}-ed25519.seed")),
        dir.join(format!("{label}-mldsa65.seed")),
        dir.join(format!("{label}-ed25519.pub")),
        dir.join(format!("{label}-mldsa65.pub")),
        dir.join(format!("{label}-anchors.json")),
    ]
}

fn keygen(options: &Options) -> Result<(), String> {
    let dir = PathBuf::from(required(options, "dir")?);
    let label = required(options, "label")?;
    if label.is_empty() || !label.chars().all(|c| c.is_ascii_alphanumeric() || c == '-') {
        return Err("--label must be ASCII letters, digits or dashes".to_string());
    }
    fs::create_dir_all(&dir).map_err(|e| format!("{}: {e}", dir.display()))?;
    let paths = key_paths(&dir, label);
    if let Some(existing) = paths.iter().find(|p| p.exists()) {
        return Err(format!(
            "{} exists; choose another label",
            existing.display()
        ));
    }
    let mut seeds = [0u8; 64];
    fs::File::open("/dev/urandom")
        .and_then(|mut source| source.read_exact(&mut seeds))
        .map_err(|e| format!("reading /dev/urandom: {e}"))?;
    let (ed_seed, pq_seed): (&[u8; 32], &[u8; 32]) = (
        seeds[..32].try_into().unwrap(),
        seeds[32..].try_into().unwrap(),
    );
    let keys = SigningKeys::from_seeds(ed_seed, pq_seed);
    let anchors = keys.anchors();
    write_new(&paths[0], ed_seed)?;
    write_new(&paths[1], pq_seed)?;
    write_new(&paths[2], &anchors.ed25519)?;
    write_new(&paths[3], &anchors.mldsa65)?;
    let json = format!(
        "{{\n  \"label\": \"{label}\",\n  \"purpose\": \"{purpose}\",\n  \"ed25519_public\": \"{ed}\",\n  \"ed25519_key_id\": \"{edid}\",\n  \"mldsa65_public_sha256\": \"{pqhash}\",\n  \"mldsa_key_id\": \"{pqid}\"\n}}\n",
        purpose = if label.starts_with("TEST") {
            "test anchors for the B03 gate; never a release key"
        } else {
            "signing anchors"
        },
        ed = hex(&anchors.ed25519),
        edid = hex(&anchors.ed25519_key_id()),
        pqhash = hex(&update_core::sha256(&anchors.mldsa65)),
        pqid = hex(&anchors.mldsa_key_id()),
    );
    write_new(&paths[4], json.as_bytes())?;
    println!("generated {label} keys in {}", dir.display());
    Ok(())
}

fn load_keys(options: &Options) -> Result<SigningKeys, String> {
    let dir = PathBuf::from(required(options, "keys")?);
    let label = required(options, "label")?;
    let paths = key_paths(&dir, label);
    let ed = read_exact(&paths[0], 32)?;
    let pq = read_exact(&paths[1], 32)?;
    let keys = SigningKeys::from_seeds(
        ed.as_slice().try_into().unwrap(),
        pq.as_slice().try_into().unwrap(),
    );
    // The published anchors must be the ones these seeds derive.
    let anchors = keys.anchors();
    if read_exact(&paths[2], ED25519_PUBLIC)? != anchors.ed25519
        || read_exact(&paths[3], MLDSA65_PUBLIC)? != anchors.mldsa65
    {
        return Err(format!("{label}: public anchors do not match the seeds"));
    }
    Ok(keys)
}

fn load_anchors(options: &Options) -> Result<TrustAnchors, String> {
    let dir = PathBuf::from(required(options, "keys")?);
    let label = required(options, "label")?;
    let paths = key_paths(&dir, label);
    let mut anchors = TrustAnchors {
        ed25519: [0; ED25519_PUBLIC],
        mldsa65: [0; MLDSA65_PUBLIC],
    };
    anchors
        .ed25519
        .copy_from_slice(&read_exact(&paths[2], ED25519_PUBLIC)?);
    anchors
        .mldsa65
        .copy_from_slice(&read_exact(&paths[3], MLDSA65_PUBLIC)?);
    Ok(anchors)
}

fn sign(options: &Options) -> Result<(), String> {
    let keys = load_keys(options)?;
    let image_path = PathBuf::from(required(options, "image")?);
    let image = fs::read(&image_path).map_err(|e| format!("{}: {e}", image_path.display()))?;
    let version = number(options, "version", None)?;
    let min_version = number(options, "min-version", Some(0))?;
    let anchors = keys.anchors();
    let manifest = manifest_for(&image, version, min_version, &anchors).map_err(|r| {
        format!(
            "manifest rejected: {r:?} (image {} bytes, max {MAX_IMAGE_LEN})",
            image.len()
        )
    })?;
    let bytes = keys
        .sign(&manifest)
        .map_err(|r| format!("signing failed: {r:?}"))?;
    // Self-check with the verifier path the loader uses.
    verify_manifest(&bytes, &anchors).map_err(|r| format!("self-check failed: {r:?}"))?;
    verify_image(&manifest, &image).map_err(|r| format!("self-check failed: {r:?}"))?;
    let out = PathBuf::from(required(options, "out")?);
    fs::write(&out, bytes).map_err(|e| format!("{}: {e}", out.display()))?;
    println!(
        "signed {} as version {version} (min {min_version}, {} bytes, sha256 {}) -> {}",
        image_path.display(),
        image.len(),
        hex(&manifest.image_sha256),
        out.display()
    );
    Ok(())
}

fn verify(options: &Options) -> Result<(), String> {
    let anchors = load_anchors(options)?;
    let manifest_path = PathBuf::from(required(options, "manifest")?);
    let bytes = read_exact(&manifest_path, MANIFEST_LEN)?;
    let manifest = verify_manifest(bytes.as_slice().try_into().unwrap(), &anchors)
        .map_err(|r| format!("REJECTED:{}", reject_name(r)))?;
    verify_bundle(options, manifest)
}

fn verify_pinned(options: &Options) -> Result<(), String> {
    let path = PathBuf::from(required(options, "manifest")?);
    let bytes = read_exact(&path, MANIFEST_LEN)?;
    let manifest = update_core::test_policy::verify_manifest(bytes.as_slice().try_into().unwrap())
        .map_err(|r| format!("REJECTED:{}", reject_name(r)))?;
    verify_bundle(options, manifest)
}

fn verify_bundle(options: &Options, manifest: Manifest) -> Result<(), String> {
    if let Some(image_path) = options.get("image") {
        let image = fs::read(image_path).map_err(|e| format!("{image_path}: {e}"))?;
        verify_image(&manifest, &image).map_err(|r| format!("REJECTED:{}", reject_name(r)))?;
    }
    println!(
        "OK version={} min_version={} image_length={}",
        manifest.version, manifest.min_version, manifest.image_length
    );
    Ok(())
}

pub fn reject_name(reject: update_core::Reject) -> &'static str {
    use update_core::Reject::*;
    match reject {
        Format => "FORMAT",
        UnknownKey => "UNKNOWN_KEY",
        Signature => "SIGNATURE",
        Digest => "DIGEST",
        Downgrade => "DOWNGRADE",
        Read => "READ",
        KeyRevoked => "KEY_REVOKED",
        KeyExpired => "KEY_EXPIRED",
        KeyNotYetValid => "KEY_NOT_YET_VALID",
        KeyPolicy => "KEY_POLICY",
    }
}

fn place(store: &mut [u8], sector: u32, bytes: &[u8]) {
    let start = sector as usize * SECTOR;
    store[start..start + bytes.len()].copy_from_slice(bytes);
}

fn pack(options: &Options) -> Result<(), String> {
    let header = Header {
        active: slot(options)?,
        generation: number(options, "generation", Some(1))?,
    };
    let mut store = vec![0u8; STORE_SECTORS as usize * SECTOR];
    place(&mut store, 0, &header.encode());
    for (name, manifest_at, image_at) in [
        ("slot-a", manifest_sector(0), image_sector(0)),
        ("slot-b", manifest_sector(1), image_sector(1)),
        ("inbox", INBOX_MANIFEST, INBOX_IMAGE),
    ] {
        let manifest_path = options.get(&format!("{name}-manifest"));
        let image_path = options.get(&format!("{name}-image"));
        match (manifest_path, image_path) {
            (None, None) => continue,
            (Some(manifest_path), Some(image_path)) => {
                let manifest_bytes = read_exact(Path::new(manifest_path), MANIFEST_LEN)?;
                let manifest = Manifest::decode(manifest_bytes.as_slice().try_into().unwrap())
                    .map_err(|r| format!("{name}: manifest {}", reject_name(r)))?;
                let image = fs::read(image_path).map_err(|e| format!("{image_path}: {e}"))?;
                verify_image(&manifest, &image)
                    .map_err(|r| format!("{name}: image {}", reject_name(r)))?;
                place(&mut store, manifest_at, &manifest_bytes);
                place(&mut store, image_at, &image);
            }
            _ => return Err(format!("--{name}-manifest and --{name}-image go together")),
        }
    }
    let out = PathBuf::from(required(options, "out")?);
    fs::write(&out, &store).map_err(|e| format!("{}: {e}", out.display()))?;
    println!(
        "packed {} sectors (active {}, generation {}) -> {}",
        STORE_SECTORS,
        if header.active == 0 { "A" } else { "B" },
        header.generation,
        out.display()
    );
    Ok(())
}

fn header(options: &Options) -> Result<(), String> {
    let header = Header {
        active: slot(options)?,
        generation: number(options, "generation", Some(1))?,
    };
    println!("{}", hex(&header.encode()));
    Ok(())
}

fn inspect(options: &Options) -> Result<(), String> {
    let path = PathBuf::from(required(options, "manifest")?);
    let bytes = read_exact(&path, MANIFEST_LEN)?;
    let manifest = Manifest::decode(bytes.as_slice().try_into().unwrap())
        .map_err(|r| format!("REJECTED:{}", reject_name(r)))?;
    println!(
        "target={} image_length={} version={} min_version={} image_sha256={} ed25519_key_id={} mldsa_key_id={}",
        manifest.target,
        manifest.image_length,
        manifest.version,
        manifest.min_version,
        hex(&manifest.image_sha256),
        hex(&manifest.ed25519_key_id),
        hex(&manifest.mldsa_key_id)
    );
    Ok(())
}
