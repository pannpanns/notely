param(
    [Parameter(Mandatory=$true)][string]$Installer,
    [Parameter(Mandatory=$true)][string]$InstallDir,
    [Parameter(Mandatory=$true)][int]$AppPid,
    [Parameter(Mandatory=$false)][string]$TargetExe = ''
)
$ErrorActionPreference = 'Stop'
try {
    # 1. Wait for Notely and its engine to release installed files.
    Wait-Process -Id $AppPid -Timeout 60 -ErrorAction SilentlyContinue
    $pWait = 0
    while ((Get-Process -Id $AppPid -ErrorAction SilentlyContinue) -and ($pWait -lt 30)) {
        Start-Sleep -Milliseconds 500
        $pWait++
    }
    
    # 2. Run NSIS installer silently in-place directly overwriting files
    $process = Start-Process -FilePath $Installer -ArgumentList ('/S', '/D=' + $InstallDir) -PassThru
    $process.WaitForExit()
    if ($process.ExitCode -ne 0) { throw "Installer selesai dengan kode $($process.ExitCode)." }

    # 3. Wait for NSIS sub-installer worker processes (e.g. Au_.exe in ~nsu.tmp) to completely finish
    $maxWaitSec = 90
    $startWait = [System.Diagnostics.Stopwatch]::StartNew()
    while ($startWait.Elapsed.TotalSeconds -lt $maxWaitSec) {
        $nsisWorkers = Get-Process | Where-Object {
            ($_.ProcessName -like 'Au_*' -or $_.ProcessName -like '*nsu*') -or
            ($_.Path -and ($_.Path -like '*~nsu*' -or $_.Path -like "*$InstallDir*updater*"))
        } -ErrorAction SilentlyContinue

        if (-not $nsisWorkers) {
            # Double check after 1.5 seconds to confirm no subsequent stage is starting
            Start-Sleep -Milliseconds 1500
            $nsisWorkers2 = Get-Process | Where-Object {
                ($_.ProcessName -like 'Au_*' -or $_.ProcessName -like '*nsu*') -or
                ($_.Path -and ($_.Path -like '*~nsu*' -or $_.Path -like "*$InstallDir*updater*"))
            } -ErrorAction SilentlyContinue
            if (-not $nsisWorkers2) { break }
        } else {
            Start-Sleep -Milliseconds 500
        }
    }

    # 4. Extra safety pause for Windows file handles to release
    Start-Sleep -Seconds 2

    # 5. Resolve target executable
    if (-not $TargetExe -or -not (Test-Path -LiteralPath $TargetExe)) {
        $TargetExe = Join-Path $InstallDir 'Notely.exe'
    }
    if (-not (Test-Path -LiteralPath $TargetExe)) {
        $candidate = Get-ChildItem -Path $InstallDir -Filter "*.exe" -File -ErrorAction SilentlyContinue |
            Where-Object { $_.Name -notmatch 'uninstall|update|Au_' } | Select-Object -First 1
        if ($candidate) { $TargetExe = $candidate.FullName }
    }

    # 6. Relaunch Notely so the user immediately sees the updated app
    if (Test-Path -LiteralPath $TargetExe) {
        for ($lockCheck = 0; $lockCheck -lt 10; $lockCheck++) {
            try {
                $fileStream = [System.IO.File]::Open($TargetExe, [System.IO.FileMode]::Open, [System.IO.FileAccess]::Read, [System.IO.FileShare]::ReadWrite)
                $fileStream.Close()
                $fileStream.Dispose()
                break
            } catch {
                Start-Sleep -Milliseconds 500
            }
        }
        Start-Process -FilePath $TargetExe -WorkingDirectory $InstallDir
    }
} catch {
    Add-Type -AssemblyName System.Windows.Forms
    [System.Windows.Forms.MessageBox]::Show("Update belum selesai. Buka Notely dan coba lagi.`n$($_.Exception.Message)", 'Notely Update') | Out-Null
} finally {
    # Delete only this updater's exact download, including on cancellation.
    for ($attempt = 0; $attempt -lt 30; $attempt++) {
        Remove-Item -LiteralPath $Installer -Force -ErrorAction SilentlyContinue
        if (-not (Test-Path -LiteralPath $Installer)) { break }
        Start-Sleep -Seconds 2
    }
    Remove-Item -LiteralPath $PSCommandPath -Force -ErrorAction SilentlyContinue
}
