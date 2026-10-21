"""Build the dependency-free Rust scanner for this host, retaining C++ fallback.

Requires Cargo/Rust 1.85+ and the platform linker (MSVC + Windows SDK on Windows).
No toolchains or global settings are installed or modified by this script.
"""
import argparse
import hashlib
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys

ROOT = Path(__file__).resolve().parents[1]
sys.path[:0] = [str(ROOT / 'backend'), str(ROOT / 'codecs')]
import native_scan


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--cargo', default=os.environ.get('CARGO', 'cargo'))
    args = parser.parse_args()
    cargo = shutil.which(args.cargo)
    if cargo is None:
        parser.error('Cargo was not found. Install Rust 1.85+ or set CARGO to its executable.')
    crate = ROOT / 'native/rust-frame-scan'
    target = ROOT / 'build/rust-frame-scan'
    environment = dict(os.environ)
    if sys.platform == 'win32':
        # Match the existing helper's standalone Windows runtime requirement.
        if 'CARGO_ENCODED_RUSTFLAGS' in environment:
            environment['CARGO_ENCODED_RUSTFLAGS'] += '\x1f-C\x1ftarget-feature=+crt-static'
        else:
            environment['RUSTFLAGS'] = environment.get('RUSTFLAGS', '') + ' -C target-feature=+crt-static'
    subprocess.run([cargo, 'build', '--release', '--locked', '--offline', '--manifest-path', str(crate / 'Cargo.toml'),
                    '--target-dir', str(target)], check=True, cwd=ROOT, env=environment)
    source_name = ('styk_piv_frame_scan.dll' if sys.platform == 'win32' else
                   'libstyk_piv_frame_scan.dylib' if sys.platform == 'darwin' else 'libstyk_piv_frame_scan.so')
    output = native_scan.build_directory(implementation='rust')
    output.mkdir(parents=True, exist_ok=True)
    library = output / native_scan.library_name()
    rustc = os.environ.get('RUSTC') or shutil.which('rustc') or str(Path(cargo).with_name('rustc.exe' if os.name == 'nt' else 'rustc'))
    sysroot = Path(subprocess.check_output([rustc, '--print', 'sysroot'], text=True).strip())
    notices = output / 'licenses' / 'rust'
    notices.mkdir(parents=True, exist_ok=True)
    for name in ['COPYRIGHT', 'LICENSE-MIT', 'LICENSE-APACHE']:
        shutil.copyfile(sysroot / 'share' / 'doc' / 'rust' / name, notices / name)
    # A temporary copy plus replacement preserves the previous usable build on failure.
    staging = output / (library.name + '.tmp')
    shutil.copyfile(target / 'release' / source_name, staging)
    os.replace(staging, library)
    digest = lambda file: hashlib.sha256(file.read_bytes()).hexdigest()
    sources = {file.relative_to(ROOT).as_posix(): digest(file)
               for file in [crate / 'Cargo.toml', crate / 'Cargo.lock', crate / 'src/lib.rs']}
    manifest = {'schemaVersion': 1, 'abiVersion': 1, 'implementation': 'rust',
                'platform': sys.platform, 'architecture': native_scan.architecture(),
                'source': {'path': 'native/rust-frame-scan/src/lib.rs', 'sha256': digest(crate / 'src/lib.rs')},
                'sources': sources,
                'licenses': {file.name: digest(file) for file in notices.iterdir() if file.is_file()},
                'library': {'path': library.relative_to(ROOT).as_posix(), 'sha256': digest(library), 'bytes': library.stat().st_size},
                'toolchain': {'cargo': subprocess.check_output([cargo, '--version'], text=True).strip(),
                              'runtime': 'static CRT' if sys.platform == 'win32' else 'system'}}
    (output / 'frame-scan-build-info.json').write_text(json.dumps(manifest, indent=2) + '\n', encoding='utf-8')
    print(json.dumps(manifest, indent=2))


if __name__ == '__main__':
    main()
