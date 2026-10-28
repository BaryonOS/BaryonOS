"""Host-process adversarial probes; printed strings are not guest evidence."""
from pathlib import Path
import os
import sys
import unittest
sys.path.insert(0,str(Path(__file__).resolve().parents[1]/'tools'))
from run_m6b import BOOT, case_markers, execute, normal_success


class NativeHarness(unittest.TestCase):
    def child(self, output, *, hold=0, status=0, cut=None, timeout=2):
        source = ('import sys,time,os; sys.stdout.write('+repr(output)+'); '
                  'sys.stdout.flush(); time.sleep('+repr(hold)+'); os._exit('+repr(status)+')')
        return execute([sys.executable,'-u','-c',source],dict(os.environ),cut=cut,timeout=timeout)

    def test_normal_requires_exact_order_exit_and_no_failure(self):
        output='\n'.join(case_markers('normal'))+'\n'
        self.assertTrue(normal_success(33,output,False,'normal'))
        for bad in (output+'M6B:PASS\n', output+'M6B:FAKE:PASS\n',output+'M6A:PASS\n',
                    output.replace('BOOT:EBS_OK\n',''), output+'M6B:FAIL:BAD\n'):
            self.assertFalse(normal_success(33,bad,False,'normal'))
        self.assertFalse(normal_success(0,output,False,'normal'))
        self.assertFalse(normal_success(33,output,True,'normal'))

    def test_genuine_live_child_kill_has_distinct_raw_exit(self):
        output='\n'.join(BOOT+('M6B:CUT:1',))+'\n'
        rc,out,err,timed,crash=self.child(output,hold=5,cut=1)
        self.assertTrue(crash['passed'])
        self.assertTrue(crash['was_running_before_kill'])
        self.assertTrue(crash['kill_requested'])
        self.assertEqual(rc,crash['termination_code_expected'])
        self.assertFalse(timed)

    def test_uncertain_mutations_require_unique_injection_and_unknown_proof(self):
        for case, operation in (('write-error', 'WRITE'), ('flush-error', 'FLUSH')):
            output = '\n'.join(case_markers(case))+'\n'
            self.assertTrue(normal_success(33, output, False, case))
            for marker in (f'M6B:INJECT:{operation}_IOERR_SNAPSHOT', 'M6B:OUTCOME:UNKNOWN',
                           'M6B:NO_REPLAY:PASS', 'M6B:NO_STALE_READ:PASS'):
                self.assertFalse(normal_success(33, output.replace(marker+'\n', ''), False, case))
                self.assertFalse(normal_success(33, output+marker+'\n', False, case))
            self.assertFalse(normal_success(33, output.replace('OUTCOME:UNKNOWN','OUTCOME:OK'), False, case))
            self.assertFalse(normal_success(33, output.replace(operation+'_IOERR', 'READ_IOERR'), False, case))
            self.assertFalse(normal_success(35, output, False, case))
            self.assertFalse(normal_success(33, output, True, case))
            for ordinary in ('normal', 'recover', 'errors', 'full', 'corrupt'):
                good = '\n'.join(case_markers(ordinary))+'\n'
                self.assertFalse(normal_success(33, good+'M6B:OUTCOME:UNKNOWN\n', False, ordinary))

    def test_self_exit_after_marker_never_counts_as_external_crash(self):
        output='\n'.join(BOOT+('M6B:CUT:1',))+'\n'
        rc,out,err,timed,crash=self.child(output,status=33,cut=1)
        self.assertFalse(crash['passed'])
        self.assertFalse(crash['kill_requested'])

    def test_duplicate_wrong_cut_and_success_marker_cannot_pass_kill(self):
        base='\n'.join(BOOT)+'\n'
        for tail in ('M6B:CUT:1\nM6B:CUT:1\n','M6B:CUT:2\n','M6B:CUT:1\nM6B:PASS\n'):
            _,_,_,_,crash=self.child(base+tail,hold=0.15,cut=1)
            self.assertFalse(crash['passed'])

    def test_timeout_without_rendezvous_is_not_a_crash_test_success(self):
        _,_,_,timed,crash=self.child('\n'.join(BOOT)+'\n',hold=5,cut=1,timeout=0.2)
        self.assertTrue(timed)
        self.assertFalse(crash['passed'])
        self.assertFalse(crash['kill_requested'])


if __name__=='__main__': unittest.main()
