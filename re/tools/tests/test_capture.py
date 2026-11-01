import contextlib, io, json, os, shutil, sys, tempfile, unittest, zipfile

HERE = os.path.dirname(os.path.abspath(__file__))
FIX = os.path.join(HERE, 'fixtures')
sys.path.insert(0, os.path.dirname(HERE))
import capture  # noqa: E402


def fixture(*parts):
    return os.path.join(FIX, *parts)


def analyse(*paths, build=None):
    return capture.analyse(capture.load_inputs(list(paths)), build, capture.read_subsystems())


def run_main(*argv):
    out = io.StringIO()
    with contextlib.redirect_stdout(out):
        capture.main(list(argv))
    return out.getvalue()


class BindFormat(unittest.TestCase):
    def test_latest_pass_wins(self):
        ea = analyse(fixture('bind-ea.log'))['ea']
        b = ea['bindings']
        self.assertEqual((b['FeTile_Build_List']['result'], b['FeTile_Build_List']['pass']), ('resolved', 'window'))
        self.assertEqual((b['Vm_Set_Bool']['result'], b['Vm_Set_Bool']['pass']), ('resolved', 'stage:1'))
        self.assertEqual((b['Ne_Log_Thunk']['result'], b['Ne_Log_Thunk']['pass']), ('unresolved', 'stage:1'))
        self.assertEqual(ea['coverage'], {'resolved': 4, 'deferred': 0, 'unresolved': 2, 'total': 6, 'unreported': 0})
        self.assertEqual([s['pass'] for s in ea['summaries']], ['init', 'window', 'stage:1'])

    def test_hints_only_from_verified_pattern_matches(self):
        ea = analyse(fixture('bind-ea.log'))['ea']
        self.assertEqual(ea['hints'], {'FeHome_Add_CareerTile': {'match': '0x173d397'},
                                       'FeTile_Build_List': {'match': '0x173a1b2'},
                                       'Vm_Set_Bool': {'match': '0x9747f0'},
                                       'g_Vm_Registry': {'match': '0x1691fb0'}})
        steam = analyse(fixture('bind-steam.log'))['steam']
        self.assertEqual(sorted(steam['hints']), ['FeHome_Add_CareerTile', 'FeTile_Build_List',
                                                  'site_FeHome_AfterCareerCall'])
        self.assertEqual(sorted(steam['unverified']), ['FeHome_Handle_Command', 'Vm_Set_String'])

    def test_bind_line_outranks_a_later_legacy_line(self):
        steam = analyse(fixture('bind-steam.log'))['steam']
        rec = steam['bindings']['FeTile_Build_List']
        self.assertEqual((rec['source'], rec['target_only'], rec['match']), ('bind', False, '0x173df80'))
        self.assertNotIn('FeTile_Build_List', steam['target_only'])

    def test_worklist_carries_reason_and_site_bytes(self):
        steam = analyse(fixture('bind-steam.log'))['steam']
        work = {w['id']: w for w in steam['worklist']}
        self.assertEqual(work['Vm_Set_Bool']['reason'],
                         'signature did not match exactly once; hint "0x975db0" bytes differ')
        self.assertEqual(work['Vm_Set_Bool']['site_bytes'], '48 89 5C 24 08 57 48 83 EC 20 48 8B F9 E8 00 00')
        ea = analyse(fixture('bind-ea.log'))['ea']
        site = next(w for w in ea['worklist'] if w['id'] == 'site_FeHome_AfterCareerCall')
        self.assertEqual((site['site'], site['site_bytes']), ('0x173d3a6', 'E9 01 02 03 04 90 90 90 90 90 90 90'))
        self.assertEqual([w['subsystem'] for w in ea['worklist']], ['core', 'fe_home'])


class BuildInference(unittest.TestCase):
    def test_build_from_selected_set(self):
        self.assertEqual(list(analyse(fixture('ea-live.log'))), ['ea'])
        self.assertEqual(list(analyse(fixture('steam-session'))), ['steam'])

    def test_build_from_bind_lines_without_selection(self):
        with tempfile.TemporaryDirectory() as tmp:
            p = os.path.join(tmp, 'loader.log')
            with open(fixture('bind-ea.log')) as f:
                lines = [ln for ln in f if '[core] bindings' not in ln]
            with open(p, 'w') as f:
                f.writelines(lines)
            self.assertEqual(list(analyse(p)), ['ea'])

    def test_fallback_set_is_unknown_until_named(self):
        with tempfile.TemporaryDirectory() as tmp:
            p = os.path.join(tmp, 'loader.log')
            with open(fixture('steam-session', 'loader.log')) as f:
                text = f.read().replace('selected bindings.json by PE timestamp 1789118616',
                                        'none match PE timestamp 1789999999; using bindings.json')
            with open(p, 'w') as f:
                f.write(text)
            got = analyse(p)
            self.assertEqual(list(got), ['unknown'])
            self.assertEqual(got['unknown']['identity']['pe_timestamp'], 1789999999)
            self.assertEqual(list(analyse(p, build='steam')), ['steam'])

    def test_build_filter(self):
        self.assertEqual(analyse(fixture('ea-live.log'), build='steam'), {})


class LegacyLogs(unittest.TestCase):
    def test_target_only_is_never_a_hint(self):
        ea = analyse(fixture('ea-live.log'))['ea']
        self.assertEqual(ea['hints'], {})
        self.assertEqual(len(ea['target_only']), 16)
        self.assertEqual(ea['target_only']['FeTile_Build_List'], '0x1738350')
        self.assertEqual(ea['target_only']['site_Sess_AiVehicleRegisterCall'], '0x603b5d')
        self.assertEqual(ea['coverage'], {'resolved': 16, 'deferred': 0, 'unresolved': 0, 'total': 269,
                                          'unreported': 253})

    def test_anchor_diagnostic_failure_is_reported_beside_a_core_resolution(self):
        ea = analyse(fixture('ea-live.log'))['ea']
        self.assertEqual(ea['bindings']['site_FeHome_AfterCareerCall']['source'], 'core')
        diag = ea['diagnostics'][0]
        self.assertEqual((diag['id'], diag['pass'], diag['site_bytes']),
                         ('site_FeHome_AfterCareerCall', 'window', 'E9 01 02 03 04 90 90 90 90 90 90 90'))

    def test_legacy_unresolved_takes_site_bytes_from_the_ea_report(self):
        ea = analyse(fixture('ea-prev.log'))['ea']
        work = {w['id']: w for w in ea['worklist']}
        self.assertEqual(sorted(work), ['FeHome_Add_CareerTile', 'Ne_Log_Thunk', 'Vm_Set_Bool', 'Vm_Set_String',
                                        'g_Ne_LogGate', 'g_Vm_Registry', 'k_Gdb_StringXorKey'])
        self.assertEqual(work['Vm_Set_String']['reason'],
                         'pattern: target differs from the KB build; string anchor: target differs from the KB build')
        self.assertEqual(work['Vm_Set_String']['site'], '0x1691fca')
        self.assertTrue(work['Vm_Set_String']['site_bytes'])
        self.assertEqual(work['k_Gdb_StringXorKey']['reason'], 'target differs from the KB build')
        self.assertEqual(work['Ne_Log_Thunk']['source'], 'ea')
        self.assertEqual(work['Ne_Log_Thunk']['pass'], 'window')
        self.assertTrue(work['Ne_Log_Thunk']['reason'].startswith('code-still-encrypted'))

    def test_progress_notes_are_not_bindings(self):
        steam = analyse(fixture('steam-session'))['steam']
        self.assertNotIn('resolution', steam['bindings'])
        self.assertEqual(steam['coverage']['unresolved'], 0)


class Inputs(unittest.TestCase):
    def test_session_dir_reads_the_session_run_only(self):
        steam = analyse(fixture('steam-session'))['steam']
        self.assertEqual([r['start'] for r in steam['runs']], ['2026-10-22 12:29:17.011'])
        self.assertEqual(steam['coverage']['total'], 253)
        self.assertEqual(len(steam['target_only']), 16)

    def test_copies_of_one_run_merge_and_the_longer_wins(self):
        with tempfile.TemporaryDirectory() as tmp:
            with open(fixture('bind-ea.log')) as f:
                lines = f.readlines()
            os.makedirs(os.path.join(tmp, 'game-logs'))
            with open(os.path.join(tmp, 'loader.log'), 'w') as f:
                f.writelines(lines[:-3])
            with open(os.path.join(tmp, 'game-logs', 'loader.log'), 'w') as f:
                f.writelines(lines)
            ea = analyse(tmp)['ea']
            self.assertEqual(len(ea['runs']), 1)
            self.assertEqual(ea['bindings']['Vm_Set_Bool']['pass'], 'stage:1')

    def test_send_logs_zip(self):
        with tempfile.TemporaryDirectory() as tmp:
            z = os.path.join(tmp, 'swofty-logs-test.zip')
            with zipfile.ZipFile(z, 'w') as w:
                w.writestr('config.json', '{}')
                w.write(fixture('ea-live.log'), 'game/loader.log')
                for rel in ('loader.log', 'game-logs/loader.log', 'game-logs/loader.prev.log'):
                    w.write(fixture('steam-session', *rel.split('/')), f'sessions/20261022-012913/{rel}')
                with open(fixture('bind-steam.log')) as f:
                    tail = f.readlines()[4:]
                w.writestr('sessions/20261023-100000/loader.log',
                           '[swofty: kept the last 1.0 KB of 4.0 KB]\n' + ''.join(tail))
            got = analyse(z)
            self.assertEqual(sorted(got), ['ea', 'steam'])
            self.assertEqual(len(got['steam']['runs']), 2)
            self.assertTrue(got['steam']['runs'][1]['truncated'] or got['steam']['runs'][0]['truncated'])
            self.assertEqual(got['steam']['bindings']['FeTile_Build_List']['source'], 'bind')
            self.assertEqual(got['ea']['coverage']['total'], 269)

    def test_json_output(self):
        doc = json.loads(run_main(fixture('bind-ea.log'), '--json'))
        self.assertEqual(doc['builds']['ea']['coverage']['resolved'], 4)
        self.assertIn('worklist', doc['builds']['ea'])


class Write(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.mkdtemp()
        for name in ('steam.json', 'ea.json'):
            shutil.copy(fixture('builds', name), os.path.join(self.tmp, name))

    def tearDown(self):
        shutil.rmtree(self.tmp)

    def load(self, name):
        with open(os.path.join(self.tmp, name)) as f:
            return json.load(f)

    def test_write_merges_hints_only(self):
        before = self.load('steam.json')
        out = run_main(fixture('bind-steam.log'), '--write', '--builds-dir', self.tmp)
        after = self.load('steam.json')
        self.assertEqual(after['hints'], {'FeHome_Add_CareerTile': {'match': '0x1742fc7'},
                                          'FeTile_Build_List': {'match': '0x173df80'},
                                          'Ne_Log_Thunk': {'match': '0x6d4c80'},
                                          'site_FeHome_AfterCareerCall': {'match': '0x1742fc7'}})
        for key in ('build', 'overrides', 'unresolvable', 'schema'):
            self.assertEqual(after[key], before[key])
        self.assertIn('2 added, 1 changed', out)
        self.assertIn('-   "match": "0x173df00"', out)
        self.assertIn('+   "match": "0x173df80"', out)
        again = run_main(fixture('bind-steam.log'), '--write', '--builds-dir', self.tmp)
        self.assertIn('already up to date', again)

    def test_write_creates_the_hints_object(self):
        run_main(fixture('bind-ea.log'), '--write', '--builds-dir', self.tmp)
        after = self.load('ea.json')
        self.assertEqual(list(after), ['schema', 'build', 'overrides', 'unresolvable', 'hints'])
        self.assertEqual(len(after['hints']), 4)

    def test_write_refuses_another_exe(self):
        path = os.path.join(self.tmp, 'steam.json')
        doc = self.load('steam.json')
        doc['build']['pe_timestamp'] = 1790000000
        with open(path, 'w') as f:
            json.dump(doc, f, indent=1)
        with open(path) as f:
            before = f.read()
        with self.assertRaises(SystemExit) as e:
            run_main(fixture('bind-steam.log'), '--write', '--builds-dir', self.tmp)
        self.assertIn('pe_timestamp 1789118616 != 1790000000', str(e.exception.code))
        with open(path) as f:
            self.assertEqual(f.read(), before)

    def test_write_refuses_a_sha_mismatch(self):
        path = os.path.join(self.tmp, 'ea.json')
        doc = self.load('ea.json')
        doc['build']['sha256'] = 'ff' * 32
        with open(path, 'w') as f:
            json.dump(doc, f, indent=1)
        with self.assertRaises(SystemExit) as e:
            run_main(fixture('bind-ea.log'), '--write', '--builds-dir', self.tmp)
        self.assertIn('sha256 cc506879c1c834aa', str(e.exception.code))

    def test_write_refuses_a_log_without_identity(self):
        with tempfile.TemporaryDirectory() as tmp:
            p = os.path.join(tmp, 'loader.log')
            with open(fixture('bind-ea.log')) as f:
                lines = [ln for ln in f if '[bind]' in ln]
            with open(p, 'w') as f:
                f.writelines(lines)
            with self.assertRaises(SystemExit) as e:
                run_main(p, '--write', '--builds-dir', self.tmp)
            self.assertIn('no build identity', str(e.exception.code))
        self.assertNotIn('hints', self.load('ea.json'))

    def test_write_never_crosses_builds(self):
        with self.assertRaises(SystemExit):
            run_main(fixture('ea-live.log'), '--build', 'steam', '--write', '--builds-dir', self.tmp)
        with self.assertRaises(SystemExit):
            run_main(fixture('bind-ea.log'), fixture('bind-steam.log'), '--write', '--builds-dir', self.tmp)


if __name__ == '__main__':
    unittest.main()
