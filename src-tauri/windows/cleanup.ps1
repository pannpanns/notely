param([Parameter(Mandatory=$true)][string]$InstallDir)
$ErrorActionPreference = 'Stop'
# NSIS may be 32-bit even when it installs an x64 app. Get-Process.Path is
# null for x64 targets in x86 PowerShell. Relaunch before inspecting processes.
if ([Environment]::Is64BitOperatingSystem -and -not [Environment]::Is64BitProcess) {
    $nativeShell = Join-Path $env:WINDIR 'Sysnative/WindowsPowerShell/v1.0/powershell.exe'
    if (-not (Test-Path -LiteralPath $nativeShell)) {
        Write-Output 'Cannot locate native 64-bit PowerShell for Notely cleanup.'
        exit 1
    }
    & $nativeShell -NoProfile -NonInteractive -ExecutionPolicy Bypass -File $PSCommandPath -InstallDir $InstallDir
    exit $LASTEXITCODE
}
$logPath = Join-Path $env:TEMP 'Notely-cleanup.log'
function Write-CleanupLog([string]$Message) {
    $line = "$(Get-Date -Format o) $Message"
    Write-Output $line
    Add-Content -LiteralPath $logPath -Value $line -Encoding UTF8 -ErrorAction SilentlyContinue
}
Write-CleanupLog "Cleanup started: x64=$([Environment]::Is64BitProcess); directory=$InstallDir"
try {
    $root = [IO.Path]::GetFullPath($InstallDir)
    $desktopPath = [IO.Path]::Combine($root, 'notely-desktop.exe')
    $enginePath = [IO.Path]::Combine($root, 'notely-engine.exe')
    function Matching-Processes([string]$Name, [string]$ExpectedPath) {
        foreach ($process in @(Get-Process -Name $Name -ErrorAction SilentlyContinue)) {
            if ($process.HasExited) { continue }
            $processPath = $process.Path
            if ([string]::IsNullOrWhiteSpace($processPath)) {
                $process.Refresh()
                if ($process.HasExited) { continue }
                throw "Cannot determine executable path for $Name PID $($process.Id); cleanup cancelled."
            }
            if ([string]::Equals($processPath, $ExpectedPath, [StringComparison]::OrdinalIgnoreCase)) {
                $process
            }
        }
    }
    # Ask each matching engine to stop capture and persist partial transcription.
    # The token stays in memory and is never written to the cleanup log.
    foreach ($engine in @(Matching-Processes 'notely-engine' $enginePath)) {
        $info = Get-CimInstance Win32_Process -Filter "ProcessId = $($engine.Id)"
        $commandLine = $info.CommandLine
        if ($commandLine -match '--port\s+"?(\d+)"?') { $port = $Matches[1] } else { continue }
        if ($commandLine -match '--token\s+"?([A-Za-z0-9_-]+)"?') { $token = $Matches[1] } else { continue }
        $base = "http://127.0.0.1:$port"
        $headers = @{Authorization="Bearer $token"}
        try { $status = Invoke-RestMethod "$base/status" -Headers $headers -TimeoutSec 5 }
        catch {
            # A crashed/unresponsive engine cannot complete its normal shutdown.
            Write-CleanupLog 'Engine did not respond; closing matching processes.'
            continue
        }
        if ($status.state -eq 'recording') {
            Write-CleanupLog 'Stopping capture and saving recording automatically.'
            $null = Invoke-RestMethod "$base/stop" -Method Post -Headers $headers -ContentType 'application/json' -Body '{}' -TimeoutSec 10
        }
        $deadline = [DateTime]::UtcNow.AddSeconds(120)
        $partialRequested = $false
        do {
            $status = Invoke-RestMethod "$base/status" -Headers $headers -TimeoutSec 5
            if ($status.state -eq 'idle') { break }
            if ($status.state -eq 'stopping' -and -not $partialRequested) {
                try {
                    $null = Invoke-RestMethod "$base/finish-partial" -Method Post -Headers $headers -ContentType 'application/json' -Body '{}' -TimeoutSec 10
                    $partialRequested = $true
                } catch { } # Capture may still be flushing. Retry on the next poll.
            }
            Start-Sleep -Milliseconds 250
        } while ([DateTime]::UtcNow -lt $deadline)
        if ($status.state -ne 'idle') { throw 'Timed out saving active audio. Files were preserved. Retry uninstall after processing completes.' }
    }
    foreach ($desktop in @(Matching-Processes 'notely-desktop' $desktopPath)) {
        try {
            if (-not $desktop.HasExited) {
                Write-CleanupLog "Closing Notely PID $($desktop.Id) automatically."
                $null = $desktop.CloseMainWindow()
                if (-not $desktop.WaitForExit(5000)) {
                    # Capture has already been saved above. Bypass a stale UI guard.
                    $desktop.Kill()
                    if (-not $desktop.WaitForExit(15000)) { throw 'Desktop exit timed out.' }
                }
            }
        } catch { if (-not $desktop.HasExited) { throw } }
    }
    # Both processes in a legacy onefile build use the same executable path.
    # Match the installation path, never kill all Python/Notely processes by name.
    foreach ($engine in @(Matching-Processes 'notely-engine' $enginePath)) {
        try {
            if (-not $engine.HasExited) {
                Write-CleanupLog "Stopping engine PID $($engine.Id) at $enginePath"
                $engine.Kill()
                if (-not $engine.WaitForExit(15000)) { throw 'Engine exit timed out.' }
            }
        } catch {
            if (-not $engine.HasExited) { throw }
        }
    }
    if (@(Matching-Processes 'notely-engine' $enginePath).Count -gt 0) {
        throw 'Notely engine is still running.'
    }
    Write-CleanupLog 'Cleanup completed; no matching engine remains.'
    exit 0
} catch {
    Write-CleanupLog $_.Exception.Message
    exit 1
}
