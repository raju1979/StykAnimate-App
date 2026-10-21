"""Backend selection, fallback and cancellation without requiring a compiler."""
import ctypes
import os
from pathlib import Path
import tempfile
import unittest
from unittest import mock

import server
import native_scan
from test_fast_decode import frame_bytes
from work_control import cancellation


class Function:
    def __init__(self, result): self.result = result
    def __call__(self, *args): return self.result


def library(implementation=None, abi=1):
    value = type('Library', (), {})()
    value.styk_piv_scan_abi = Function(abi)
    value.styk_piv_scan_result_size = Function(ctypes.sizeof(native_scan.ScanResult))
    value.styk_piv_scan = Function(0)
    if implementation is not None:
        value.styk_piv_scan_implementation = Function(implementation)
    return value


class NativeLoadingTests(unittest.TestCase):
    def test_rust_preferred_and_invalid_rust_uses_cpp(self):
        for marker, abi, expected in [(2, 1, 'rust'), (9, 1, 'cpp'), (2, 99, 'cpp')]:
            with tempfile.TemporaryDirectory() as directory:
                root = Path(directory)
                for name in ['rust', 'cpp']:
                    (root / name).mkdir()
                    (root / name / native_scan.library_name()).write_bytes(b'test-library')
                def load(file):
                    return library(marker, abi) if 'rust' in Path(file).parts else library()
                with mock.patch.dict(os.environ, {}, clear=True), mock.patch.object(native_scan, '_attempted', False), \
                     mock.patch.object(native_scan, '_library', None), mock.patch.object(native_scan, '_status', {'available': False, 'reason': 'not loaded'}), \
                     mock.patch.object(native_scan, 'build_directory', side_effect=lambda **kw: root / kw.get('implementation', 'cpp')), \
                     mock.patch.object(native_scan.ctypes, 'CDLL', side_effect=load):
                    self.assertEqual(native_scan.status()['implementation'], expected)

    def test_missing_explicit_library_uses_python_instead_of_a_different_native_binary(self):
        with mock.patch.dict(os.environ, {'STYK_NATIVE_SCAN_LIBRARY': '/missing/not-a-library'}, clear=True), \
             mock.patch.object(native_scan, '_attempted', False), mock.patch.object(native_scan, '_library', None), \
             mock.patch.object(native_scan, '_status', {'available': False, 'reason': 'not loaded'}):
            self.assertIsNone(native_scan._load())
            self.assertFalse(native_scan.status()['available'])

    def test_cancelled_native_result_does_not_advance_reader_or_publish_output(self):
        class Cancelled(Exception): pass
        calls = 0
        def check():
            nonlocal calls
            calls += 1
            if calls == 3: raise Cancelled()
        class CompletingLibrary:
            def styk_piv_scan(self, *args):
                result = args[-1]._obj
                result.end_offset = len(args[0]); result.frame_count = 1
                args[-3][0] = 5
                return 0
        source = frame_bytes()
        reader = server.FastReader(source)
        figures = [{'figure_type_id': 1, 'segment_count': 1, 'segments': [{'segment_id': 1}]}]
        with mock.patch.object(native_scan, '_load', return_value=CompletingLibrary()):
            with self.assertRaises(Cancelled), cancellation(check):
                native_scan.scan_keyframes(reader, figures, 2, max_pose_bytes=100000)
        self.assertEqual(reader.pos, 0)


if __name__ == '__main__': unittest.main()
