import os
from pathlib import Path
import sys
import unittest
from unittest.mock import Mock, patch
sys.path.insert(0, str(Path(__file__).resolve().parents[1]/'backend/engine'))
import parent_watch


class ParentWatchTests(unittest.TestCase):
    def test_developer_mode_does_not_start_monitor(self):
        with patch.object(parent_watch.threading, 'Thread') as thread:
            parent_watch.watch_parent(None)
            thread.assert_not_called()

    def test_reject_invalid_pid(self):
        for pid in (-1, os.getpid()):
            with self.assertRaises(ValueError): parent_watch.watch_parent(pid)

    def test_owner_death_closes_handle_and_exits_engine(self):
        kernel = Mock()
        kernel.OpenProcess.return_value = 123
        kernel.WaitForSingleObject.return_value = 0
        with patch.object(parent_watch.sys, 'platform', 'win32'), \
             patch.object(parent_watch.ctypes, 'WinDLL', return_value=kernel, create=True), \
             patch.object(parent_watch.threading, 'Thread') as thread, \
             patch.object(parent_watch.os, '_exit') as exit_engine:
            parent_watch.watch_parent(424242)
            kernel.OpenProcess.assert_called_once_with(0x00100000, False, 424242)
            thread.call_args.kwargs['target']()
            kernel.WaitForSingleObject.assert_called_once_with(123, 0xFFFFFFFF)
            kernel.CloseHandle.assert_called_once_with(123)
            exit_engine.assert_called_once_with(0)

    def test_missing_owner_fails_closed(self):
        kernel = Mock()
        kernel.OpenProcess.return_value = None
        with patch.object(parent_watch.sys, 'platform', 'win32'), \
             patch.object(parent_watch.ctypes, 'WinDLL', return_value=kernel, create=True), \
             patch.object(parent_watch.ctypes, 'get_last_error', return_value=87, create=True):
            with self.assertRaises(OSError): parent_watch.watch_parent(424242)
