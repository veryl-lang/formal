import copy
import importlib.util
from pathlib import Path
import unittest
import tempfile
import json

spec = importlib.util.spec_from_file_location('dashboard_build', Path(__file__).parents[1] / 'build.py')
build = importlib.util.module_from_spec(spec)
spec.loader.exec_module(build)


class Results(unittest.TestCase):
    def setUp(self):
        self.identity = dict(schema=1, formal='a'*40, upstream='b'*40,
                             locks={'Cargo.lock':'c'*40}, dirty=False, upstreamDirty=False,
                             ci=dict(run='12', attempt='2'))
        self.contracts = [dict(id='simulator/kani/check_contracts/add',name='add',target='simulator',
                               method='kani',harness='check_add',covers=['carry'],manifest='proof/Cargo.toml',adapter='check_contracts')]

    def bundle(self, records, **changes):
        identity = dict(self.identity, **changes)
        summary = 'ci_run_id=12\nci_run_attempt=2\ntarget=simulator\nmanifest=proof/Cargo.toml\nadapter=check_contracts\nstatus=failed\n'
        return dict(schema=1,jobStatus='failure',runs=[dict(identity=identity,summary=summary,
                        path='targets/simulator/kani/target/contract-logs/run-1',
                        records='\t'.join(build.HEADER)+'\n'+records)])

    def load(self, bundle, current=None, **kwargs):
        contracts = copy.deepcopy(self.contracts)
        runs = build.load_results(contracts, current or self.identity, [bundle], **kwargs)
        return contracts[0],runs

    def test_classified_positive_survives_later_mutation_failure(self):
        c, runs = self.load(self.bundle('positive\tadd\tcheck_add\t\trunning\t0\tunknown\t1\n'
                                       'positive\tadd\tcheck_add\t\tpassed\t120\t0\t1\n'
                                       'mutation\tadd\tcheck_add\twrong-carry\tfailed\t10\t101\t1\n'))
        self.assertEqual(c['execution'], 'passed')
        self.assertEqual(c['mutationCounts'], {'failed':1})
        self.assertEqual(runs[0]['jobStatus'], 'failure')

    def test_stale_locks_revisions_dirty_trees_and_missing_identity_never_pass(self):
        record='positive\tadd\tcheck_add\t\tpassed\t1\t0\t1\n'
        for changes in [dict(formal='x'),dict(upstream='x'),dict(locks={'Cargo.lock':'x'}),
                        dict(dirty=True),dict(upstreamDirty=True),dict(schema=0)]:
            with self.subTest(changes=changes):
                self.assertEqual(self.load(self.bundle(record,**changes))[0]['execution'],'not-run')
        self.assertEqual(self.load(self.bundle(record),dict(self.identity,dirty=True))[0]['execution'],'not-run')
        bundle=self.bundle(record);bundle['runs'][0]['identity']=None
        self.assertEqual(self.load(bundle)[0]['execution'],'not-run')

    def test_wrong_attempt_or_runner_identity_is_excluded(self):
        record='positive\tadd\tcheck_add\t\tpassed\t1\t0\t1\n'
        self.assertEqual(self.load(self.bundle(record),run='12',attempt='3')[0]['execution'],'not-run')
        for field, wrong in [('manifest','other/Cargo.toml'),('adapter','other'),('target','frontend'),('ci_run_attempt','1')]:
            bundle=self.bundle(record)
            before=dict(line.split('=',1) for line in bundle['runs'][0]['summary'].splitlines())[field]
            bundle['runs'][0]['summary']=bundle['runs'][0]['summary'].replace(field+'='+before,field+'='+wrong)
            self.assertEqual(self.load(bundle,run='12',attempt='2')[0]['execution'],'not-run')

    def test_truncated_and_interrupted_checks_remain_unfinished(self):
        bundle=self.bundle('positive\tadd\tcheck_add\t\trunning\t0\tunknown\t1\npositive\tadd\tcheck_add\t\tpassed')
        self.assertEqual(self.load(bundle)[0]['execution'],'running')
        self.assertEqual(self.load(self.bundle(''))[0]['execution'],'not-run')
        bundle=self.bundle('positive\tadd\tcheck_add\t\trunning\t0\tunknown\t1\n'
                           'positive\tadd\tcheck_add\t\tpassed\t1\t0\t1')
        self.assertEqual(self.load(bundle)[0]['execution'],'running')

    def test_required_covers_and_harness_must_match(self):
        for record in ['positive\tadd\tcheck_add\t\tpassed\t1\t0\t0\n',
                       'positive\tadd\tother\t\tpassed\t1\t0\t1\n']:
            self.assertEqual(self.load(self.bundle(record))[0]['execution'],'not-run')

    def test_failure_is_not_hidden_by_another_passing_run(self):
        contracts=copy.deepcopy(self.contracts)
        build.load_results(contracts,self.identity,[self.bundle('positive\tadd\tcheck_add\t\tfailed\t1\t1\t1\n'),
                                                   self.bundle('positive\tadd\tcheck_add\t\tpassed\t1\t0\t1\n')])
        self.assertEqual(contracts[0]['execution'],'failed')

    def test_mutation_only_does_not_prove_original(self):
        c,_=self.load(self.bundle('mutation\tadd\tcheck_add\twrong-carry\tpassed\t1\t101\t1\n'))
        self.assertEqual(c['execution'],'not-run')
        self.assertEqual(c['mutationCounts'],{'passed':1})


class ScopeMetadata(unittest.TestCase):
    def test_stale_missing_and_duplicate_assignments_are_rejected(self):
        with tempfile.TemporaryDirectory() as directory:
            root=Path(directory)
            path=root/'targets/analyzer/coverage.json'
            path.parent.mkdir(parents=True)
            catalog=[dict(id='a',name='a',method='kani',target='analyzer')]
            row=dict(id='arithmetic',contracts=dict(kani=['a']))
            with self.assertRaises(ValueError):
                build.families(root,copy.deepcopy(catalog))
            for rows in [[],[dict(row,contracts=dict(kani=['removed']))],[row,row]]:
                path.write_text(json.dumps(dict(families=rows)))
                with self.assertRaises(ValueError):
                    build.families(root,copy.deepcopy(catalog))
            path.write_text(json.dumps(dict(families=[row])))
            result=build.families(root,copy.deepcopy(catalog))
            self.assertEqual(result[0]['proofIds'],['a'])


class ScopeDocumentation(unittest.TestCase):
    def test_headers_track_source_and_missing_headers_fail(self):
        with tempfile.TemporaryDirectory() as directory:
            root=Path(directory)
            path=root/'targets/probe/harness.rs'
            path.parent.mkdir(parents=True)
            path.write_text('//! Arbitrary inputs at width 8.\n//! Exact alias only.\n//!\n//! No partial-overlap theorem.\n\nfn proof() {}\n')
            docs=[dict(path='targets/probe/harness.rs',label='Kani')]
            self.assertEqual(build.scope_docs(root,docs),[
                'Kani: Arbitrary inputs at width 8. Exact alias only.',
                'Kani: No partial-overlap theorem.'])
            path.write_text('//! Width 16; exact alias only.\nfn proof() {}\n')
            self.assertEqual(build.scope_docs(root,docs),['Kani: Width 16; exact alias only.'])
            path.write_text('fn proof() {}\n')
            with self.assertRaises(ValueError):
                build.scope_docs(root,docs)
            with self.assertRaises(ValueError):
                build.scope_docs(root,[dict(path='.local/notes.rs',label='Private')])


if __name__ == '__main__':
    unittest.main()
