"""Hold a Windows process handle so the engine cannot outlive its desktop owner."""
import ctypes
from ctypes import wintypes
import os
import sys
import threading


def watch_parent(parent_pid):
    if not parent_pid:
        return  # CLI/developer mode has no desktop owner.
    if parent_pid <= 0 or parent_pid == os.getpid():
        raise ValueError('Invalid desktop parent PID')
    if sys.platform != 'win32':
        raise RuntimeError('Desktop parent monitoring currently requires Windows')
    kernel = ctypes.WinDLL('kernel32', use_last_error=True)
    kernel.OpenProcess.argtypes = [wintypes.DWORD, wintypes.BOOL, wintypes.DWORD]
    kernel.OpenProcess.restype = wintypes.HANDLE
    kernel.WaitForSingleObject.argtypes = [wintypes.HANDLE, wintypes.DWORD]
    kernel.WaitForSingleObject.restype = wintypes.DWORD
    kernel.CloseHandle.argtypes = [wintypes.HANDLE]
    kernel.CloseHandle.restype = wintypes.BOOL
    handle = kernel.OpenProcess(0x00100000, False, parent_pid)  # SYNCHRONIZE only
    if not handle:
        # Fail closed if the owner already exited or cannot be monitored.
        raise OSError(ctypes.get_last_error(), 'Cannot monitor Notely desktop')

    def wait():
        try:
            status = kernel.WaitForSingleObject(handle, 0xFFFFFFFF)
        finally:
            kernel.CloseHandle(handle)
        # Normal closure is guarded by the desktop while recording. This path
        # covers crashes/forced termination: do not leave capture/GPU threads alive.
        os._exit(0 if status == 0 else 1)

    thread = threading.Thread(target=wait, name='notely-parent-watch', daemon=True)
    thread.start()
