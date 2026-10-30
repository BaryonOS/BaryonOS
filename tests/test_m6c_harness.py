"""Host-process adversarial probes; printed strings are not guest evidence."""
from pathlib import Path
import os
import sys
import unittest
sys.path.insert(0,str(Path(__file__).resolve().parents[1]/'tools'))
from run_m6c import BOOT, case_markers, execute, normal_success, unique_markers


class NativeHarness(unittest.TestCase):
    def test_rejected_corrupt_or_legacy_volume_never_reaches_ready(self):
        for case in ("corrupt", "legacy"):
            output = "\n".join(case_markers(case))+"\n"
            self.assertTrue(normal_success(33, output, False, case))
            self.assertFalse(normal_success(33, "M6C:READY\n"+output, False, case))

    def child(self, output, *, hold=0, status=0, cut=None, timeout=2):
        source = ('import sys,time,os; sys.stdout.write('+repr(output)+'); '
                  'sys.stdout.flush(); time.sleep('+repr(hold)+'); os._exit('+repr(status)+')')
        return execute([sys.executable,'-u','-c',source],dict(os.environ),cut=cut,timeout=timeout)

    def test_normal_requires_exact_order_exit_and_no_failure(self):
        output='\n'.join(case_markers('normal'))+'\n'
        self.assertTrue(normal_success(33,output,False,'normal'))
        for bad in (output+'M6C:PASS\n', output+'M6C:FAKE:PASS\n',output+'M6A:PASS\n',
                    output.replace('BOOT:EBS_OK\n',''), output+'M6C:FAIL:BAD\n'):
            self.assertFalse(normal_success(33,bad,False,'normal'))
        self.assertFalse(normal_success(0,output,False,'normal'))
        self.assertFalse(normal_success(33,output,True,'normal'))

    def test_broker_latency_diagnostic_is_tolerated_but_never_a_pass_marker(self):
        # The kernel prints the largest publish-to-completion latency after
        # M6C:SURVIVOR_PROGRESS:PASS. It is diagnostic only: the controlled marker
        # sequence must ignore it, and a PASS-shaped variant must still be rejected.
        output='\n'.join(case_markers('normal'))+'\n'
        diagnostic=output.replace('M6C:SURVIVOR_PROGRESS:PASS\n',
                                  'M6C:SURVIVOR_PROGRESS:PASS\nM6C:BROKER:MAX_COMPLETION_TICKS:3\n'
                                  'M6C:BROKER:TIMEOUTS:0\nM6C:BROKER:OUTSTANDING_TICKS:12\n')
        self.assertTrue(normal_success(33,diagnostic,False,'normal'))
        self.assertFalse(normal_success(33,diagnostic.replace('TICKS:3','TICKS:3:PASS'),False,'normal'))
        # Printed before a failure marker, the diagnostic never masks the failure.
        self.assertFalse(normal_success(33,diagnostic+'M6C:FAIL:USER_OR_PROTOCOL\n',False,'normal'))
        fault='\n'.join(case_markers('flush-error'))+'\n'
        self.assertTrue(unique_markers(fault.replace('M6C:SURVIVOR_PROGRESS:PASS\n',
                        'M6C:SURVIVOR_PROGRESS:PASS\nM6C:BROKER:MAX_COMPLETION_TICKS:27\n'),
                        case_markers('flush-error')))

    def test_genuine_live_child_kill_has_distinct_raw_exit(self):
        output='\n'.join(BOOT+('M6C:READY','M6C:CUT:1',))+'\n'
        rc,out,err,timed,crash=self.child(output,hold=5,cut=1)
        self.assertTrue(crash['passed'])
        self.assertTrue(crash['was_running_before_kill'])
        self.assertTrue(crash['kill_requested'])
        self.assertEqual(rc,crash['termination_code_expected'])
        self.assertFalse(timed)

    def test_self_exit_after_marker_never_counts_as_external_crash(self):
        output='\n'.join(BOOT+('M6C:READY','M6C:CUT:1',))+'\n'
        rc,out,err,timed,crash=self.child(output,status=33,cut=1)
        self.assertFalse(crash['passed'])
        self.assertFalse(crash['kill_requested'])

    def test_duplicate_wrong_cut_and_success_marker_cannot_pass_kill(self):
        base='\n'.join(BOOT+('M6C:READY',))+'\n'
        for tail in ('M6C:CUT:1\nM6C:CUT:1\n','M6C:CUT:2\n','M6C:CUT:1\nM6C:PASS\n'):
            _,_,_,_,crash=self.child(base+tail,hold=0.15,cut=1)
            self.assertFalse(crash['passed'])

    def test_timeout_without_rendezvous_is_not_a_crash_test_success(self):
        _,_,_,timed,crash=self.child('\n'.join(BOOT)+'\n',hold=5,cut=1,timeout=0.2)
        self.assertTrue(timed)
        self.assertFalse(crash['passed'])
        self.assertFalse(crash['kill_requested'])


if __name__=='__main__': unittest.main()
