param(
    [Parameter(Mandatory=$true)][string]$Installer,
    [Parameter(Mandatory=$true)][string]$InstallDir,
    [Parameter(Mandatory=$true)][int]$AppPid,
    [Parameter(Mandatory=$false)][string]$TargetExe = ''
)
$ErrorActionPreference = 'Stop'
$launched = $false
try {
    # 1. Wait for Notely and its engine to release installed files.
    Wait-Process -Id $AppPid -Timeout 60 -ErrorAction SilentlyContinue
    $pWait = 0
    while ((Get-Process -Id $AppPid -ErrorAction SilentlyContinue) -and ($pWait -lt 30)) {
        Start-Sleep -Milliseconds 500
        $pWait++
    }
    
    # 2. Run NSIS installer silently in-place directly overwriting files
    # NSIS requires /D= to be unquoted even if the path contains spaces
    $psi = New-Object System.Diagnostics.ProcessStartInfo
    $psi.FileName = $Installer
    $psi.Arguments = "/S /D=$InstallDir"
    $psi.UseShellExecute = $false
    $installerProc = [System.Diagnostics.Process]::Start($psi)
    $installerProc.WaitForExit()
    if ($installerProc.ExitCode -ne 0) { throw "Installer selesai dengan kode $($installerProc.ExitCode)." }

    # 3. Wait for NSIS sub-installer worker processes (e.g. Au_.exe in ~nsu.tmp) to completely finish
    $maxWaitSec = 90
    $startWait = [System.Diagnostics.Stopwatch]::StartNew()
    while ($startWait.Elapsed.TotalSeconds -lt $maxWaitSec) {
        $workerFound = $false
        try {
            $procs = Get-Process -ErrorAction SilentlyContinue
            foreach ($p in $procs) {
                if ($p.ProcessName -like 'Au_*' -or $p.ProcessName -like '*nsu*') {
                    $workerFound = $true
                    break
                }
            }
        } catch {}

        if (-not $workerFound) {
            Start-Sleep -Milliseconds 1500
            $workerFound2 = $false
            try {
                $procs2 = Get-Process -ErrorAction SilentlyContinue
                foreach ($p2 in $procs2) {
                    if ($p2.ProcessName -like 'Au_*' -or $p2.ProcessName -like '*nsu*') {
                        $workerFound2 = $true
                        break
                    }
                }
            } catch {}
            if (-not $workerFound2) { break }
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
        for ($lockCheck = 0; $lockCheck -lt 15; $lockCheck++) {
            try {
                $fileStream = [System.IO.File]::Open($TargetExe, [System.IO.FileMode]::Open, [System.IO.FileAccess]::Read, [System.IO.FileShare]::ReadWrite)
                $fileStream.Close()
                $fileStream.Dispose()
                break
            } catch {
                Start-Sleep -Milliseconds 400
            }
        }
        Start-Sleep -Milliseconds 800
        Start-Process -FilePath $TargetExe -WorkingDirectory $InstallDir
        $launched = $true
    }
} catch {
    $logMsg = "[$(Get-Date -Format 'yyyy-MM-dd HH:mm:ss')] Update error: $($_.Exception.Message)"
    Add-Content -Path (Join-Path $env:TEMP 'notely-update.log') -Value $logMsg -ErrorAction SilentlyContinue
    if (-not $launched) {
        Add-Type -AssemblyName System.Windows.Forms
        [System.Windows.Forms.MessageBox]::Show("Update belum selesai. Buka Notely dan coba lagi.`n$($_.Exception.Message)", 'Notely Update') | Out-Null
    }
} finally {
    # Delete only this updater's exact download, including on cancellation.
    for ($attempt = 0; $attempt -lt 30; $attempt++) {
        Remove-Item -LiteralPath $Installer -Force -ErrorAction SilentlyContinue
        if (-not (Test-Path -LiteralPath $Installer)) { break }
        Start-Sleep -Seconds 2
    }
    Remove-Item -LiteralPath $PSCommandPath -Force -ErrorAction SilentlyContinue
}
