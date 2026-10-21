"""Matched PIV frame-validation diagnostic; excludes decompression, UI and GPU.

Synthetic fixtures contain ordinary poses, ordered and scrambled identities,
draw orders and long joins. Every backend must return identical durations/bounds.
Use --rust-library to include the optional Rust build in the comparison.
"""
import argparse
import cProfile
import hashlib
import json
import os
from pathlib import Path
import statistics
import struct
import subprocess
import sys
import time

ROOT = Path(__file__).resolve().parents[1]
sys.path[:0] = [str(ROOT / 'backend'), str(ROOT / 'codecs')]


def fixture(frames, objects, segments, joined=False, scrambled=False):
    metadata = [{'figure_type_id': 1, 'segment_count': segments,
                 'segments': [{'segment_id': i + 1} for i in range(segments)],
                 'image_assets': [], 'text_assets': []}]
    pieces = [struct.pack('<I', frames)]
    angles = struct.pack(f'<{segments}d', *[i / 10 for i in range(segments)])
    for frame in range(frames):
        pieces.append(struct.pack('<HBHH', 1 + frame % 4, 0, 0, objects))
        for obj in range(objects):
            uid = ((obj + 1) * 2654435761) & 0xffffffff if scrambled else obj + 1
            pieces.append(struct.pack('<IHfIB', uid, 1, 1., 0, 0))
            pieces.extend([angles, struct.pack('<ffHHB', float(obj), float(frame), 0, 0,
                                             int(joined and obj + 1 < objects))])
            if joined and obj + 1 < objects:
                pieces.append(struct.pack('<HH', obj + 1, segments))
        pieces.extend([struct.pack(f'<{objects}H', *range(objects)), b'\x01'])
    return b''.join(pieces), metadata


def worker(args):
    import server
    import fast_decode
    import native_scan
    from profile_import import memory_snapshot
    source = args.source.read_bytes()
    figures = json.loads(args.source.with_suffix('.json').read_text())
    if args.mode == 'python':
        os.environ['STYK_DISABLE_NATIVE_SCAN'] = '1'
    elif not native_scan.status()['available']:
        raise RuntimeError('Requested native scanner was not loaded')
    runs = []
    expected = None
    for _ in range(7):
        reader = server.FastReader(source)
        begin = time.perf_counter()
        result = fast_decode.scan_keyframes(reader, figures, 1)
        elapsed = (time.perf_counter() - begin) * 1000
        digest = hashlib.sha256(json.dumps([result, reader.pos], separators=(',', ':')).encode()).hexdigest()
        if expected is not None and digest != expected:
            raise AssertionError('Unstable scanner result')
        expected = digest
        runs.append(elapsed)
    if args.mode == 'python':
        profile = cProfile.Profile()
        profile.runcall(fast_decode.scan_keyframes, server.FastReader(source), figures, 1)
        profile.dump_stats(str(args.source.with_suffix('.prof')))
    print(json.dumps({'mode': args.mode, 'median_ms': statistics.median(runs[1:]),
                      'runs_ms': runs, 'sha256': expected, 'memory': memory_snapshot(),
                      'native': native_scan.status()}))


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--rust-library', type=Path)
    parser.add_argument('--output', type=Path, default=ROOT / 'artifacts/rust-scan')
    parser.add_argument('--mode', choices=['python', 'cpp', 'rust'])
    parser.add_argument('--source', type=Path)
    args = parser.parse_args()
    if args.mode:
        worker(args)
        return
    args.output.mkdir(parents=True, exist_ok=True)
    report = {'scope': 'Frame validation plus ctypes metadata/output boundary; not application import time.', 'fixtures': []}
    for name, frames, objects, segments, joined, scrambled in [('small', 60, 20, 10, False, False),
            ('dense', 120, 1000, 32, False, False), ('joined', 60, 1500, 24, True, False),
            ('scrambled', 120, 1000, 32, False, True)]:
        source, figures = fixture(frames, objects, segments, joined, scrambled)
        file = args.output / f'{name}.bin'
        file.write_bytes(source)
        file.with_suffix('.json').write_text(json.dumps(figures))
        rows = []
        for mode in ['python', 'cpp', *(['rust'] if args.rust_library else [])]:
            env = dict(os.environ)
            env.pop('STYK_DISABLE_NATIVE_SCAN', None)
            env.pop('STYK_NATIVE_SCAN_LIBRARY', None)
            if mode == 'rust':
                env['STYK_NATIVE_SCAN_LIBRARY'] = str(args.rust_library.resolve())
            # Explicitly measure the existing C++ library even when Rust is installed.
            if mode == 'cpp':
                sys.path.insert(0, str(ROOT / 'backend'))
                import native_scan
                env['STYK_NATIVE_SCAN_LIBRARY'] = str(native_scan.build_directory() / native_scan.library_name())
            output = subprocess.check_output([sys.executable, __file__, '--mode', mode, '--source', str(file)], env=env, text=True)
            row = json.loads(output)
            rows.append(row)
            print(f'{name}: {mode} {row["median_ms"]:.3f} ms', flush=True)
        if len({row['sha256'] for row in rows}) != 1:
            raise AssertionError('Scanner outputs differ')
        report['fixtures'].append({'name': name, 'frames': frames, 'objects_per_frame': objects,
                                   'segments': segments, 'bytes': len(source), 'runs': rows})
    (args.output / 'report.json').write_text(json.dumps(report, indent=2) + '\n')


if __name__ == '__main__':
    main()
