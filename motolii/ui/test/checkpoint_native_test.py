"""Run against `scripts/motolii-ui.sh native`; uses only disposable documents."""
import ctypes
import json
import pathlib
import tempfile

root = pathlib.Path(__file__).resolve().parents[3]
lib = ctypes.CDLL(str(root / 'motolii/target/debug/libmotolii_ui.dylib'))
lib.motolii_probe_open.argtypes = [ctypes.c_char_p]
lib.motolii_probe_open.restype = ctypes.c_void_p
lib.motolii_probe_request.argtypes = [ctypes.c_void_p, ctypes.c_char_p]
lib.motolii_probe_request.restype = ctypes.c_char_p
lib.motolii_probe_close.argtypes = [ctypes.c_void_p]
ctx = lib.motolii_probe_open(b'')
assert ctx


def request(op, **args):
    reply = json.loads(lib.motolii_probe_request(ctx, json.dumps(dict(op=op, **args)).encode()))
    return reply.get('status', reply)


try:
    with tempfile.TemporaryDirectory() as folder:
        directory = pathlib.Path(folder)
        original = str(directory / 'project.rrd')
        copy = str(directory / 'copy.rrd')
        state = request('create', kind='rectangle')
        assert not state.get('error'), state.get('error')
        request('save', path=original)
        state = request('create', kind='text')
        before = (state['path'], state['dirty'], state['undo'], len(state['layers']))
        state = request('save', path=copy, copy=True)
        assert (state['path'], state['dirty'], state['undo'], len(state['layers'])) == before
        request('create', kind='rectangle')
        state = request('restoreCheckpoint', path=copy, source=original)
        assert not state.get('error'), state.get('error')
        assert len(state['layers']) == 2 and state['dirty'] and state['path'] == original and state['undo'] == 0
        for invalid in ['missing.rrd', 'corrupt.rrd']:
            if invalid.startswith('corrupt'):
                (directory / invalid).write_text('not an RRD')
            state = request('restoreCheckpoint', path=str(directory / invalid), source=original)
            assert state.get('error') and len(state['layers']) == 2
        # A later explicit save still targets the original document, not the immutable checkpoint.
        state = request('save', path=original)
        assert not state.get('error') and not state['dirty']
        print('Native checkpoint copy/restore, missing/corrupt rejection and explicit save: passed')
finally:
    lib.motolii_probe_close(ctx)
