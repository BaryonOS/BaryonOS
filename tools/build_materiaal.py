"""Build and snapshot the actual isolated desktop executables; never emulate UI."""
import sys

from common import ROOT, BUILD, TARGET, run_logged, sha256
from build_users import inspect_user
from run_fpu import private_path, publish, publish_json


def build_users(*, probe=False):
    private_path(BUILD)
    private_path(TARGET)
    output = private_path(BUILD / "users/materiaal")
    specifications = [
        ("desktop-init", "desktop-init", "NOS_DESKTOP_INIT_ELF"),
        ("keyring", "keyring-service", "NOS_KEYRING_SERVICE_ELF"),
        ("volume-docs", "volume-service", "NOS_VOLUME_SERVICE_ELF"),
        ("materiaal", "materiaal-app", "NOS_MATERIAAL_ELF"),
        ("compositor", "native-compositor", "NOS_COMPOSITOR_ELF"),
        ("input-service", "input-source", "NOS_INPUT_SOURCE_ELF"),
        ("input-service", "input-policy", "NOS_INPUT_POLICY_ELF"),
    ]
    if probe:
        specifications.append(("desktop-init", "desktop-controller", "NOS_DESKTOP_CONTROLLER_ELF"))
    # Check all destinations before the first external command. In particular,
    # run_logged writes its three files on both success and failure; a later
    # per-component check would already be too late for an earlier linked log.
    private_path(output / "build-report.json")
    for package, name, _variable in specifications:
        private_path(TARGET / ("materiaal-" + package))
        private_path(output / (name + ".elf"))
        for suffix in ("stdout.txt", "stderr.txt", "json"):
            private_path(BUILD / ("logs/build-materiaal-" + package + "." + suffix))
    output.mkdir(parents=True, exist_ok=True)
    built = set()
    applications = []
    extra_env = {}
    for package, name, variable in specifications:
        target = private_path(TARGET / ("materiaal-" + package))
        if package not in built:
            run_logged([sys.executable, "-B", str(ROOT / "sdk/examples" / package / "build.py")],
                       "build-materiaal-" + package,
                       extra_env={"CARGO_TARGET_DIR": str(target)})
            built.add(package)
        source = private_path(target / "x86_64-unknown-none/release" / name,
                              regular=True, cargo_output=True)
        destination = private_path(output / (name + ".elf"))
        publish(destination, source.read_bytes())
        applications.append({"name": name, "sha256": sha256(destination), "elf": inspect_user(destination)})
        extra_env[variable] = str(destination)
    report = {"passed": True, "native_runtime_performed": False,
              "probe_controller": probe, "apps": applications}
    publish_json(output / "build-report.json", report)
    return extra_env, report
